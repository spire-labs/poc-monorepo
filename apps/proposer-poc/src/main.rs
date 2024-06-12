use ethers::{
    contract::abigen,
    core::abi::{decode, encode, ParamType, Token},
    middleware::SignerMiddleware,
    providers::{Http, Provider},
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

struct TxEncoded {
    tx_content: TxContentEncoded,
    tx_hash: TxHash,
    signature: Bytes,
}

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
    parent_hash: [u8; 32],
    block_number: u32,
    db: DatabaseConnection,
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let db_path = env::var("DB_PATH").unwrap();
    let db = Database::connect(&db_path).await.unwrap();

    Migrator::up(&db, None)
        .await
        .expect("Database migration failed");

    let start_block_num = env::var("BLOCK_NUM")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0u32);

    let app_state = AppState {
        parent_hash: [0u8; 32],
        block_number: start_block_num,
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

        let state = Arc::clone(&shared_state);
        tokio::spawn(async move {
            let mut guard = state.lock().await;
            let txs = get_validity_conditions(guard.block_number).await.unwrap();

            for tx in &txs {
                tx.execute_transaction(&guard.db).await.unwrap();
            }

            let encoded_txs = encode_transactions(&txs);

            let txs = txs
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

            let new_hash = propose_block(txs, encoded_txs, guard.parent_hash, guard.block_number)
                .await
                .unwrap();

            guard.parent_hash = new_hash;
            guard.block_number += 2;
        });
    }
}

async fn get_validity_conditions(
    block_num: u32,
) -> Result<Vec<spvm_rs::Transaction>, Box<dyn std::error::Error>> {
    abigen!(Slashing, "contracts/Slashing.json");
    let provider_url = env::var("PROVIDER")?;
    let provider = Provider::<Http>::try_from(provider_url)?;
    let client = Arc::new(provider);

    let slashing_address = env::var("SLASHING_ADDRESS")?;
    let slashing_address = slashing_address.parse::<Address>()?;

    let slashing = Slashing::new(slashing_address, client.clone());

    let validity_conditions = slashing
        .get_validity_conditions(U256::from(block_num))
        .call()
        .await?;
    // println!("validity_conditions: {:?}", validity_conditions);

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
) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let tx_hash = keccak256(&tx_encoded);

    let pv_key = env::var("PRIVATE_KEY")?;
    let wallet = LocalWallet::from_str(&pv_key)?.with_chain_id(31337u64);

    let proposer_address = wallet.address();
    let signature = wallet.sign_hash(tx_hash.into())?;

    let provider_url = env::var("PROVIDER")?;
    let provider = Provider::<Http>::try_from(provider_url)?;

    let signer = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(signer);

    let spvm = env::var("SPVM_ADDRESS")?;
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
