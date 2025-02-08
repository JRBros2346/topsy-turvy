use std::time::Duration;

use axum::{extract::State, http::HeaderMap, Json};
use libsql::params::IntoValue;
use serde::{Deserialize, Serialize};

mod admin;
mod auth;
mod code;
mod config;
mod submit;

pub use admin::admin_page;
pub use auth::handle_auth;
pub use config::Config;
pub use submit::{handle_submit_with_db, TestCase};

#[tracing::instrument(name = "get_solved", skip(headers, conf))]
pub async fn get_solved(headers: HeaderMap, State(conf): State<Config>) -> Json<Output> {
    use axum::http::header::AUTHORIZATION;
    use libsql::params;
    if !headers.contains_key(AUTHORIZATION) {
        tracing::debug!("No authorization header provided");
        return Json(Output::Unauthorized);
    }
    let token = match headers.get(AUTHORIZATION).and_then(|x| x.to_str().ok()) {
        Some(x) => x,
        None => {
            tracing::error!("Invalid authorization header value");
            return Json(Output::Unauthorized);
        }
    };
    let email = match conf.decrypt(token) {
        Some(email) => email,
        None => {
            tracing::debug!("Failed to decode the authorization header");
            return Json(Output::Unauthorized);
        }
    };
    let mut rows = match conf
        .query(
            "SELECT solved FROM players WHERE email = ?1 LIMIT 1",
            params![email.clone()],
        )
        .await
    {
        Some(rows) => rows,
        None => {
            tracing::error!(
                "Database query error while fetching solved count for the user {}",
                email
            );
            return Json(Output::ServerError);
        }
    };
    let row = match rows.next().await {
        Ok(Some(row)) => row,
        Ok(None) => {
            tracing::debug!("No player found with email {}", email);
            return Json(Output::Unauthorized);
        }
        Err(e) => {
            tracing::error!("Error retrieving row for user {}: {:?}", email, e);
            return Json(Output::ServerError);
        }
    };
    let solved = match row.get::<u64>(0) {
        Ok(val) => val as usize,
        Err(e) => {
            tracing::error!("Error extracting solved count for user {}: {:?}", email, e);
            return Json(Output::ServerError);
        }
    };
    tracing::info!("User {} has solved {} problems", email, solved);
    Json(Output::Solved(solved))
}

#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    Cpp,
    Javascript,
    Python,
    Java,
}
impl Language {
    pub fn is_compiled(&self) -> bool {
        use Language::*;
        matches!(self, Rust | Cpp | Java)
    }
}

impl IntoValue for Language {
    fn into_value(self) -> libsql::Result<libsql::Value> {
        use libsql::Value::*;
        use Language::*;
        match self {
            Cpp => Ok(Text("cpp".to_string())),
            Rust => Ok(Text("rust".to_string())),
            Java => Ok(Text("java".to_string())),
            Python => Ok(Text("python".to_string())),
            Javascript => Ok(Text("javascript".to_string())),
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "status", content = "message")]
pub enum Output {
    ServerError,
    Unauthorized,
    Token(String),
    Solved(usize),
    InvalidProblem(usize),
    CannotCompile(String),
    RuntimeError {
        stdout: String,
        stderr: String,
    },
    Timeout(TestCase),
    WrongAnswer {
        test: TestCase,
        stdout: String,
        stderr: String,
    },
    Hidden,
    Accepted(
        #[serde(with = "serde_millis")] Duration,
        #[serde(with = "serde_millis")] Duration,
    ),
}
