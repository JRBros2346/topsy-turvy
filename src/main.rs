use axum::http::StatusCode;

mod code;
mod submit;

#[tokio::main]
async fn main() {
    use axum::routing;
    use axum::Router;
    axum::serve(
        tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap(),
        Router::new()
            .route("/api/submit", routing::post(submit::submit))
            .fallback(|| async { (StatusCode::NOT_FOUND, "404 Not Found") }),
    )
    .await
    .unwrap();
}
