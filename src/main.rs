use axum::{
    http::StatusCode,
    {routing, Router},
};
use topsy_turvy::{admin_page, get_solved, handle_auth, handle_submit_with_db, Config};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_BACKTRACE", "full");
    dotenv::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();
    tracing::info!("Starting server...");
    axum::serve(
        tokio::net::TcpListener::bind("0.0.0.0:3000")
            .await
            .inspect(|_| tracing::info!("Listening on 0.0.0.0:3000"))
            .inspect_err(|e| tracing::error!("Failed to bind to port 3000: {e:?}"))
            .unwrap(),
        Router::new()
            .nest("/admin", admin_page())
            .route("/api/submit", routing::post(handle_submit_with_db))
            .route("/api/auth", routing::post(handle_auth))
            .with_state(Config::new().await)
            .fallback(|| async { (StatusCode::NOT_FOUND, "404 Not Found") }),
    )
    .await
    .inspect_err(|e| tracing::error!("Server error: {:?}", e))
    .unwrap();
}
