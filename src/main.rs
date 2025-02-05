use std::sync::Arc;

use axum::routing;
use axum::Router;
use config::AppState;
use routes::submission::submit;
use tokio;

pub mod config;
pub mod error;
pub mod routes;
pub mod models;

#[tokio::main]
async fn main() -> std::io::Result<()>{
    let router = Router::new()
        .route("/api/submit", routing::post(submit))
        .with_state(Arc::new(AppState::new()));
    Ok(
        axum::serve(
            tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap(),
            router.into_make_service()
        )
        .await
        .unwrap()
   )
}
