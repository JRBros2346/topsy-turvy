use crate::{Config, Output};
use axum::{extract::State, http::HeaderMap, Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Player {
    pub email: String,
    pub number: String,
}

#[derive(Deserialize)]
pub struct SetCounter {
    pub email: String,
    pub count: u32,
}

async fn is_auth(headers: HeaderMap, conf: &Config) -> bool {
    use axum::http::header::AUTHORIZATION;
    headers.contains_key(AUTHORIZATION)
        && headers[AUTHORIZATION]
            .to_str()
            .map(|tok| conf.verify_admin_token(tok))
            .unwrap_or(false)
}

#[tracing::instrument(name = "authorize", skip(conf, password))]
async fn authorize(State(conf): State<Config>, Json(password): Json<String>) -> Json<Output> {
    tracing::debug!("Recieved admin authorization attempt");
    match conf.get_admin_token(&password) {
        Some(tok) => {
            tracing::info!("Admin authorized successfully");
            Json(Output::Token(tok))
        }
        None => {
            tracing::info!("Admin authorization failed");
            Json(Output::Unauthorized)
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "status", content = "message")]
pub enum AdminOutput {
    Success(String),
    Failure(String),
    Unauthorized,
}

#[tracing::instrument(name = "add_player", skip(conf, payload))]
async fn add_player(
    headers: HeaderMap,
    State(conf): State<Config>,
    Json(payload): Json<Player>,
) -> Json<AdminOutput> {
    use libsql::params;
    if !is_auth(headers, &conf).await {
        return Json(AdminOutput::Unauthorized);
    };
    match conf
        .query(
            "SELECT email FROM players WHERE email = ?1",
            params![payload.email.clone()],
        )
        .await
    {
        Some(mut rows) => {
            if let Ok(Some(_)) = rows.next().await {
                tracing::warn!("Player with email {} already exists", payload.email);
                return Json(AdminOutput::Failure("Player already exists".to_string()));
            }
        }
        None => {
            tracing::error!(
                "Database query error while checking player existence for {}",
                payload.email
            );
            return Json(AdminOutput::Failure("Database error".to_string()));
        }
    }
    let hashed = match Config::argon2_generate(&payload.number) {
        Some(hash) => hash,
        None => {
            tracing::error!("Failed to generate password hash for {}", payload.email);
            return Json(AdminOutput::Failure("Failed to hash password".to_string()));
        }
    };
    conf.execute(
        "INSERT INTO players VALUE (?1, ?2, 0)",
        params![payload.email.clone(), hashed],
    )
    .await;
    tracing::info!("Added new player with email {}", payload.email);
    Json(AdminOutput::Success(
        "Player added successfully".to_string(),
    ))
}
#[tracing::instrument(name = "player_password", skip(conf, payload))]
async fn player_password(
    headers: HeaderMap,
    State(conf): State<Config>,
    Json(payload): Json<Player>,
) -> Json<AdminOutput> {
    use libsql::params;
    if !is_auth(headers, &conf).await {
        return Json(AdminOutput::Unauthorized);
    };
    let hashed = match Config::argon2_generate(&payload.number) {
        Some(hash) => hash,
        None => {
            tracing::error!("Failed to generate password hash for {}", payload.email);
            return Json(AdminOutput::Failure(
                "Failed to hash new password".to_string(),
            ));
        }
    };
    conf.execute(
        "UPDATE players SET number = ?1 WHERE email = ?2",
        params![hashed, payload.email.clone()],
    )
    .await;
    tracing::info!("Updated password for player {}", payload.email);
    Json(AdminOutput::Success(
        "Password updated successfully".to_string(),
    ))
}

#[tracing::instrument(name = "set_counter", skip(conf, payload))]
async fn set_counter(
    headers: HeaderMap,
    State(conf): State<Config>,
    Json(payload): Json<SetCounter>,
) -> Json<AdminOutput> {
    use libsql::params;
    if !is_auth(headers, &conf).await {
        return Json(AdminOutput::Unauthorized);
    };
    conf.execute(
        "UPDATE players SET solved = ?1 WHERE email = ?2",
        params![payload.count, payload.email.clone()],
    )
    .await;
    tracing::info!(
        "Set solved counter for player {} to {}",
        payload.email,
        payload.count
    );
    Json(AdminOutput::Success(
        "Player solved counter set successfully".to_string(),
    ))
}

pub fn admin_page() -> Router<Config> {
    use axum::routing;
    Router::new()
        // .route("/", routing::get(web_page))
        .route("/auth", routing::post(authorize))
        .route("/add_player", routing::post(add_player))
        .route("/player_password", routing::post(player_password))
        .route("/reset_counter", routing::post(set_counter))
}
