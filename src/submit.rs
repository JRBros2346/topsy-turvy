use crate::Output;
use crate::Language;
use axum::Json;
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


pub async fn handle_submit(Json(payload): Json<Submission>) -> Json<Output> {
    use tempfile::TempDir;
    use crate::code::{compile_code, test_code};
    let problem = match PROBLEMS.get(payload.problem) {
        Some(problem) => problem.clone(),
        None => return Json(Output::InvalidProblem(payload.problem)),
    };
    let dir = TempDir::new().unwrap();
    if let Err(err) = compile_code(&payload.code, payload.language, dir.path()).await {
        return Json(err);
    }
    let mut results = vec![];
    for test in problem.public {
        match test_code(payload.language, test, dir.path()).await {
            Ok(dur) => results.push(dur),
            Err(err) => return Json(err),
        }
    }
    match test_code(payload.language, problem.hidden, dir.path()).await {
        Ok(dur) => results.push(dur),
        Err(Output::WrongAnswer { .. }) => return Json(Output::Hidden),
        Err(err) => return Json(err),
    }
    let avg = results.iter().sum::<Duration>() / results.len() as u32;
    let error = match results.iter().map(|&r| r.abs_diff(avg)).max() {
        Some(error) => error,
        None => {
            eprintln!("No results");
            return Json(Output::ServerError);
        }
    };
    Json(Output::Accepted(avg, error))
}
