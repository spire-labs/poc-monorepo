use dotenvy::dotenv;
use ethers::contract::abigen;
use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Middleware, Provider};
use ethers::signers::LocalWallet;
use ethers::signers::Signer;
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::types::{Address, Eip2930TransactionRequest, TransactionRequest, U128, U256, U64};
use ethers::types::{Bytes, Eip1559TransactionRequest};
use ethers::utils::{format_units, hex, parse_units};
use std::convert::TryFrom;
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // get latest block number
    dotenv().ok();
    let provider_url = std::env::var("ANVIL_RPC_URL").expect("ANVIL_RPC_URL is not set in .env file");
    let chain_a_election_address_str = std::env::var("CHAIN_A_ELECTION_ADDRESS")
        .expect("CHAIN_A_ELECTION_ADDRESS is not set in .env file");
    let private_key = std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY is not set in .env file");
    let account = private_key.parse::<LocalWallet>()?;
    let provider = Provider::<Http>::try_from(provider_url)?;
    let signer = SignerMiddleware::new(provider, account.with_chain_id(31337u64));
    let client = Arc::new(signer);
    abigen!(ElectionContract, "../contracts/ElectionContract.json");
    let chain_a_election_address: Address = chain_a_election_address_str.parse()?;
    let chain_a_election_contract =
        ElectionContract::new(chain_a_election_address, Arc::new(client.clone()));

    let mut prev_block_number = 0;
    loop {
        let current_block_number = client.get_block_number().await?.as_u64();
        // println!("Current block number: {:?}", current_block);
        if current_block_number > prev_block_number {
            prev_block_number = current_block_number;
            println!("New block detected: {:?}", current_block_number);
            let get_winner_call =
                chain_a_election_contract.get_winner(U256::from(prev_block_number));
            let get_winner_tx_hash = get_winner_call.call().await;
            println!("Get winner tx hash: {:?}", get_winner_tx_hash);
            let refresh_tickets_tx_hash = chain_a_election_contract.refresh_tickets().call().await;
            println!("Refresh tickets tx hash: {:?}", refresh_tickets_tx_hash);
        }
    }
}
