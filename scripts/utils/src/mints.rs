use ethers::abi::{encode, Token};
use ethers::types::{Signature, TxHash};
use ethers::{
    signers::{Signer, Wallet},
    types::{Address, H256},
    utils::keccak256,
};
use ethers_core::types::Bytes;
use serde::{Deserialize, Serialize};
use spvm_rs::*;
use std::{env, str::FromStr};

#[derive(Deserialize, Serialize, Debug)]
pub struct PrivilegedTransaction {
    pub tx_hash: Bytes,
    pub tx_content: Bytes,
    pub signature: Bytes,
    pub preconfer_contract: Address,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    println!("Hello, world!");

    let priv_key = env::var("PRIVATE_KEY").unwrap();
    let wallet = Wallet::from_str(&priv_key).unwrap();
    let tx_params = TransactionParams::Mint(MintTransactionParams {
        token_ticker: "ETH".to_string(),
        supply: 1000,
        owner: Address::from_str("0x70997970c51812dc3a010c7d01b50e0d17dc79c8 ").unwrap(),
    });
    println!("Wallet address: {:?}", wallet.address());
    let tx_content = TransactionContent {
        from: wallet.address(),
        tx_type: 0,
        tx_param: tx_params,
        nonce: 0,
    };
    let tx_content_bytes: Bytes = encode_tx_content(&tx_content);
    // mint 1000 eth tokens to the gateway address
    // sign the tip_tx
    let tx_hash = keccak256(&tx_content_bytes);
    let tx_hash_slice = TxHash::from_slice(&tx_hash);
    let signature: Signature = wallet.sign_hash(tx_hash_slice).unwrap();
    let token = Token::Bytes(signature.into());
    println!("Signature Token: {:?}", token);
    let signature: Bytes = Bytes::from(token.into_bytes().unwrap());
    // let tx_hash_bytes = Bytes::from_str(format!("{:?}", tx_hash_slice).as_str()).unwrap();
    // println!("Signature: {:?}", signature);
    // println!("Tx Hash: {:?}", tx_hash);
    //  turn [247, 127, 216, 141, 225, 209, 116, 193, 97, 90, 12, 79, 177, 108, 53, 107, 255, 245, 163, 12, 14, 195, 92, 35, 150, 192, 129, 82, 147, 194, 61, 58]
    // into hex
    let tx_hash_str = tx_hash
        .iter()
        .map(|&i| format!("{:02X}", i))
        .collect::<Vec<String>>()
        .join("");
    // println!("Tx Hash: {:?}", tx_hash_str);
    let tx_hash_bytes = Bytes::from(tx_hash);
    // let signature = Bytes::from_str("ffd3fda67b6b799db2bfc72e5c7993e66d5ba74c66cace23ce1abd80fc33355064b36100d2d776e306754c9c274712c46491fb3641cd80120f9aed6af38dc6321c").unwrap();
    let preconfer_contract =
        Address::from_str("0x2279B7A0a67DB372996a5FaB50D91eAA73d2eBe6").unwrap();

    // print signature length
    println!("Signature length: {:?}", signature.len());
    send_mint_request(
        tx_hash_bytes,
        tx_content_bytes,
        signature,
        preconfer_contract,
    )
    .await;
}

async fn send_mint_request(
    tx_hash: Bytes,
    tx_content: Bytes,
    signature: Bytes,
    preconfer_contract: Address,
) {
    println!("Tx Hash: {:?}", tx_hash);
    println!("Tx Content: {:?}", tx_content);
    println!("Signature: {:?}", signature);
    println!("Preconfer Contract: {:?}", preconfer_contract);
    let client = reqwest::Client::new();

    let tx = PrivilegedTransaction {
        tx_hash,
        tx_content,
        signature,
        preconfer_contract,
    };

    let _res = client
        // enter enfrocer url here
        .post("http://localhost:5555/apply_tx")
        // with json body
        .json(&tx)
        .send()
        .await;
    match _res {
        Ok(res) => {
            println!("Response: {:?}", res);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
