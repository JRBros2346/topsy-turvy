use dotenv::dotenv;
use reqwest::Client;
use serde::Deserialize;
use std::{env, fs::File, io::Read};
use crate::{error::TopsyTurvyError, models::test_case::TestCaseFile};



pub struct AppState {
    pub judge0_client: Client,
    pub config: Config
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            judge0_client: Client::new(),
            config: Config::new()
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub judge0_authn_token: String,
    pub judge0_authz_token: String,
    pub judge0_api_endpoint: String,
    pub environment: String,
    pub test_cases: TestCaseFile
}

impl Config {
    pub fn new() -> Self {
        dotenv().ok();
        let judge0_authn_token = env::var("JUDGE0_AUTHN_TOKEN")
            .map_err(|_| TopsyTurvyError::EnvVarNotFound("JUDGE0_AUTHN_TOKEN".to_string()))
            .unwrap();
        let judge0_authz_token = env::var("JUDGE0_AUTHZ_TOKEN")
            .map_err(|_| TopsyTurvyError::EnvVarNotFound("JUDGE0_AUTHZ_TOKEN".to_string()))
            .unwrap();
        let judge0_api_endpoint = env::var("JUDGE0_API_ENDPOINT")
            .map_err(|_| TopsyTurvyError::EnvVarNotFound("JUDGE0_API_ENDPOINT".to_string()))
            .unwrap();
        let environment = env::var("ENVIRONMENT")
            .map_err(|_| TopsyTurvyError::EnvVarNotFound("ENVIRONMENT".to_string()))
            .unwrap();
        let test_cases_file = env::var("TEST_CASES_FILE")
            .map_err(|_| TopsyTurvyError::EnvVarNotFound("TEST_CASES_FILE".to_string()))
            .unwrap();
        let mut file = File::open(&test_cases_file)
            .map_err(|_| {
                TopsyTurvyError::InternalError("Failed to read test cases file".to_string())
            })
            .unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|_| {
            TopsyTurvyError::InternalError("Error while parsing test cases".to_string())
        }).unwrap();
        let test_cases: TestCaseFile = serde_json::from_str(&contents)
            .map_err(|_| {
                TopsyTurvyError::InternalError("Error while deserializing test cases".to_string())
            })
            .unwrap();
        Config {
            judge0_authn_token,
            judge0_authz_token,
            judge0_api_endpoint,
            environment,
            test_cases,
        }
    }
}
