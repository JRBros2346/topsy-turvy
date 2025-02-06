use axum::{body::Body, http::Response, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::cell::LazyCell;
use tokio::time::Duration;

const PROBLEMS: LazyCell<[TestCases; 1]> = LazyCell::new(|| {
    [TestCases {
        public: vec![
            TestCase {
                input: "4\n2 7 11 15\n9\n".to_string(),
                output: "0 1".to_string(),
            },
            TestCase {
                input: "3\n3 2 4\n6\n".to_string(),
                output: "1 2".to_string(),
            },
            TestCase {
                input: "2\n3 3\n6\n".to_string(),
                output: "0 1".to_string(),
            },
        ],
        hidden: TestCase {
            input: "3\n3 2 3\n6\n".to_string(),
            output: "0 2".to_string(),
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
#[serde(rename_all = "lowercase")]
enum Language {
    Rust,
    Cpp,
    Javascript,
    Python,
    Java,
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
    RuntimeError(String),
    Timeout(TestCase),
    WrongAnswer(TestCase, String),
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
            Output::RuntimeError(err) => Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(err))
                .unwrap(),
            Output::Timeout(test_case) => Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(format!("Timeout: {}", test_case.input)))
                .unwrap(),
            Output::WrongAnswer(test_case, output) => Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(format!(
                    "Wrong Answer: {}\nExpected: {}",
                    output, test_case.output
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

pub async fn submit(Json(payload): Json<Submission>) -> Output {
    let problem = match PROBLEMS.get(payload.problem) {
        Some(problem) => problem.clone(),
        None => return Output::InvalidProblem(payload.problem),
    };
    match payload.language {
        Language::Rust => {
            use crate::code::rust::rust_run;
            rust_run(&payload.code, problem).await
        }
        Language::Cpp => {
            use crate::code::cpp::cpp_run;
            cpp_run(&payload.code, problem).await
        }
        Language::Javascript => Output::ServerError,
        Language::Python => Output::ServerError,
        Language::Java => Output::ServerError,
    }
}
