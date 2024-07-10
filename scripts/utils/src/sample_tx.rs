use ethers::{
    core::utils::keccak256,
    signers::{LocalWallet, Signer, Wallet},
    types::{Address, Bytes, Signature, TxHash, H256},
};
use spvm_rs::*;
use std::str::FromStr;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let _ = get_sample_mint_tx().await;

    // let (tx_hash, signature, tx_content) = get_sample_mint_tx().await;

    // let request = format!(
    // 	"curl -X POST http://0.0.0.0:5433/request_preconfirmation -H \"Content-Type: application/json\" -d '{{\"tx_content\":\"{:?}\", \"tx_hash\":\"{:?}\", \"signature\":\"{:?}\"}}'",
    // 	tx_content, tx_hash, signature.to_string()
    // );

    // // To remove extra quotes and slashes, you can use `replace` method to clean the output
    // let cleaned_request = request.replace("\\\"", "\"").replace("'{", "{").replace("}'", "}");

    // // Print or use the cleaned_request variable
    // println!("{}", cleaned_request);
}

async fn get_sample_mint_tx() -> (H256, Signature, Bytes) {
    let priv_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let wallet = Wallet::from_str(priv_key).unwrap();
    let tx = create_mint_transaction("IS FALLING", &wallet, wallet.address(), 200, 0);

    println!("MINT tx: {:?}\n", tx);
    println!("tx_hash: {:?}\n", tx.tx_hash);
    // println!("signature: {:?}\n", tx.signature);
    println!("signature string: {:?}\n", tx.signature.to_string());
    println!(
        "encoded tx_content: {:?}",
        encode_tx_content(&tx.tx_content)
    );

    (tx.tx_hash, tx.signature, encode_tx_content(&tx.tx_content))
}

async fn get_sample_transfer_tx() {
    let priv_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let wallet = Wallet::from_str(priv_key).unwrap();
    let transfer_address = "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720"
        .parse::<Address>()
        .unwrap();
    let tx = create_transfer_transaction("RAIN", &wallet, transfer_address, 100, 1);

    println!("tx: {:?}\n", tx);
    println!("tx_hash: {:?}\n", tx.tx_hash);
    // println!("signature: {:?}\n", tx.signature);
    println!("signature string: {:?}\n", tx.signature.to_string());
    println!(
        "encoded tx_content: {:?}",
        encode_tx_content(&tx.tx_content)
    );
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

fn create_mint_transaction(
    ticker: &str,
    from: &LocalWallet,
    owner: Address,
    supply: u16,
    nonce: u32,
) -> Transaction {
    create_transaction(
        from,
        0,
        TransactionParams::Mint(MintTransactionParams {
            token_ticker: ticker.to_string(),
            owner,
            supply,
        }),
        nonce,
    )
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
