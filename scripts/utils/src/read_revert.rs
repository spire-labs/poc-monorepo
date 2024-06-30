use dotenvy::dotenv;
use ethers::contract::abigen;
use ethers::providers::{Http, Middleware, Provider};
use ethers::types::{Address, U256};
use std::convert::TryFrom;
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // get latest block number
    dotenv().ok();
    let provider_url = std::env::var("RPC_URL").expect("PROVIDER is not set in .env file");
    println!("Connecting to provider: {}", provider_url);
    let provider = Provider::<Http>::try_from(provider_url)?;

    let current_block = provider.get_block_number().await?;
    println!("Current block number: {:?}", current_block);
    Ok(())
}
