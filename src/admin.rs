use axum::{body::Body, extract::State, response::Html, Json, Router};

use crate::Config;

async fn web_page() -> Html<String> {
    Html(include_str!("admin.html").to_string())
}

async fn authorize(State(conf): State<Config>, Json(password): Json<String>) -> Result<Body, Body> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    if Argon2::default()
        .verify_password(
            password.as_bytes(),
            &PasswordHash::new(&conf.admin_hash).map_err(|e| {
                eprintln!("{e}");
                Body::from("Auth Error")
            })?,
        )
        .is_ok()
    {
        Ok(Body::from(format!("Token: {}", conf.admin_token)))
    } else {
        Ok(Body::from("Unauthorized"))
    }
}

pub fn admin_page() -> Router<Config> {
    use axum::routing;
    Router::new()
        .route("/", routing::get(web_page))
        .route("/auth", routing::post(authorize))
}
