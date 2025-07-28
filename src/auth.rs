use crate::{Config, Output};
use axum::extract::{Json, State};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Player {
    pub user_id: String,
    pub password: String,
}

#[tracing::instrument(name = "handle_ayth", skip(conf, payload))]
pub async fn handle_auth(
    State(conf): State<Config>,
    Json(payload): Json<Player>,
) -> Result<Json<Output>, Json<Output>> {
    use libsql::params;
    let mut rows = conf
        .query(
            "SELECT password FROM players WHERE user_id = ?1 LIMIT 1",
            params![payload.user_id.clone()],
        )
        .await
        .ok_or(Output::ServerError)?;
    let row = rows
        .next()
        .await
        .inspect_err(|e| tracing::error!("{e} {} {}", file!(), line!()))
        .map_err(|_| Output::ServerError)?;
    let row = row.ok_or(Output::Unauthorized)?;
    let password = row
        .get_str(0)
        .inspect_err(|e| tracing::error!("{e} {} {}", file!(), line!()))
        .map_err(|_| Output::ServerError)?
        .to_string();
    tracing::debug!(
        "Recieved password from DB for user_id {}: {}",
        payload.user_id,
        password
    );
    let verify = Config::argon2_verify(&payload.password, &password);
    tracing::debug!("argon2_verify result: {:?}", verify);
    if verify == Some(true) {
        let encrypted = conf.encrypt(&payload.user_id).ok_or(Output::ServerError)?;
        tracing::debug!("Encryption successful for user_id {}", payload.user_id);
        Ok(Json(Output::Token(encrypted)))
    } else {
        Err(Json(Output::Unauthorized))
    }
}
