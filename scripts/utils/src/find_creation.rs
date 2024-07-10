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
    let provider_url =
        std::env::var("ANVIL_RPC_URL").expect("ANVIL_RPC_URL is not set in .env file");
    let chain_a_election_address_str = std::env::var("CHAIN_A_ELECTION_ADDRESS")
        .expect("CHAIN_A_ELECTION_ADDRESS is not set in .env file");
    let chain_a_election_address = chain_a_election_address_str.parse::<Address>()?;
    let chain_a_election_address =
        "0xce0066b1008237625dDDBE4a751827de037E53D2".parse::<Address>()?;
    println!(
        "Looking for contract creation transactions for contract address: {:?}",
        chain_a_election_address
    );
    let private_key = std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY is not set in .env file");
    let account = private_key.parse::<LocalWallet>()?;
    let provider = Provider::<Http>::try_from(provider_url)?;
    let signer = SignerMiddleware::new(provider, account.with_chain_id(31337u64));
    let client = Arc::new(signer);
    let current_block_number = client.get_block_number().await?.as_u64();
    for block_number in 0..current_block_number {
        // get list of transactions
        let txs = client.get_block_with_txs(block_number).await?.unwrap();
        let txs = txs.transactions;
        for tx in txs {
            // println!("{:?}", tx);
            if tx.to == None {
                // generate transaction receipt
                let tx_receipt = client.get_transaction_receipt(tx.hash).await?.unwrap();
                if tx_receipt.contract_address == Some(chain_a_election_address) {
                    println!("Contract creation transaction found");
                    println!("Block number: {:?}", block_number);
                    println!("Tx Hash {:?}", tx.hash);
                    // println!("Tx {:?}", tx);
                    // println!("Receipt {:?}", tx_receipt);
                }
            }
        }
    }
    Ok(())
}
