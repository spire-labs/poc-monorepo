use ethers::abi::Address;
use ethers::providers::{Http, Middleware, Provider};
use ethers_contract::abigen;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use std::env;
use std::sync::{Arc, Mutex};
use tracing_subscriber;

mod handlers;
mod router;
mod services;
mod tests;
mod utils;

use crate::router::*;
use crate::utils::abi::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    dotenvy::dotenv().ok();

    println!("yay!");

    let db_url = env::var("GATEWAY_API_DB").expect("GATEWAY_API_DB is not set in .env file");
    let database: DatabaseConnection = Database::connect(db_url).await.unwrap();

    Migrator::up(&database, None)
        .await
        .expect("Database connection failed");

    dotenvy::dotenv().ok();
    // find all the events that have been emitted by SPVM contract (all user accounts)
    let chain_a_spvm_address_str = env::var("CHAIN_A_SPVM_CONTRACT_ADDRESS")
        .expect("CHAIN_A_SPVM_CONTRACT_ADDRESS is not set in .env file");
    let chain_a_spvm_address = chain_a_spvm_address_str.parse::<Address>().unwrap();
    let chain_b_spvm_address_str = env::var("CHAIN_B_SPVM_CONTRACT_ADDRESS")
        .expect("CHAIN_B_SPVM_CONTRACT_ADDRESS is not set in .env file");
    let chain_b_spvm_address = chain_b_spvm_address_str.parse::<Address>().unwrap();
    abigen!(SPVM, "contracts/SPVM.json");
    let provider_url = env::var("ANVIL_RPC_URL").unwrap();
    let provider = match Provider::<Http>::try_from(provider_url) {
        Ok(provider) => provider,
        Err(e) => {
            println!("Error creating provider: {:?}", e);
            return;
        }
    };
    let client = Arc::new(provider);

    let chain_a_contract = SPVM::new(chain_a_spvm_address, client.clone());
    let chain_b_contract = SPVM::new(chain_b_spvm_address, client.clone());

    let addresses_to_fetch = vec![
        "0xa0ee7a142d267c1f36714e4a8f75612f20a79720",
        "0x5fc8d32690cc91d4c39d9d3abcbd16989f875707",
        "0x70997970c51812dc3a010c7d01b50e0d17dc79c8",
        "0xa0ee7a142d267c1f36714e4a8f75612f20a79720",
        "0x4253252263d15e795263458c0b85d63a0bf465df",
    ];

    let enforcer_db_url =
        env::var("ENFORCER_DB_URL").expect("ENFORCER_DB_URL is not set in .env file");
    let enforcer_database: DatabaseConnection = Database::connect(enforcer_db_url).await.unwrap();
    // clear the `state` table of enforcer database
    // Update balance

    for address_str in addresses_to_fetch {
        let address = address_str.parse::<Address>().unwrap();
        let balance = chain_a_contract
            .get_balance(String::from("RAIN"), address)
            .call()
            .await
            .unwrap();
        println!("Balance of {}: {}", address, balance);
        // insert into database
    }

    println!("let's do this!");

    // fetch deployed contract addresses and abis once on startup, and store in shared state
    let contract_data = fetch_contract_data()
        .await
        .expect("Failed to fetch contract data");

    let shared_data = Arc::new(Mutex::new(contract_data));

    let provider_url = env::var("ANVIL_RPC_URL").expect("ANVIL_RPC_URL is not set in .env file");
    let provider = Provider::<Http>::try_from(provider_url).unwrap();
    let current_block = provider.get_block_number().await.unwrap();
    let app_state = AppState {
        db: Arc::new(database),
        contract_data: shared_data,
        nonce: Arc::new(Mutex::new(0)),
        block_number: Arc::new(Mutex::new(current_block.clone().as_u64())),
    };

    println!("here we go!");

    let app = router().with_state(app_state);

    let port = env::var("GATEWAY_API_PORT")
        .unwrap_or_else(|_| "5433".to_string())
        .parse::<u16>()
        .expect("GATEWAY_API_PORT must be a valid u16");

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
