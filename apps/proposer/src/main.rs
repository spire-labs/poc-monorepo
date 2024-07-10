use ethers::{
    contract::abigen,
    core::abi::{decode, encode, ParamType, Token},
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, Bytes, Signature, TxHash, U256},
    utils::keccak256,
};
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use spvm_rs::*;
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{self, Duration};

#[derive(Debug)]
struct TxEncoded {
    tx_content: TxContentEncoded,
    tx_hash: TxHash,
    signature: Bytes,
}

#[derive(Debug)]
struct TxContentEncoded {
    from: Address,
    tx_type: u8,
    tx_param: Bytes,
    nonce: u32,
}

// struct Block {
//     transactions: Vec<TxEncoded>,
//     block_hash: Bytes,
//     parent_hash: Bytes,
//     block_number: u32,
//     proposer: Address,
//     proposer_signature: Bytes,
// }

struct AppState {
    appchain_a: Appchain,
    appchain_b: Appchain,
    db: DatabaseConnection,
}

struct Appchain {
    parent_hash: [u8; 32],
    block_number: u32,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let db_path = env::var("PROPOSER_DB").unwrap();
    let db = Database::connect(db_path).await.unwrap();
    Migrator::up(&db, None)
        .await
        .expect("Database migration failed");

    let start_block_num = env::var("BLOCK_NUM")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0u32);

    let app_state = AppState {
        appchain_a: Appchain {
            parent_hash: [0u8; 32],
            block_number: start_block_num,
        },
        appchain_b: Appchain {
            parent_hash: [0u8; 32],
            block_number: start_block_num,
        },
        db,
    };

    let block_time = env::var("BLOCK_TIME")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(12);

    let shared_state = Arc::new(Mutex::new(app_state));
    let mut interval = time::interval(Duration::from_secs(block_time));

    loop {
        interval.tick().await;

        let provider_url = env::var("ANVIL_RPC_URL").unwrap();
        let provider = Provider::<Http>::try_from(provider_url).unwrap();
        let client = Arc::new(provider);

        let block_num = client.get_block_number().await.unwrap().as_u32();

        let state = Arc::clone(&shared_state);
        tokio::spawn(async move {
            let mut guard = state.lock().await;
            let a_txs = get_validity_conditions(block_num /* L1 Block number */, true)
                .await
                .unwrap();
            let b_txs = get_validity_conditions(block_num /* L1 Block number */, false)
                .await
                .unwrap();

            let mut new_b_txs: Vec<Transaction> = b_txs;

            for tx in &a_txs {
                tx.execute_transaction(&guard.db).await.unwrap();
                // check for certain tx to sponsor
                let pv_key = env::var("PROPOSER_PRIVATE_KEY").unwrap();
                let wallet = LocalWallet::from_str(&pv_key)
                    .unwrap()
                    .with_chain_id(31337u64);
                // is transfer?
                // is to address this address?
                if let TransactionParams::Transfer(tx_params) = tx.tx_content.tx_param.clone() {
                    if tx_params.to == wallet.address() {
                        // nowe we know this transaction is a bridge, next step is to check if it is a cross-chain swap or just a simple bridge
                        // for thie PoC, we assume every transfer with an odd amount is a bridge
                        if tx_params.amount % 2 == 1 {
                            // bridge
                            // construct a transfer transaction on appchain B that sends the bridged amount of the same token to the from address
                            // add it to new_b_txs
                            let new_tx = create_transfer_transaction(
                                &tx_params.token_ticker,
                                &wallet,
                                tx.tx_content.from,
                                tx_params.amount,
                                get_nonce_on_appchain(false).await.unwrap(),
                            );

                            new_b_txs.push(new_tx);
                        } else {
                            // cross-chain swap
                            // construct a transfer transaction on appchain B that sends the bridged amount of a different token to the from address
                            // add it to new_b_txs

                            let new_tx = create_transfer_transaction(
                                env::var("BRIDGE_TICKER").unwrap().as_str(),
                                &wallet,
                                tx.tx_content.from,
                                env::var("BRIDGE_AMOUNT").unwrap().parse::<u16>().unwrap(),
                                get_nonce_on_appchain(false).await.unwrap(),
                            );

                            new_b_txs.push(new_tx);
                        }
                    }
                }
            }
            for tx in &new_b_txs {
                tx.execute_transaction(&guard.db).await.unwrap();
            }

            let a_encoded_txs = encode_transactions(&a_txs);
            let b_encoded_txs = encode_transactions(&new_b_txs);

            let a_txs = a_txs
                .iter()
                .map(|tx| TxEncoded {
                    tx_hash: tx.tx_hash,
                    tx_content: TxContentEncoded {
                        from: tx.tx_content.from,
                        tx_type: tx.tx_content.tx_type,
                        tx_param: encode_tx_params(&tx.tx_content.tx_param),
                        nonce: tx.tx_content.nonce,
                    },
                    signature: tx.signature.to_vec().into(),
                })
                .collect();

            let b_txs = new_b_txs
                .iter()
                .map(|tx| TxEncoded {
                    tx_hash: tx.tx_hash,
                    tx_content: TxContentEncoded {
                        from: tx.tx_content.from,
                        tx_type: tx.tx_content.tx_type,
                        tx_param: encode_tx_params(&tx.tx_content.tx_param),
                        nonce: tx.tx_content.nonce,
                    },
                    signature: tx.signature.to_vec().into(),
                })
                .collect();

            let a_new_hash = propose_block(
                a_txs,
                a_encoded_txs,
                guard.appchain_a.parent_hash,
                guard.appchain_a.block_number,
                true,
            )
            .await
            .unwrap();
            let b_new_hash = propose_block(
                b_txs,
                b_encoded_txs,
                guard.appchain_b.parent_hash,
                guard.appchain_b.block_number,
                false,
            )
            .await
            .unwrap();

            guard.appchain_a.parent_hash = a_new_hash;
            guard.appchain_b.parent_hash = b_new_hash;

            guard.appchain_a.block_number += 2;
            guard.appchain_b.block_number += 2;
        });
    }
}

