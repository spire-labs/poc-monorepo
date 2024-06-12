use axum::{http::StatusCode, Json};
use ethers::abi::Uint;
use hex::FromHexError;
use k256::ecdsa;
use sea_orm::DbErr;
use serde::{Deserialize, Serialize};
use serde_json::json;

use ::entity::{
    challenge::Model as ChallengeModel, enforcer_metadata::Model as EnforcerMetadataModel,
    preconf_status::Model as StatusModel,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct MetadataChallengeResponse {
    pub challenge: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterEnforcerResponse {
    pub address: String,
    pub name: String,
    pub pre_conf_contracts: Vec<String>,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PreconfirmationStatusResponse {
    pub tx_hash: String,
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BalanceResponse {
    pub balance: Uint,
}

pub fn to_status_response(status: StatusModel) -> PreconfirmationStatusResponse {
    PreconfirmationStatusResponse {
        tx_hash: status.tx_hash.to_owned(),
        status: status.status.to_owned(),
    }
}

pub fn to_challenge_response(challenge: ChallengeModel) -> MetadataChallengeResponse {
    MetadataChallengeResponse {
        challenge: challenge.challenge.to_owned(),
    }
}

pub fn to_db_error_response(error: DbErr) -> (StatusCode, axum::Json<serde_json::Value>) {
    return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"status": "error","message": format!("{:?}", error)})),
    );
}

pub fn to_balance_response(balance: Uint) -> (StatusCode, axum::Json<serde_json::Value>) {
    return (
        StatusCode::OK,
        Json(json!({"status": "ok","balance": format!("{:?}", balance)})),
    );
}

pub fn to_register_enforcer_response(enforcer: EnforcerMetadataModel) -> RegisterEnforcerResponse {
    RegisterEnforcerResponse {
        address: enforcer.address.to_owned(),
        name: enforcer.name.to_owned(),
        pre_conf_contracts: enforcer.pre_conf_contracts.to_owned(),
        url: enforcer.url.to_owned(),
    }
}

pub fn to_contract_error_response(error: String) -> (StatusCode, axum::Json<serde_json::Value>) {
    return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"status": "error","message": format!("{:?}", error)})),
    );
}

pub fn to_ecdsa_error(error: ecdsa::Error) -> (StatusCode, axum::Json<serde_json::Value>) {
    return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"status": "error", "message": format!("{:?}", error)})),
    );
}

pub fn to_hex_error(error: FromHexError) -> (StatusCode, axum::Json<serde_json::Value>) {
    return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"status": "error", "message": format!("{:?}", error)})),
    );
}

pub fn to_wrong_address_error() -> (StatusCode, axum::Json<serde_json::Value>) {
    return (
        StatusCode::BAD_REQUEST,
        Json(json!({"status": "error", "message": "Wrong rollup contract address supplied"})),
    );
}

pub fn to_wrong_enforcer_address_error() -> (StatusCode, axum::Json<serde_json::Value>) {
    return (
        StatusCode::BAD_REQUEST,
        Json(
            json!({"status": "error", "message": "Enforcer address does not match challenge string/signature"}),
        ),
    );
}

pub fn to_already_registered_response() -> (StatusCode, axum::Json<serde_json::Value>) {
    return (
        StatusCode::OK,
        Json(json!({"status": "ok", "message": "Enforcer metadata already registered"})),
    );
}
