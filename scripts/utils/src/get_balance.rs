use dotenvy::dotenv;
use ethers::providers::{Http, Middleware, Provider};
use ethers::types::Address;
use std::convert::TryFrom;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let provider_url =
        std::env::var("ANVIL_RPC_URL").expect("ANVIL_RPC_URL is not set in .env file");
    println!("Connecting to provider: {}", provider_url);
    let provider = Provider::<Http>::try_from(provider_url)?;

    // let address: Address = "0x4253252263d15e795263458c0b85d63a0bf465df".parse()?;
    let address: Address = "0xE401FBb0d6828e9f25481efDc9dd18Da9E500983".parse()?;

    let balance = provider.get_balance(address, None).await?;

    println!("The balance is: {}", balance);

    Ok(())
}