fn create_transfer_transaction(
    ticker: &str,
    from: &LocalWallet,
    to: Address,
    amount: u16,
    nonce: u32,
) -> Transaction {
    create_transaction(
        from,
        1,
        TransactionParams::Transfer(TransferTransactionParams {
            token_ticker: ticker.to_string(),
            to,
            amount,
        }),
        nonce,
    )
}

async fn get_nonce_on_appchain(appchain_a: bool) -> Result<u32, Box<dyn std::error::Error>> {
    let provider_url = env::var("ANVIL_RPC_URL")?;
    let provider = Provider::<Http>::try_from(provider_url)?;
    let client = Arc::new(provider);

    let spvm = if appchain_a {
        env::var("CHAIN_A_SPVM_CONTRACT_ADRESS")?
    } else {
        env::var("CHAIN_B_SPVM_CONTRACT_ADRESS")?
    };

    let spvm_address = spvm.parse::<Address>()?;

    abigen!(Spvm, "contracts/SPVM.json");

    let spvm = Spvm::new(spvm_address, client.clone());

    let pv_key = env::var("PROPOSER_PRIVATE_KEY")?;
    let wallet = LocalWallet::from_str(&pv_key)?.with_chain_id(31337u64);

    let nonce = spvm.nonces(wallet.address()).call().await?;

    Ok(nonce)
}

fn create_transaction(
    signer: &LocalWallet,
    tx_type: u8,
    tx_param: TransactionParams,
    nonce: u32,
) -> Transaction {
    let tx_content = TransactionContent {
        from: signer.address(),
        tx_type,
        tx_param,
        nonce,
    };

    let message = encode_tx_content(&tx_content);
    let tx_hash = TxHash::from_slice(&keccak256(message));
    let signature = signer.sign_hash(tx_hash).unwrap();

    Transaction {
        tx_content,
        tx_hash,
        signature,
    }
}

