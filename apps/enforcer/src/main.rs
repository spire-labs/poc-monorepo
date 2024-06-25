use axum::{routing::post, Router};
use dotenv::dotenv;
use ethers::{
    contract::abigen,
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, Bytes, TxHash},
};
use local_ip_address::local_ip;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use serde::{Deserialize, Serialize};
use spvm_rs::*;
use std::env;
use std::sync::{Arc, Mutex};
use tokio::time::{self, Duration};
use std::collections::HashMap;

mod api;


type ValidityConditions = Arc<Mutex<HashMap<Address, Vec<Transaction>>>>;

#[derive(Clone)]
pub struct AppState {
    pub validity_txs: ValidityConditions,
    pub db: Arc<DatabaseConnection>,
    pub block_num: Arc<Mutex<u64>>,
}

struct TxEncoded {
    tx_hash: TxHash,
    tx_content: TxContentEncoded,
    signature: Bytes,
}

struct TxContentEncoded {
    from: Address,
    tx_type: u8,
    tx_param: Bytes,
    nonce: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MetadatPayload {
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub challenge: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RegisterEnforcerMetadata {
    pub address: String, //TODO: type check for address
    pub challenge_string: String,
    pub signature: String, //TODO: type check for hex encoded string
    pub name: String,
    pub preconf_contracts: Vec<String>, //TODO: type check for address
    pub url: String,                    //TODO: type check that is actually a url
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let validity_txs: ValidityConditions = Arc::new(Mutex::new(HashMap::new()));

    register_with_gateway().await;

    let validity_txs_clone = Arc::clone(&validity_txs);

    let block_time = env::var("BLOCK_TIME")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(12);

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(block_time));
        loop {
            interval.tick().await;
            let validity_txs_clone = Arc::clone(&validity_txs_clone);
            let result = submit_validity_condition(validity_txs_clone).await;

            match result {
                Ok(_) => tracing::info!("Successfully submitted validity condition."),
                Err(e) => tracing::error!("Failed to submit validity condition: {:?}", e),
            }
        }
    });

    let db_path = env::var("DB").unwrap();

    let db = match Database::connect(db_path).await {
        Ok(database) => {
            tracing::info!("Successfully connected to database.");
            Some(database)
        }
        Err(e) => {
            tracing::error!("Failed to connect to database: {}", e);
            None
        }
    }
    .unwrap();

    Migrator::up(&db, None)
        .await
        .expect("Database migration failed");

    let port = env::var("ENFORCER_API_PORT")
        .unwrap_or_else(|_| "5555".to_string())
        .parse::<u16>()
        .expect("GATEWAY_API_PORT must be a valid u16");

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .unwrap();
    tracing::debug!("Enforcer listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app(db)).await.unwrap();
}

fn app(db: DatabaseConnection) -> Router {
    let validity_txs: ValidityConditions = Arc::new(Mutex::new(HashMap::new()));

    let start_block_num = env::var("BLOCK_NUM")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(1);

    let shared_state = AppState {
        validity_txs: validity_txs.clone(),
        db: Arc::new(db),
        block_num: Arc::new(Mutex::new(start_block_num)),
    };

    Router::new()
        .route(
            "/request_preconfirmation",
            post(api::request_preconfirmation).with_state(shared_state),
        )
        // .route("/metadata", post(api::metadata))
        .route("/apply_tx", post(api::apply_tx))
}

async fn submit_validity_condition(
    validity_txs: ValidityConditions,
) -> Result<(), Box<dyn std::error::Error>> {
    let validity_txs = {
        let mut validity_txs = validity_txs.lock().unwrap();
        let extracted = validity_txs.clone();
        validity_txs.clear();
        extracted
    };

	for (preconf_add, txs) in validity_txs.iter() {
		let transactions: Vec<TxEncoded> = txs
			.iter()
			.map(|tx| {
				let encoded_tx_param = encode_tx_params(&tx.tx_content.tx_param);
				TxEncoded {
					tx_hash: tx.tx_hash,
					tx_content: TxContentEncoded {
						from: tx.tx_content.from,
						tx_type: tx.tx_content.tx_type,
						tx_param: encoded_tx_param,
						nonce: tx.tx_content.nonce,
					},
					signature: tx.signature.to_vec().into(),
				}
			})
			.collect();

		abigen!(Slashing, "contracts/Slashing.json");

		let provider_url = env::var("PROVIDER")?;

		let provider = Provider::<Http>::try_from(provider_url)?;
		let client = SignerMiddleware::new(
			provider,
			env::var("PRIVATE_KEY")
				.unwrap()
				.parse::<LocalWallet>()
				.unwrap()
				.with_chain_id(31337u64),
		);

		let contract = Slashing::new(
				*preconf_add,
				Arc::new(client),
		);

		let transactions: Vec<Transaction> = transactions
			.into_iter()
			.map(|tx_enc| Transaction {
				tx_hash: tx_enc.tx_hash.into(),
				tx_content: PreconfTransactionContent {
					from: tx_enc.tx_content.from,
					tx_type: tx_enc.tx_content.tx_type,
					tx_param: tx_enc.tx_content.tx_param,
					nonce: tx_enc.tx_content.nonce,
				},
				signature: tx_enc.signature,
			})
			.collect();

		let _ = contract
			.submit_validity_conditions(transactions)
			.send()
			.await?
			.await?;
	}
    Ok(())
}

