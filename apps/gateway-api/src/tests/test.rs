use axum::{
    routing::{get, post},
    Router,
};
use sea_orm::{
    DatabaseBackend, //MockDatabase TODO: mock feature flag needs to be enabled in toml file, this breaks regular db connections
};
use tracing_subscriber;

use crate::handlers::handler::*;
use crate::router::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    //axum::serve(listener, app()).await.unwrap();

    // TODO: Once mock flag is working for seaorm uncomment this block
    /*
        /// Having a function that produces our app makes it easy to call it from tests
        /// without having to create an HTTP server.
        fn app() -> Router<AppState> {
            Router::new()
                .route("/", get(health_check))
                .route("/request_preconfirmation", post(request_preconfirmation))
                //.route("/debug_contract_data", get(get_contract_data))
                //.route("/preconfirmation_status:tx_id", get(get_preconf_status))
                .route("/request_balance", get(get_wallet_balance))
                //.route("/request_preconfirmation", post(create_preconf_request))
                .route(
                    "/register_preconfirmation_commitment",
                    post(register_preconf_commitment),
                )
                .route("/preconfirmation_status", get(get_preconf_status))
                .route(
                    "/enforcer_metadata",
                    get(create_enforcer_challenge).post(register_enforcer),
                )
                .fallback(not_found_404)
                .with_state(AppState { db: MockDatabase::new(DatabaseBackend::Postgres) })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use axum::{
            body::Body,
            http::{Request, StatusCode},
        };
        use http_body_util::BodyExt; // for `collect`
        use serde_json::{json, Value};
        use tower::ServiceExt; // for `call`, `oneshot`, and `ready`

        #[tokio::test]
        async fn not_found_404() {
            let app = app();

            let request = Request::builder()
                .method("GET")
                .uri("/does-not-exist")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                    "message": "The requested resource was not found"
                }"#,
                ))
                .unwrap();

            // `Router` implements `tower::Service<Request<Body>>` so we can
            // call it like any tower service, no need to run an HTTP server.
            let response = app.oneshot(request).await.unwrap();

            assert_eq!(response.status(), StatusCode::NOT_FOUND);
            let body = response.into_body().collect().await.unwrap().to_bytes();
            let body: Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(body, json!({ "data": [1, 2,] }));
        }
        */
}
