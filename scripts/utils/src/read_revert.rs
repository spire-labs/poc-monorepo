use dotenvy::dotenv;
use ethers::contract::abigen;
use ethers::providers::{Http, Middleware, Provider};
use ethers::signers::LocalWallet;
use ethers::signers::Signer;
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::types::{Address, Eip2930TransactionRequest, TransactionRequest, U256};
use ethers::types::{Bytes, Eip1559TransactionRequest};
use ethers::utils::{format_units, hex, parse_units};
use std::convert::TryFrom;
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // get latest block number
    dotenv().ok();
    let provider_url =
        std::env::var("ANVIL_RPC_URL").expect("ANVIL_RPC_URL is not set in .env file");
    println!("Connecting to provider: {}", provider_url);
    let provider = Provider::<Http>::try_from(provider_url)?;

    let current_block = provider.get_block_number().await?;
    println!("Current block number: {:?}", current_block);

    let address = "0x443b5def71bd68b3cddcd320e576feb176eb6389";
    let private_key = "ADEFAF232449009E1807AF87347A668706AF14604FB8391FBCEC8D54E69380D0";
    let account = private_key.parse::<LocalWallet>()?;

    let address2 = "0xb39909620495017c398a1801378448813d66edde";
    let private_key2 = "45F5F615FDBCF5586E9C2822278CBE5F146023E42530544E166D436CDC8AA1C8";
    let account2 = private_key2.parse::<LocalWallet>()?;

    assert_eq!(account.address(), address.parse().unwrap());
    assert_eq!(account2.address(), address2.parse().unwrap());
    // connect to the network

    // get the balance of the first account
    let balance = provider.get_balance(address, None).await?;
    // print in ETH (18 decimals)
    println!(
        "Balance of {}: {} ETH",
        address,
        format_units(balance, 18).unwrap()
    );
    // transfer some funds from the first account to the second
    let from = account.address();
    let to = account2.address();
    // let tx = TransactionRequest::new()
    //     .to(to)
    //     .value(parse_units(0.05, "ether").unwrap())
    //     .from(from);
    let tx = TransactionRequest::new()
        .to(to)
        .value(parse_units(0.05, "ether").unwrap())
        .from(from);

    let balance_before = provider.get_balance(from, None).await?;
    let nonce1 = provider.get_transaction_count(from, None).await?;

    // sign
    let typed_tx = TypedTransaction::Eip2930(Eip2930TransactionRequest::new(
        tx.clone(),
        Default::default(),
    ));
    println!("{:?}", typed_tx);
    let raw_tx = account.sign_transaction(&typed_tx).await?;
    println!("{:?}", raw_tx.to_string());
    // convert signature to bytes
    let raw_tx_bytes: Bytes = hex::decode(raw_tx.to_string().as_bytes()).unwrap().into();
    println!("Raw tx: {:?}", raw_tx_bytes);
    let tx = provider.send_raw_transaction(raw_tx_bytes).await?.await?;
    // let tx = provider.send_transaction(tx, None).await?.await?;
    println!("{}", serde_json::to_string(&tx)?);

    let nonce2 = provider.get_transaction_count(from, None).await?;

    assert!(nonce1 < nonce2);

    let balance_after = provider.get_balance(from, None).await?;
    assert!(balance_after < balance_before);

    println!("Balance before {balance_before}");
    println!("Balance after {balance_after}");

    let current_block = provider.get_block_number().await?;
    println!("New block number: {:?}", current_block);

    Ok(())
}
