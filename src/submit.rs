use crate::{Config, Language, Output};
use axum::{
    extract::{Json, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TestCase {
    pub input: String,
    pub output: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TestCases {
    pub public: Vec<TestCase>,
    pub hidden: TestCase,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Submission {
    code: String,
    language: Language,
}

#[tracing::instrument(name = "handle_submit_with_db", skip(headers, conf, payload))]
pub async fn handle_submit_with_db(
    headers: HeaderMap,
    State(conf): State<Config>,
    Json(payload): Json<Submission>,
) -> Result<Json<Output>, Json<Output>> {
    use axum::http::header::AUTHORIZATION;
    if headers.contains_key(AUTHORIZATION) {
        use libsql::params;
        let user = conf
            .decrypt(
                headers[AUTHORIZATION]
                    .to_str()
                    .inspect_err(|e| tracing::error!("{e} {} {}", file!(), line!()))
                    .map_err(|_| Json(Output::ServerError))?,
            )
            .ok_or(Output::Unauthorized)?;
        let mut rows = conf
            .query(
                "SELECT solved FROM players WHERE email = ?1 LIMIT 1",
                params![user.clone()],
            )
            .await
            .ok_or(Output::ServerError)?;
        let row = rows
            .next()
            .await
            .inspect_err(|e| tracing::error!("{e} {} {}", file!(), line!()))
            .map_err(|_| Output::ServerError)?;
        if let Some(row) = row {
            let problem = row
                .get::<u64>(0)
                .inspect_err(|e| tracing::error!("{e} {} {}", file!(), line!()))
                .map_err(|_| Output::ServerError)? as usize;
            if conf.completed(problem) {
                return Ok(Json(Output::Completed));
            }
            tracing::debug!("Player is on problem index: {}", problem);
            let output =
                handle_submit(conf.problems(problem), &payload.code, payload.language).await?;
            if let Output::Accepted(..) = output {
                use chrono::Utc;
                tracing::info!(
                    "Submission accepted for user: {} on problem {}",
                    user,
                    problem
                );
                let transaction = conf.transaction().await.ok_or(Output::ServerError)?;
                transaction
                    .execute(
                        "INSERT INTO submissions VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![
                            user.clone(),
                            problem as u64,
                            payload.language,
                            payload.code,
                            Utc::now().to_rfc3339()
                        ],
                    )
                    .await
                    .inspect_err(|e| tracing::error!("{e} {} {}", file!(), line!()))
                    .map_err(|_| Output::ServerError)?;
                transaction
                    .execute(
                        "UPDATE players SET solved = ?1 WHERE email = ?2",
                        params![problem as u64 + 1, user],
                    )
                    .await
                    .inspect_err(|e| tracing::error!("{e} {} {}", file!(), line!()))
                    .map_err(|_| Output::ServerError)?;
                transaction
                    .commit()
                    .await
                    .inspect_err(|e| tracing::error!("{e} {} {}", file!(), line!()))
                    .map_err(|_| Output::ServerError)?;
            }
            return Ok(output.into());
        }
    }
    tracing::info!("Authorization header missing or invalid");
    Ok(Json(Output::Unauthorized))
}

#[tracing::instrument(name = "handle_submit", skip(code))]
async fn handle_submit(
    tests: &TestCases,
    code: &str,
    language: Language,
) -> Result<Output, Output> {
    use crate::code::{compile_code, test_code};
    use tempfile::TempDir;
    let dir = TempDir::new().unwrap();
    tracing::info!("Compiling code for language: {:?}", language);
    compile_code(&code, language, dir.path()).await?;
    let mut results = vec![];
    for test in &tests.public {
        tracing::info!("Testing public test case with input: {}", test.input);
        results.push(test_code(language, test.clone(), dir.path()).await?);
    }
    tracing::info!("Testing hidden test case");
    results.push(
        test_code(language, tests.hidden.clone(), dir.path())
            .await
            .map_err(|e| {
                if let Output::WrongAnswer { .. } = e {
                    Output::Hidden
                } else {
                    e
                }
            })?,
    );
    let avg = results.iter().sum::<Duration>() / results.len() as u32;
    let error = results
        .iter()
        .map(|&r| r.abs_diff(avg))
        .max()
        .ok_or(Output::ServerError)
        .inspect_err(|_| tracing::error!("No results to compute error margin"))?;
    tracing::info!(
        "Computed average time: {:?}, error margin: {:?}",
        avg,
        error
    );
    Ok(Output::Accepted(avg, error))
}
