use crate::{Config, Output};
use axum::{extract::State, http::HeaderMap, Json, Router};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

//
// Payload Types
//
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

//
// Structures for Query Results
//
#[derive(Serialize)]
pub struct PlayerInfo {
    pub email: String,
    pub solved: u32,
}

#[derive(Serialize)]
pub struct SubmissionInfo {
    pub email: String,
    pub problem: u32,
    pub language: String,
    pub code: String,
    pub timestamp: String,
}

//
// Admin Output Enum
//
#[derive(Serialize)]
#[serde(tag = "status", content = "message")]
pub enum AdminOutput {
    Success(String),
    Failure(String),
    Unauthorized,
    Players(Vec<PlayerInfo>),
    Submissions(Vec<SubmissionInfo>),
}

//
// Helper: Check if the request is authorized for admin endpoints.
//
async fn is_auth(headers: HeaderMap, conf: &Config) -> bool {
    use axum::http::header::AUTHORIZATION;
    headers.contains_key(AUTHORIZATION)
        && headers[AUTHORIZATION]
            .to_str()
            .map(|tok| conf.verify_admin_token(tok))
            .unwrap_or(false)
}

//
// Handlers
//
#[tracing::instrument(name = "authorize", skip(conf, password))]
async fn authorize(State(conf): State<Config>, Json(password): Json<String>) -> Json<Output> {
    debug!("Received admin authorization attempt");
    match conf.get_admin_token(&password) {
        Some(tok) => {
            info!("Admin authorized successfully");
            Json(Output::Token(tok))
        }
        None => {
            info!("Admin authorization failed");
            Json(Output::Unauthorized)
        }
    }
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
    }
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
            error!(
                "Database query error while checking player existence for {}",
                payload.email
            );
            return Json(AdminOutput::Failure("Database error".to_string()));
        }
    }
    let hashed = match Config::argon2_generate(&payload.number) {
        Some(hash) => hash,
        None => {
            error!("Failed to generate password hash for {}", payload.email);
            return Json(AdminOutput::Failure("Failed to hash password".to_string()));
        }
    };
    conf.execute(
        "INSERT INTO players VALUES (?1, ?2, 0)",
        params![payload.email.clone(), hashed],
    )
    .await;
    info!("Added new player with email {}", payload.email);
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
    }
    let hashed = match Config::argon2_generate(&payload.number) {
        Some(hash) => hash,
        None => {
            error!("Failed to generate password hash for {}", payload.email);
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
    info!("Updated password for player {}", payload.email);
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
    }
    conf.execute(
        "UPDATE players SET solved = ?1 WHERE email = ?2",
        params![payload.count, payload.email.clone()],
    )
    .await;
    info!(
        "Set solved counter for player {} to {}",
        payload.email, payload.count
    );
    Json(AdminOutput::Success(
        "Player solved counter set successfully".to_string(),
    ))
}

#[tracing::instrument(name = "get_players", skip(headers, conf))]
async fn get_players(headers: HeaderMap, State(conf): State<Config>) -> Json<AdminOutput> {
    use libsql::params;
    if !is_auth(headers.clone(), &conf).await {
        return Json(AdminOutput::Unauthorized);
    }
    let mut rows = match conf
        .query("SELECT email, solved FROM players", params![])
        .await
    {
        Some(rows) => rows,
        None => {
            error!("Database query error in get_players");
            return Json(AdminOutput::Failure("Database error".to_string()));
        }
    };
    let mut players = Vec::new();
    while let Ok(Some(row)) = rows.next().await {
        let email = match row.get_str(0) {
            Ok(val) => val.to_string(),
            Err(e) => {
                error!("Error getting email: {:?}", e);
                continue;
            }
        };
        let solved = match row.get::<u64>(1) {
            Ok(val) => val as u32,
            Err(e) => {
                error!("Error getting solved: {:?}", e);
                continue;
            }
        };
        players.push(PlayerInfo { email, solved });
    }
    info!("Retrieved {} players", players.len());
    Json(AdminOutput::Players(players))
}

#[tracing::instrument(name = "get_submissions", skip(headers, conf))]
async fn get_submissions(headers: HeaderMap, State(conf): State<Config>) -> Json<AdminOutput> {
    use libsql::params;
    if !is_auth(headers.clone(), &conf).await {
        return Json(AdminOutput::Unauthorized);
    }
    let mut rows = match conf
        .query(
            "SELECT email, problem, language, code, timestamp FROM submissions",
            params![],
        )
        .await
    {
        Some(rows) => rows,
        None => {
            error!("Database query error in get_submissions");
            return Json(AdminOutput::Failure("Database error".to_string()));
        }
    };
    let mut submissions = Vec::new();
    while let Ok(Some(row)) = rows.next().await {
        let email = match row.get_str(0) {
            Ok(val) => val.to_string(),
            Err(e) => {
                error!("Error getting email: {:?}", e);
                continue;
            }
        };
        let problem = match row.get::<u64>(1) {
            Ok(val) => val as u32,
            Err(e) => {
                error!("Error getting problem: {:?}", e);
                continue;
            }
        };
        let language = match row.get_str(2) {
            Ok(val) => val.to_string(),
            Err(e) => {
                error!("Error getting language: {:?}", e);
                continue;
            }
        };
        let code = match row.get_str(3) {
            Ok(val) => val.to_string(),
            Err(e) => {
                error!("Error getting code: {:?}", e);
                continue;
            }
        };
        let timestamp = match row.get_str(4) {
            Ok(val) => val.to_string(),
            Err(e) => {
                error!("Error getting timestamp: {:?}", e);
                continue;
            }
        };
        submissions.push(SubmissionInfo {
            email,
            problem,
            language,
            code,
            timestamp,
        });
    }
    info!("Retrieved {} submissions", submissions.len());
    Json(AdminOutput::Submissions(submissions))
}

//
// Router
//
pub fn admin_page() -> Router<Config> {
    use axum::routing;
    Router::new()
        // .route("/", routing::get(web_page))
        .route("/auth", routing::post(authorize))
        .route("/add_player", routing::post(add_player))
        .route("/player_password", routing::post(player_password))
        .route("/set_counter", routing::post(set_counter))
        .route("/get_players", routing::get(get_players))
        .route("/get_submissions", routing::get(get_submissions))
}
