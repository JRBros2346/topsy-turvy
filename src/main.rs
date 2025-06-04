use axum::{extract::State, Router};
use tokio::net::TcpListener;

#[derive(Default, Clone)]
pub struct AppState {}

#[tokio::main]
async fn main() {
    let app = Router::new().with_state(AppState::default());
    axum::serve(TcpListener::bind("0.0.0.0:5000").await.unwrap(), app)
        .await
        .unwrap();
}

pub async fn submit(State(state): State<AppState>) {}
