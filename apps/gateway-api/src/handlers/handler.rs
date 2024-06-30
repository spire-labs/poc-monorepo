use axum::extract::Query;
use axum::response::IntoResponse;
use axum::{http::StatusCode, Json};
use axum_macros::debug_handler;
use ethers::{
    contract::abigen,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, Signature, TxHash, U256},
    utils::keccak256,
};
use ethers_contract::Contract;
use k256::ecdsa::{RecoveryId, Signature as Secp256k1Signature, VerifyingKey};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde_json::{json, Value};
use sha3::{Digest, Keccak256};
use spvm_rs::*;
use std::env;
use std::str::FromStr;
use std::sync::Arc;

use crate::router::AppState;
use crate::services::{MutationDB, QueryDB};
use crate::utils::abi::{
    decode_tx_content, encode_preconf_payload, encode_tx_content, strip_0x_prefix,
};
use crate::utils::{response::*, types::*};

use entity::{enforcer_metadata, preconf_commitment, preconf_status};

pub async fn health_check() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
        "status": "ok" })),
    )
}

pub async fn not_found_404() -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "message": "The requested resource was not found",
        })),
    )
}

#[debug_handler]
pub async fn request_preconfirmation(
    state: axum::extract::State<AppState>,
    data: Json<SubmitPreconfirmationRequest>,
) -> StatusCode {
    // for debugging purposes, print the data
    // println!("{:?}", data);
    println!("[Debug] request_preconfirmation handler");
    let tx_content_string = data.tx_content.to_string();
    let tx_content_str = strip_0x_prefix(&tx_content_string);
    println!("Tx Content String: {}", tx_content_str);

    let tx_content = match decode_tx_content(tx_content_str) {
        Ok(content) => content,
        Err(e) => {
            println!("Error decoding tx content: {:?}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    // save status of this transacition hash to the database as PENDING
    let tx = preconf_status::ActiveModel {
        tx_hash: Set(data.tx_hash.clone().to_string()),
        status: Set("PENDING".to_string()),
    };
    match tx.insert(&*state.db).await {
        Ok(_) => (),
        Err(e) => {
            println!("Error inserting: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    // check that the transaction hash is a keccak256 hash of the transaction content
    let tx_hash = keccak256(&data.tx_content);
    if tx_hash.to_vec() == data.tx_hash.to_vec() {
        println!("Transaction hash is valid");
    } else {
        println!("Transaction hash is invalid");
        return StatusCode::BAD_REQUEST;
    }

    let election_address = {
        let contract_data = state.contract_data.lock().unwrap();
        contract_data.chain_a.election.address.clone()
    };
    // let election_abi = contract_data.chain_a.election.abi;

    // get the address of the next enforcer
    abigen!(Election, "contracts/ElectionContract.json");

    let provider_url = env::var("PROVIDER").unwrap();
    let provider = match Provider::<Http>::try_from(provider_url) {
        Ok(provider) => provider,
        Err(e) => {
            println!("Error creating provider: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    let client = Arc::new(provider);

    let election_contract = Election::new(election_address, client);

    let mut next_enforcer;
    let block_num = {
        let mut block_num = state.block_number.lock().unwrap();
        *block_num += 2;
        *block_num - 2
    };

    if let Ok(add) = election_contract
        .get_winner(U256::from(block_num))
        .call()
        .await
    {
        next_enforcer = add;
    } else {
        println!("Error getting winner");
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    // determine the amount to tip
    let tip = price_inclusion_preconfirmation(&tx_content, data.tx_hash.to_string()).await;

    let pv_key = match env::var("PRIVATE_KEY") {
        Ok(key) => key,
        Err(e) => {
            println!("Error getting private key: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    let wallet = match pv_key.parse::<LocalWallet>() {
        Ok(wallet) => wallet,
        Err(_e) => {
            println!("Error parsing private key");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    let nonce = {
        let mut nonce = state.nonce.lock().unwrap();
        *nonce += 1;
        *nonce - 1
    };

    // generate tip_tx to send
    let raw_tip_tx = TransactionContent {
        from: wallet.address(),
        tx_type: 1,
        tx_param: TransactionParams::Transfer(TransferTransactionParams {
            token_ticker: "ETH".to_string(),
            to: next_enforcer,
            amount: tip,
        }),
        nonce,
    };

    let raw_tip_tx_encoded = encode_tx_content(&raw_tip_tx);

    // sign the tip_tx
    let tip_hash = TxHash::from_slice(&keccak256(&raw_tip_tx_encoded));
    let signed_tip_tx = match wallet.sign_hash(tip_hash) {
        Ok(sig) => sig,
        Err(e) => {
            println!("Error signing tip tx: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    let tip_tx_hash = keccak256(&raw_tip_tx_encoded);

    let tip_tx = Transaction {
        tx_content: raw_tip_tx,
        tx_hash: tip_tx_hash.into(),
        signature: signed_tip_tx,
    };

    // get preconf contract address
    let preconfer_contract = {
        let contract_data = state.contract_data.lock().unwrap();
        contract_data.chain_a.slashing.address.clone() // clone the data while we have the lock
    };
    // generate the preconf request object
    let preconf_request = PreconfirmationPayload {
        transaction: Transaction {
            tx_content,
            tx_hash: TxHash::from_str(&data.tx_hash.to_string()).unwrap(),
            signature: Signature::from_str(&data.signature.to_string()).unwrap(),
        },
        tip_tx,
        preconfer_contract,
    };

    // send the preconf request to the enforcer's api url
    let enforcer_url: String = get_enforcer_url_by_address(next_enforcer.to_string(), &state.db)
        .await
        .expect("Failed to get enforcer url");

    let mut preconf_commitment;
    // send preconf request to enforcer's api
    if let Some(commitment) = send_preconf_request(&preconf_request, enforcer_url)
        .await
        .unwrap()
    {
        preconf_commitment = commitment;
    } else {
        let tx = preconf_status::ActiveModel {
            tx_hash: Set(data.tx_hash.clone().to_string()),
            status: Set("DENIED".to_string()),
        };
        if let Err(e) = tx.update(&*state.db).await {
            println!("Error inserting: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
        return StatusCode::BAD_REQUEST;
    }

    // check that the preconf commitment is valid - Signature and Block
    let encoded_preconf_payload = encode_preconf_payload(&preconf_request);
    let preconf_payload_hash = TxHash::from_slice(&keccak256(&encoded_preconf_payload));
    preconf_commitment
        .commitment
        .verify(preconf_payload_hash, preconf_commitment.signer)
        .unwrap();

    if U256::from(block_num) != preconf_commitment.block_number {
        return StatusCode::BAD_REQUEST;
    }

    // update status of this transaction to APPROVED
    let tx = preconf_status::ActiveModel {
        tx_hash: Set(data.tx_hash.clone().to_string()),
        status: Set("APPROVED".to_string()),
    };
    match tx.update(&*state.db).await {
        Ok(_) => (),
        Err(e) => {
            println!("Error inserting: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    // store preconfirmation in db
    let preconf = preconf_commitment::ActiveModel {
        tx_hash: Set(data.tx_hash.clone().to_string()),
        commitment: Set(preconf_commitment.commitment.to_string()),
        status: Set("APPROVED".to_string()),
    };

    match preconf.insert(&*state.db).await {
        Ok(_) => (),
        Err(e) => {
            println!("Error inserting: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    // return 200 OK
    StatusCode::OK
}

// send preconf request to enforcer's api
pub async fn send_preconf_request(
    preconf_request: &PreconfirmationPayload,
    enforcer_ip: String,
) -> Result<Option<PreconfirmationCommitment>, reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/request_preconfirmation", enforcer_ip))
        .json(preconf_request)
        .send()
        .await?;

    if res.status().is_success() {
        let commitment = res.json::<PreconfirmationCommitment>().await?;
        Ok(Some(commitment))
    } else {
        Ok(None)
    }
}

// returns the enforcer url for a given enforcer address
// accesses the url determined at setup
pub async fn get_enforcer_url_by_address(
    address: String,
    db: &DatabaseConnection,
) -> Result<String, Box<dyn std::error::Error>> {
    let ip = enforcer_metadata::Entity::find()
        .filter(enforcer_metadata::Column::Address.eq(address))
        .one(db)
        .await?
        .unwrap();

    Ok(ip.url)
}

// returns the wei value of the tip
pub async fn price_inclusion_preconfirmation(
    _tx_content: &TransactionContent,
    _tx_hash: String,
) -> u16 {
    return 100;
}

#[debug_handler]
pub async fn get_preconf_status(
    state: axum::extract::State<AppState>,
    params: Query<RequestPreconfirmationStatus>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let status = QueryDB::get_preconf_status_by_tx_hash(&state.db, params.tx_hash.clone()).await;

    match status {
        Ok(status) => {
            let status = json!({
                "data": to_status_response(status.unwrap())
            });
            return Ok(Json(status));
        }
        Err(e) => {
            return Err(to_db_error_response(e));
        }
    }
}

#[debug_handler]
pub async fn get_wallet_balance(
    state: axum::extract::State<AppState>,
    params: Query<CheckBalance>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    dotenvy::dotenv().ok();
    let rpc_url = env::var("RPC_URL").expect("RPC_URL is not set in .env file");
    let provider: Arc<Provider<Http>> = Arc::new(Provider::<Http>::try_from(rpc_url).unwrap());

    let contract_data = state.contract_data.lock().unwrap().clone().to_owned();
    let contract_a = contract_data.chain_a.spvm;
    let contract_b = contract_data.chain_b.spvm;

    let mut rollup_contract_address = Default::default();
    let mut rollup_contract_abi = Default::default();

    if params.rollup_contract == contract_a.address {
        rollup_contract_address = contract_a.address;
        rollup_contract_abi = contract_a.abi;
    } else if params.rollup_contract == contract_b.address {
        contract_b.address;
        contract_b.abi;
    } else {
        return Err(to_wrong_address_error());
    }

    let user_address: Address = params.address;
    let ticker = params.token_ticker.clone();

    let contract = Contract::new(rollup_contract_address, rollup_contract_abi, provider);

    //returns user balance from spvm contract mapping
    let balance = contract
        .method("getBalance", (ticker, user_address))
        //TODO: wrap this error in response method
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(format!("ABI error: {}", e))),
            )
        })?
        .call()
        .await;
    match balance {
        Ok(balance) => {
            return Ok(to_balance_response(balance));
        }
        Err(e) => {
            return Err(to_contract_error_response(e.to_string()));
        }
    }
}

#[debug_handler]
pub async fn create_enforcer_challenge(
    state: axum::extract::State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let challenge = MutationDB::create_challenge_string(&state.db).await;

    match challenge {
        Ok(challenge) => {
            let challenge = json!({
                "data": to_challenge_response(challenge.unwrap())
            });
            return Ok(Json(challenge));
        }
        Err(e) => {
            return Err(to_db_error_response(e));
        }
    }
}

#[debug_handler]
pub async fn register_enforcer(
    state: axum::extract::State<AppState>,
    payload: Json<RegisterEnforcerMetadata>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let enforcer_address = payload.address.clone();

    let msg = payload.challenge_string.as_bytes();

    let hex_sig = hex::decode(payload.signature.clone()).map_err(|e| to_hex_error(e))?;
    let signature_slice = hex_sig.as_slice();
    let signature =
        Secp256k1Signature::from_slice(signature_slice).map_err(|e| to_ecdsa_error(e))?;
    println!("{:?}", signature);

    let recovery_id = RecoveryId::try_from(0u8).map_err(|e| to_ecdsa_error(e))?;
    println!("{:?}", recovery_id);

    let recovered_vk =
        VerifyingKey::recover_from_digest(Keccak256::new_with_prefix(msg), &signature, recovery_id)
            .unwrap();

    let key_point = recovered_vk.to_encoded_point(false);
    println!("{:?}", key_point.to_bytes());
    let pub_key = key_point.as_bytes();

    let pub_key_hash = keccak256(&pub_key[1..]); //TODO: refactor so we don't use keccak from two different places (ethers/sha3)
    let recovered_address =
        // Have to format H160 to lower hex because of some display issues, otherwise value gets truncated
        format!("{:#x}", Address::from_slice(&pub_key_hash[12..32])).to_string();
    println!("{:?}", recovered_address);

    if enforcer_address == recovered_address {
        //lookup enforcer info in database. If already included in db ignore, else store enforcer info.
        let query = QueryDB::get_enforcer_by_address(&state.db, enforcer_address.clone())
            .await
            .unwrap();

        match query {
            Some(_value) => Ok(to_already_registered_response()),
            None => {
                let enforcer = MutationDB::register_enforcer_metadata(&state.db, payload.0).await;
                match enforcer {
                    Ok(enforcer) => {
                        let enforcer = json!({
                            "message": "Enforcer successfully registered",
                            "data": to_register_enforcer_response(enforcer.unwrap()),
                        });
                        return Ok((StatusCode::OK, Json(enforcer))); //TODO: Figure out why I had to manually add the status code here.
                    }
                    Err(e) => {
                        return Err(to_db_error_response(e));
                    }
                }
            }
        }
    } else {
        return Err(to_wrong_enforcer_address_error());
    }
}
