use crate::{Config, Output};
use axum::extract::{Json, State};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Player {
    pub email: String,
    pub number: String,
}

#[tracing::instrument(name = "handle_ayth", skip(conf, payload))]
pub async fn handle_auth(
    State(conf): State<Config>,
    Json(payload): Json<Player>,
) -> Result<Json<Output>, Json<Output>> {
    use libsql::params;
    let mut rows = conf
        .query(
            "SELECT number FROM players WHERE email = ?1 LIMIT 1",
            params![payload.email.clone()],
        )
        .await
        .ok_or(Output::ServerError)?;
    let row = rows
        .next()
        .await
        .inspect_err(|e| tracing::error!("{e} {} {}", file!(), line!()))
        .map_err(|_| Output::ServerError)?;
    let row = row.ok_or(Output::Unauthorized)?;
    let number = row
        .get_str(0)
        .inspect_err(|e| tracing::error!("{e} {} {}", file!(), line!()))
        .map_err(|_| Output::ServerError)?
        .to_string();
    tracing::debug!(
        "Recieved number from DB for email {}: {}",
        payload.email,
        number
    );
    let verify = Config::argon2_verify(&payload.number, &number);
    tracing::debug!("argon2_verify result: {:?}", verify);
    if verify == Some(true) {
        let encrypted = conf.encrypt(&payload.email).ok_or(Output::ServerError)?;
        tracing::debug!("Encryption successful for email {}", payload.email);
        Ok(Json(Output::Token(encrypted)))
    } else {
        Err(Json(Output::Unauthorized))
    }
}
