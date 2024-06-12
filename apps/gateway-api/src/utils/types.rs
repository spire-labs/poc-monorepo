use ethers::types::{Address, Bytes, Signature, U256};
use serde::{Deserialize, Serialize};
use spvm_rs::Transaction;

//For use by the wallet to query a user's balance on the spvm contract
#[derive(Deserialize, Serialize, Debug)]
pub struct CheckBalance {
    pub address: Address,
    pub token_ticker: String,
    pub rollup_contract: Address,
}

// To be submitted by the wallet to create the pre-confirmation request -- do we use RequestPreconfirmationData or SubmitPreconfirmationRequest?
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RequestPreconfirmationData {
    pub tx_hash: String,
    pub tx_content: String,
}

// To be submitted by the wallet to create the pre-confirmation request
#[derive(Deserialize, Serialize, Debug)]
pub struct SubmitPreconfirmationRequest {
    pub tx_hash: Bytes,
    pub tx_content: Bytes,
    pub signature: Bytes,
}

//Sent to enforcer after checking metadata associated with address
#[derive(Deserialize, Serialize, Debug)]
pub struct PreconfirmationPayload {
    pub transaction: Transaction,
    pub tip_tx: Transaction,
    pub preconfer_contract: Address,
}

// This is a response type. TODO: move to utils/response.rs
// To be submitted by the enforcer and stored in db for future use by slashing contract
#[derive(Deserialize, Serialize, Debug)]
pub struct PreconfirmationCommitment {
    pub preconfirmation_request: PreconfirmationPayload,
    pub commitment: Signature,
    pub signer: Address,
    pub block_number: U256,
}

// I think this is a response type, TODO: move to utils/response.rs
#[derive(Deserialize, Serialize, Debug)]
pub struct RequestPreconfirmationCommitment {
    pub tx_hash: String,
    pub commitment: PreconfirmationCommitment,
    pub status: String,
}

// Types for db reads/writes
//
//For enforcer metadata registration
#[derive(Deserialize, Serialize, Debug)]
pub struct RegisterEnforcerMetadata {
    pub address: String, //TODO: type check for address
    pub challenge_string: String,
    pub signature: String, //TODO: type check for hex encoded string
    pub name: String,
    pub preconf_contracts: Vec<String>, //TODO: type check for address
    pub url: String,                    //TODO: type check that is actually a url
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UpdatePreconfirmationStatus {
    pub tx_hash: String, //TODO: Type to txhash once db schema matches
    pub status: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RequestPreconfirmationStatus {
    pub tx_hash: String, //TODO: Type to txhash once db schema matches
}
