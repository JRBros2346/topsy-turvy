use axum::{body::Body, extract::State, http::StatusCode, Json};
use serde::Deserialize;

use crate::Config;

#[derive(Deserialize)]
pub struct Player {
    pub email: String,
    pub number: String,
}

pub async fn handle_auth(
    State(conf): State<Config>,
    Json(payload): Json<Player>,
) -> Result<Body, StatusCode> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    use libsql::params;
    if Argon2::default()
        .verify_password(
            payload.number.as_bytes(),
            &conf
                .conn
                .query(
                    "SELECT number FROM players WHERE email = ?1 LIMIT 1",
                    params![payload.email.clone()],
                )
                .await
                .map_err(|e| {
                    eprintln!("{e}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?
                .next()
                .await
                .map_err(|e| {
                    eprintln!("{e}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?
                .ok_or(StatusCode::UNAUTHORIZED)?
                .get_str(0)
                .map_err(|e| {
                    eprintln!("{e}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })
                .map(PasswordHash::new)?
                .map_err(|e| {
                    eprintln!("{e}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?,
        )
        .is_ok()
    {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        let mut mac = Hmac::<Sha256>::new_from_slice(conf.secret_key.as_bytes()).map_err(|e| {
            eprintln!("{e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        mac.update(payload.email.as_bytes());
        Ok(Body::from(format!(
            "Token: {}",
            mac.finalize().into_bytes()[..]
                .into_iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        )))
    } else {
        Ok(Body::from("Unauthorized"))
    }
}
