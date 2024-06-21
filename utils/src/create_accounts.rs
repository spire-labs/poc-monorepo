// create ethereum accounts

use coins_bip39::English;
use ethers::core::rand::thread_rng;
use ethers::prelude::*;
use ethers::signers::LocalWallet;
use ethers::{
    core::{types::TransactionRequest, utils::Anvil},
    providers::{Http, Middleware, Provider},
};
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let endpoint = "https://1rpc.io/sepolia";

    // connect to the network
    let provider = Provider::<Http>::try_from(endpoint)?;
    // Generate a new mnemonic

    let wallet = LocalWallet::new(&mut thread_rng());

    // Optionally, the wallet's chain id can be set, in order to use EIP-155
    // replay protection with different chains
    println!("Wallet address: {:#x}", wallet.address());
    println!(
        "Wallet private key: {}",
        wallet
            .signer()
            .to_bytes()
            .iter()
            .map(|&i| format!("{:02X}", i))
            .collect::<Vec<String>>()
            .join("")
    );
    // println!("Wallet private key: {:?}", wallet.signer().to_bytes());

    // The wallet can be used to sign messages
    let message = b"hello";
    let signature = wallet.sign_message(message).await?;
    assert_eq!(signature.recover(&message[..]).unwrap(), wallet.address());

    // LocalWallet is clonable:
    let wallet_clone = wallet.clone();
    let signature2 = wallet_clone.sign_message(message).await?;
    assert_eq!(signature, signature2);
    Ok(())
}