async fn register_with_gateway() {
    let gateway_ip = env::var("GATEWAY_IP").unwrap();
    let client = reqwest::Client::new();

    let challenge_string: MetadatPayload = client
        .get(format!("{}/enforcer_metadata", gateway_ip))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    println!("challenge_string: {:?}", challenge_string);

    let pv_key = env::var("PRIVATE_KEY").unwrap();
    let wallet = pv_key.parse::<LocalWallet>().unwrap();
    let commitment = wallet
        .sign_message(challenge_string.data.challenge.as_bytes())
        .await
        .unwrap();

    let resp = RegisterEnforcerMetadata {
        address: wallet.address().to_string(),
        challenge_string: challenge_string.data.challenge,
        signature: commitment.to_string(),
        name: env::var("ENFORCER_NAME").unwrap(),
        preconf_contracts: vec![env::var("PRECONF_CONTRACT").unwrap()],
        url: local_ip().unwrap().to_string(),
    };

    let _ = client
        .post(format!("{}/enforcer_metadata", gateway_ip))
        .json(&resp)
        .send()
        .await
        .unwrap();
}

#[cfg(test)]
mod tests {
    use self::api::PreconfirmationPayload;

    use super::*;
    use axum::{
        body::Body,
        http::{self, Request, StatusCode},
    };
    use entity::*;
    use ethers::{
        core::rand::thread_rng,
        core::utils::keccak256,
        signers::{LocalWallet, Signer},
        types::{Address, TxHash},
    };
    use sea_orm::{entity::prelude::*, Database, DbBackend, Schema};
    use serde_json::json;
    use tower::ServiceExt;

    async fn setup() -> (LocalWallet, DatabaseConnection) {
        let wallet0 = LocalWallet::new(&mut thread_rng());

        let db = Database::connect("sqlite::memory:").await.unwrap();
        setup_schema(&db).await;

        (wallet0, db)
    }

    async fn setup_schema(db: &DbConn) {
        let schema = Schema::new(DbBackend::Sqlite);

        let stmt0 = schema.create_table_from_entity(state::Entity);
        let stmt1 = schema.create_table_from_entity(nonces::Entity);
        let stmt2 = schema.create_table_from_entity(initialized_tickers::Entity);

        let _result = db.execute(db.get_database_backend().build(&stmt0)).await;

        let _result = db.execute(db.get_database_backend().build(&stmt1)).await;

        let _result = db.execute(db.get_database_backend().build(&stmt2)).await;
    }

    fn create_transaction(
        signer: &LocalWallet,
        tx_type: u8,
        tx_param: TransactionParams,
        nonce: u32,
    ) -> Transaction {
        let tx_content = TransactionContent {
            from: signer.address(),
            tx_type,
            tx_param,
            nonce,
        };

        let message = serde_json::to_vec(&tx_content).unwrap();
        let tx_hash = TxHash::from_slice(&keccak256(message));
        let signature = signer.sign_hash(tx_hash).unwrap();

        Transaction {
            tx_content,
            tx_hash,
            signature,
        }
    }

    fn create_mint_transaction(
        ticker: &str,
        from: &LocalWallet,
        owner: Address,
        supply: u16,
        nonce: u32,
    ) -> Transaction {
        create_transaction(
            from,
            0,
            TransactionParams::Mint(MintTransactionParams {
                token_ticker: ticker.to_string(),
                owner,
                supply,
            }),
            nonce,
        )
    }

    fn create_transfer_transaction(
        ticker: &str,
        from: &LocalWallet,
        to: Address,
        amount: u16,
        nonce: u32,
    ) -> Transaction {
        create_transaction(
            from,
            1,
            TransactionParams::Transfer(TransferTransactionParams {
                token_ticker: ticker.to_string(),
                to,
                amount,
            }),
            nonce,
        )
    }

    fn create_preconf_payload_mint(
        ticker: &str,
        from: &LocalWallet,
        to: Address,
        supply: u16,
        nonce: u32,
    ) -> PreconfirmationPayload {
        let tx = create_mint_transaction(ticker, from, to, supply, nonce);

        PreconfirmationPayload {
            transaction: tx.clone(),
            tip_tx: tx,
            preconfer_contract: "0x4253252263d15e795263458C0B85d63A0BF465df"
                .parse()
                .unwrap(),
        }
    }

    fn create_preconf_payload_transfer(
        ticker: &str,
        from: &LocalWallet,
        to: Address,
        amount: u16,
        nonce: u32,
    ) -> PreconfirmationPayload {
        let tx = create_transfer_transaction(ticker, from, to, amount, nonce);

        PreconfirmationPayload {
            transaction: tx.clone(),
            tip_tx: tx,
            preconfer_contract: "0x4253252263d15e795263458C0B85d63A0BF465df"
                .parse()
                .unwrap(),
        }
    }

    #[tokio::test]
    async fn metadata() {
        let db_path = env::var("DB").unwrap();

        let db = Database::connect(db_path).await.unwrap();
        let app = app(db);

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/metadata")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(
                        serde_json::to_vec(&json!({
                            "challange": "hello"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn request_preconfirmation() {
        let db_path = env::var("DB").unwrap();

        let db = Database::connect(db_path).await.unwrap();
        let app = app(db);

        let (wallet0, _db) = setup().await;

        let preconf_payload =
            create_preconf_payload_mint("ABCD", &wallet0, wallet0.address(), 100, 0);

        let response = match app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/request_preconfirmation")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(serde_json::to_vec(&preconf_payload).unwrap()))
                    .unwrap(),
            )
            .await
        {
            Ok(response) => response,
            Err(e) => {
                println!("Error: {}", e);
                return;
            }
        };

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn request_preconfirmation_fail() {
        let db_path = env::var("DB").unwrap();

        let db = Database::connect(db_path).await.unwrap();
        let app = app(db);

        let (wallet0, _db) = setup().await;

        let preconf_payload =
            create_preconf_payload_transfer("ABCD", &wallet0, wallet0.address(), 100, 0);

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/request_preconfirmation")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(serde_json::to_vec(&preconf_payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
