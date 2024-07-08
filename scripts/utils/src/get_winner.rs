use dotenvy::dotenv;
use ethers::contract::abigen;
use ethers::providers::{Http, Middleware, Provider};
use ethers::types::{Address, U256};
use std::convert::TryFrom;
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let provider_url = std::env::var("RPC_URL").expect("PROVIDER is not set in .env file");
    println!("Connecting to provider: {}", provider_url);
    let provider = Provider::<Http>::try_from(provider_url)?;

    // let address: Address = "0x4253252263d15e795263458c0b85d63a0bf465df".parse()?;
    let election_address: Address = "0xe7f1725e7734ce288f8367e1bb143e90bb3f0512".parse()?;
    abigen!(
        Election,
        "../../apps/gateway-api/contracts/ElectionContract.json"
    );
    let client = Arc::new(provider);
    let election_contract = Election::new(election_address, client);
    let winner = election_contract
        .get_winner(U256::from(291203023))
        .call()
        .await?;
    println!("Winner: {:?}", winner);
    Ok(())
}