async fn get_validity_conditions(
    block_num: u32,
    appchain_a: bool,
) -> Result<Vec<spvm_rs::Transaction>, Box<dyn std::error::Error>> {
    abigen!(Slashing, "contracts/Slashing.json");
    let provider_url = env::var("ANVIL_RPC_URL")?;
    let provider = Provider::<Http>::try_from(provider_url)?;
    let client = Arc::new(provider);

    let slashing_address: String = if appchain_a {
        env::var("CHAIN_A_SLASHING_CONTRACT_ADDRESS")?
    } else {
        env::var("CHAIN_B_SLASHING_CONTRACT_ADDRESS")?
    };

    let slashing_address = slashing_address.parse::<Address>()?;

    let slashing = Slashing::new(slashing_address, client.clone());

    let validity_conditions = slashing
        .get_validity_conditions(U256::from(block_num))
        .call()
        .await?;
    println!("validity_conditions: {:?}", validity_conditions);

    let mut spvm_txs = Vec::new();

    for tx in validity_conditions {
        let tx_param_tokens = decode(
            &[ParamType::Tuple(vec![
                ParamType::String,
                ParamType::Address,
                ParamType::Uint(16),
            ])],
            &tx.tx_content.tx_param,
        )?;

        let spvm_tx = spvm_rs::Transaction {
            tx_hash: tx.tx_hash.into(),
            tx_content: TransactionContent {
                tx_type: tx.tx_content.tx_type,
                from: tx.tx_content.from,
                tx_param: decode_transaction_params(tx.tx_content.tx_type, tx_param_tokens)
                    .unwrap(),
                nonce: tx.tx_content.nonce,
            },
            signature: Signature::from_str(&tx.signature.to_string()).unwrap(),
        };
        spvm_txs.push(spvm_tx);
    }

    Ok(spvm_txs)
}

async fn propose_block(
    txs: Vec<TxEncoded>,
    tx_encoded: Bytes,
    parent_hash: [u8; 32],
    block_number: u32,
    appchain_a: bool,
) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let tx_hash = keccak256(&tx_encoded);

    let pv_key = env::var("PROPOSER_PRIVATE_KEY")?;
    let wallet = LocalWallet::from_str(&pv_key)?.with_chain_id(31337u64);

    let proposer_address = wallet.address();
    let signature = wallet.sign_hash(tx_hash.into())?;

    let provider_url = env::var("ANVIL_RPC_URL")?;
    let provider = Provider::<Http>::try_from(provider_url)?;

    let signer = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(signer);

    let spvm = if appchain_a {
        env::var("CHAIN_A_SPVM_CONTRACT_ADRESS")?
    } else {
        env::var("CHAIN_B_SPVM_CONTRACT_ADRESS")?
    };

    let spvm_address = spvm.parse::<Address>()?;

    abigen!(Spvm, "contracts/SPVM.json");

    let spvm = Spvm::new(spvm_address, client.clone());

    // convert txs to Vec<SpvmTransaction>
    let txs = txs
        .iter()
        .map(|tx| Spvmtransaction {
            tx_content: TransactionContent {
                from: tx.tx_content.from,
                tx_type: tx.tx_content.tx_type,
                tx_param: tx.tx_content.tx_param.clone(),
                nonce: tx.tx_content.nonce,
            },
            transaction_hash: tx.tx_hash.into(),
            signature: tx.signature.clone(),
        })
        .collect();

    let mut block_num = client.get_block_number().await?.as_u64();

    while block_num % 2 != 1 {
        time::sleep(Duration::from_secs(1)).await;
        block_num = client.get_block_number().await?.as_u64();
    }

    println!("block_num: {:?}", block_num);
    println!("txs: {:?}", txs);

    let _ = spvm
        .propose_block(Block {
            transactions: txs,
            block_hash: tx_hash,
            parent_hash,
            block_number,
            proposer: proposer_address,
            proposer_signature: signature.to_vec().into(),
        })
        .send()
        .await?
        .await?;

    Ok(tx_hash)
}

fn encode_transactions(txs: &[Transaction]) -> Bytes {
    let tokens = Token::Array(
        txs.iter()
            .map(|tx| {
                let encoded_content = encode_tx_content(&tx.tx_content);
                Token::Tuple(vec![
                    Token::Bytes(encoded_content.to_vec()),
                    Token::Bytes(tx.tx_hash.as_bytes().to_vec()),
                    Token::Bytes(tx.signature.to_vec()),
                ])
            })
            .collect(),
    );

    encode(&[tokens]).into()
}
