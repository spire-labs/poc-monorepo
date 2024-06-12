use axum::{
    routing::{get, post},
    Router,
};
use sea_orm::DatabaseConnection;
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};

use crate::handlers::handler::*;
use crate::utils::abi::ContractData;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub contract_data: Arc<Mutex<ContractData>>,
    pub nonce: Arc<Mutex<u32>>,
    pub block_number: Arc<Mutex<u64>>,
}

pub fn router() -> Router<AppState> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/", get(health_check))
        .route("/request_preconfirmation", post(request_preconfirmation))
        .route("/request_balance", get(get_wallet_balance))
        .route("/preconfirmation_status", get(get_preconf_status))
        .route("/enforcer_metadata", get(create_enforcer_challenge))
        .route("/enforcer_metadata", post(register_enforcer))
        .fallback(not_found_404)
        .layer(cors)
}
