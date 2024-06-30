use ethers::providers::{Http, Middleware, Provider};
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

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let database: DatabaseConnection = Database::connect(db_url).await.unwrap();

    Migrator::up(&database, None)
        .await
        .expect("Database connection failed");

    println!("let's do this!");

    // fetch deployed contract addresses and abis once on startup, and store in shared state
    let contract_data = fetch_contract_data()
        .await
        .expect("Failed to fetch contract data");

    let shared_data = Arc::new(Mutex::new(contract_data));

    let provider_url = env::var("RPC_URL").expect("PROVIDER is not set in .env file");
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
