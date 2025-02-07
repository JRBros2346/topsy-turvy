use crate::Language;
use axum::{body::Body, http::Response, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::cell::LazyCell;
use tokio::time::Duration;

const PROBLEMS: LazyCell<[TestCases; 1]> = LazyCell::new(|| {
    [TestCases {
        public: vec![
            TestCase {
                input: "5\n".to_string(),
                output: "15".to_string(),
            },
            TestCase {
                input: "10\n".to_string(),
                output: "55".to_string(),
            },
            TestCase {
                input: "6\n".to_string(),
                output: "21".to_string(),
            },
        ],
        hidden: TestCase {
            input: "71\n".to_string(),
            output: "2556".to_string(),
        },
    }]
});

#[derive(Serialize, Clone)]
pub struct TestCase {
    pub input: String,
    pub output: String,
}

#[derive(Clone)]
pub struct TestCases {
    pub public: Vec<TestCase>,
    pub hidden: TestCase,
}

#[derive(Deserialize)]
pub struct Submission {
    problem: usize,
    code: String,
    language: Language,
}

#[derive(Serialize)]
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
    Accepted(Duration, Duration),
}

impl IntoResponse for Output {
    fn into_response(self) -> Response<Body> {
        use axum::http::StatusCode;
        match self {
            Output::ServerError => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap(),
            Output::InvalidProblem(problem) => Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(format!("Invalid Problem: {}", problem)))
                .unwrap(),
            Output::CannotCompile(err) => Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(err))
                .unwrap(),
            Output::RuntimeError { stdout, stderr } => Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(format!("Runtime Error\nStdout: {}\nStderr: {}", stdout, stderr)))
                .unwrap(),
            Output::Timeout(test_case) => Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(format!("Timeout: {}", test_case.input)))
                .unwrap(),
            Output::WrongAnswer { test, stdout, stderr } => Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(format!(
                    "Wrong Answer\nInput: {}\nExpected: {}\nGot: {}\nError: {}",
                    test.input, test.output, stdout, stderr
                )))
                .unwrap(),
            Output::Hidden => Response::builder()
                .status(StatusCode::OK)
                .body(Body::empty())
                .unwrap(),
            Output::Accepted(run_time, total_time) => Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(format!(
                    "Accepted\nRun Time: {:?}\nTotal Time: {:?}",
                    run_time, total_time
                )))
                .unwrap(),
        }
    }
}

pub async fn handle_submit(Json(payload): Json<Submission>) -> Output {
    use tempfile::TempDir;
    use crate::code::{compile_code, test_code};
    let problem = match PROBLEMS.get(payload.problem) {
        Some(problem) => problem.clone(),
        None => return Output::InvalidProblem(payload.problem),
    };
    let dir = TempDir::new().unwrap();
    if let Err(err) = compile_code(&payload.code, payload.language, dir.path()).await {
        return err;
    }
    let mut results = vec![];
    for test in problem.public {
        match test_code(payload.language, test, dir.path()).await {
            Ok(dur) => results.push(dur),
            Err(err) => return err,
        }
    }
    match test_code(payload.language, problem.hidden, dir.path()).await {
        Ok(dur) => results.push(dur),
        Err(Output::WrongAnswer { .. }) => return Output::Hidden,
        Err(err) => return err,
    }
    let avg = results.iter().sum::<Duration>() / results.len() as u32;
    let error = match results.iter().map(|&r| r.abs_diff(avg)).max() {
        Some(error) => error,
        None => {
            eprintln!("No results");
            return Output::ServerError;
        }
    };
    Output::Accepted(avg, error)
}
