use libsql::params;

#[tokio::main]
async fn main() {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };
    use axum::http::StatusCode;
    use axum::routing;
    use axum::Router;
    use libsql::Builder;
    use std::env;
    use topsy_turvy::{admin_page, handle_auth, handle_submit, Config};
    dotenv::dotenv().ok();
    let conn = Builder::new_local(env::current_dir().unwrap().join("revil.db"))
        .build()
        .await
        .unwrap()
        .connect()
        .unwrap();
    conn.execute(include_str!("init.sql"), params!())
        .await
        .unwrap();
    let salt = SaltString::generate(&mut OsRng);
    axum::serve(
        tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap(),
        Router::new()
            .nest("/admin", admin_page())
            .route("/api/submit", routing::post(handle_submit))
            .route("/api/auth", routing::post(handle_auth))
            .with_state(Config {
                conn,
                admin_hash: Argon2::default()
                    .hash_password(env::var("ADMIN_PASS").unwrap().as_bytes(), &salt)
                    .unwrap()
                    .to_string(),
                admin_token: env::var("ADMIN_TOKEN").unwrap(),
                secret_key: env::var("SECRET_KEY").unwrap(),
            })
            .fallback(|| async { (StatusCode::NOT_FOUND, "404 Not Found") }),
    )
    .await
    .unwrap();
}
