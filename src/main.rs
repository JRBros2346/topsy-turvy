use axum::http::StatusCode;

use topsy_turvy::handle_submit;
use topsy_turvy::authorize;

#[tokio::main]
async fn main() {
    use axum::routing;
    use axum::Router;
    axum::serve(
        tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap(),
        Router::new()
        .route("/api/auth", routing::post(authorize))
            .route("/api/submit", routing::post(handle_submit))
            .fallback(|| async { (StatusCode::NOT_FOUND, "404 Not Found") }),
    )
    .await
    .unwrap();
}
