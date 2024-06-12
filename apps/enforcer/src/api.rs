use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_macros::debug_handler;
use dotenv::dotenv;
use ethers::{
    core::abi::{encode, Token},
    core::utils::keccak256,
    signers::{LocalWallet, Signer},
    types::{Address, Bytes, Signature, TxHash, U256},
};
use sea_orm::Database;
use serde::{Deserialize, Serialize};
use spvm_rs::{decode_tx_content, encode_tx_content, Transaction};
use std::env;
use std::str::FromStr;
use tracing::{error, info};

use crate::AppState;

#[derive(Deserialize, Serialize, Debug)]
pub struct PreconfirmationPayload {
    pub transaction: Transaction,
    pub tip_tx: Transaction,
    pub preconfer_contract: Address,
}

#[derive(Deserialize, Serialize)]
pub struct PreconfirmationCommitment {
    pub preconfirmation_request: PreconfirmationPayload,
    pub commitment: Signature,
    pub signer: Address,
    pub block_number: U256,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MetadatPayload {
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub challenge: String,
}

#[derive(Deserialize, Serialize)]
pub struct RegisterEnforcerMetadata {
    pub address: String, //TODO: type check for address
    pub challenge_string: String,
    pub signature: String, //TODO: type check for hex encoded string
    pub name: String,
    pub preconf_contracts: Vec<String>, //TODO: type check for address
    pub url: String,                    //TODO: type check that is actually a url
}

pub fn encode_preconf_payload(payload: &PreconfirmationPayload) -> Bytes {
    let encoded_tx = encode_transaction(&payload.transaction);
    let encoded_tip_tx = encode_transaction(&payload.tip_tx);

    let tokens = Token::Tuple(vec![
        Token::Bytes(encoded_tx.to_vec()),
        Token::Bytes(encoded_tip_tx.to_vec()),
        Token::Address(payload.preconfer_contract),
    ]);

    encode(&[tokens]).into()
}

pub fn encode_transaction(tx: &Transaction) -> Bytes {
    let encoded_cont = encode_tx_content(&tx.tx_content);
    let tokens = Token::Tuple(vec![
        Token::Bytes(encoded_cont.to_vec()),
        Token::Bytes(tx.tx_hash.as_bytes().to_vec()),
        Token::Bytes(tx.signature.into()),
    ]);

    encode(&[tokens]).into()
}

#[debug_handler]
pub async fn request_preconfirmation(
    State(state): State<AppState>,
    Json(payload): Json<PreconfirmationPayload>,
) -> impl IntoResponse {
    dotenv().ok();
    info!("Received request: {:?}", payload);

    let db = state.db;
    let transaction_result = payload.transaction.execute_transaction(&db).await;

    if let Err(e) = transaction_result {
        error!("Transaction execution failed: {}", e);
        return (StatusCode::BAD_REQUEST, format!("Transaction Error: {}", e)).into_response();
    }

    drop(transaction_result);

    let pv_key = match env::var("PRIVATE_KEY") {
        Ok(key) => key,
        Err(e) => {
            error!("Private key not found {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response();
        }
    };

    let wallet = match pv_key.parse::<LocalWallet>() {
        Ok(wallet) => wallet,
        Err(e) => {
            error!("Failed to parse private key: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response();
        }
    };

    let encoded_payload = encode_preconf_payload(&payload);
    let payload_hash = TxHash::from_slice(&keccak256(encoded_payload));
    let commitment = match wallet.sign_hash(payload_hash) {
        Ok(sig) => {
            info!("Message signed successfully.");
            sig
        }
        Err(e) => {
            error!("Failed to sign message: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response();
        }
    };

    let validity_txs = &state.validity_txs;
    let mut validity_txs = validity_txs
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .unwrap();

    validity_txs.push(payload.transaction.clone());
    validity_txs.push(payload.tip_tx.clone());

    let block_number = {
        let mut block_num = state.block_num.lock().unwrap();
        *block_num += 2;
        *block_num - 2
    };

    (
        StatusCode::OK,
        Json(PreconfirmationCommitment {
            preconfirmation_request: payload,
            commitment,
            signer: wallet.address(),
            block_number: U256::from(block_number),
        }),
    )
        .into_response()
}

/*
pub async fn metadata(Json(payload): Json<MetadatPayload>) -> impl IntoResponse {
    let pv_key = match env::var("PRIVATE_KEY") {
        Ok(key) => key,
        Err(e) => {
            error!("Private key not found {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response();
        }
    };

    let wallet = match pv_key.parse::<LocalWallet>() {
        Ok(wallet) => wallet,
        Err(e) => {
            error!("Failed to parse private key: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response();
        }
    };

    let commitment = match wallet.sign_message(payload.data.as_bytes()).await {
        Ok(sig) => {
            info!("Message signed successfully.");
            sig
        }
        Err(e) => {
            error!("Failed to sign message: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response();
        }
    };

    (
        StatusCode::OK,
        Json(MetadataResponse {
            challange_signature: commitment,
            address: wallet.address(),
            name: env::var("ENFORCER_NAME").unwrap(),
            // TODO - get supported preconf contracts from db
            supported_preconf_contracts: vec![Address::zero()],
        }),
    )
        .into_response()
}
*/

#[derive(Deserialize, Serialize, Debug)]
pub struct PrivilegedTransaction {
    pub tx_hash: Bytes,
    pub tx_content: Bytes,
    pub signature: Bytes,
}

pub async fn apply_tx(Json(payload): Json<PrivilegedTransaction>) -> StatusCode {
    let db_path = match env::var("DB") {
        Ok(path) => path,
        Err(e) => {
            error!("Database path not found: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    let db = match Database::connect(db_path).await {
        Ok(database) => {
            info!("Successfully connected to database.");
            database
        }
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            return StatusCode::BAD_GATEWAY;
        }
    };

    let tx_content = match decode_tx_content(&payload.tx_content.to_string()) {
        Ok(content) => content,
        Err(e) => {
            println!("Error decoding tx content: {:?}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    let tx = Transaction {
        tx_hash: TxHash::from_str(&payload.tx_hash.to_string()).unwrap(),
        tx_content,
        signature: Signature::from_str(&payload.signature.to_string()).unwrap(),
    };

    let transaction_result = tx.execute_transaction(&db).await;

    if let Err(e) = transaction_result {
        error!("Transaction execution failed: {}", e);
        return StatusCode::BAD_REQUEST;
    }

    StatusCode::OK
}
