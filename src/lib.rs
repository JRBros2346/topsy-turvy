use std::time::Duration;

use serde::{Deserialize, Serialize};

mod admin;
mod auth;
mod code;
mod config;
mod submit;

pub use admin::admin_page;
pub use auth::handle_auth;
pub use config::Config;
pub use submit::{handle_submit, TestCase};

#[derive(Deserialize, Clone, Copy)]
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

#[derive(Serialize)]
#[serde(tag = "status", content = "message")]
pub enum Output {
    ServerError,
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
