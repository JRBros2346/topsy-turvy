use std::sync::Arc;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;

use axum::{extract::State, Json};

use crate::error::TopsyTurvyError;
use crate::models::submission::SubmissionResponse;
use crate::models::submission::SubmissionToken;
use crate::{config::AppState, models::submission::{Judge0Submission, Judge0Submissions, SubmissionRequest}};


pub async fn submit(
    State(state): State<Arc<AppState>>,
    Json(submission_payload): Json<SubmissionRequest>) -> Result<Json<SubmissionResponse>, TopsyTurvyError> {
        let test_cases = &state.config.test_cases.get(&submission_payload.problem_id);
        let mut judge0_submission_body: Judge0Submissions = Judge0Submissions::new();
        if let Ok(test_cases) = test_cases {
            for test_case in &test_cases.test_cases {
                let submission = Judge0Submission {
                    source_code: submission_payload.source_code.clone(),
                    language_id: submission_payload.language_id,
                    stdin: test_case.stdin.clone(),
                    expected_output: test_case.expected_output.clone()
                };
                judge0_submission_body.insert(submission);
            }    
            let mut headers = HeaderMap::new();
            headers.insert("X-Auth-Token", HeaderValue::from_str(&state.config.judge0_authn_token).unwrap());
            let response = state.judge0_client.clone().post(format!("{}/submissions/batch?base64_encoded=true", &state.config.judge0_api_endpoint)).headers(headers).json::<Judge0Submissions>(&judge0_submission_body).send().await.map_err(|_| TopsyTurvyError::Judge0RequestError)?;
            let submission_tokens = response.json::<Vec<SubmissionToken>>().await.map_err(|_| TopsyTurvyError::Judge0RequestError)?;
            Ok(Json(SubmissionResponse {submission_tokens: submission_tokens.clone().to_vec() }))
        } else {
            return Err(TopsyTurvyError::InternalError("Failed to parse test cases".to_string()))
        }
}