use std::io;

use ::futures::future;
use serde::Deserialize;

use crate::{languages::Language, testcases::TestCase};

mod languages;
mod testcases;

pub async fn submit(submission: Submission, test_cases: Vec<TestCase>) -> Result<Output, Output> {
    let (dir, compile_error) = submission.lang.compile(&submission.program).await?;

    let results = future::join_all(test_cases.into_iter().map(|tc| {
        let dir = dir.path().to_path_buf();
        let lang = submission.lang.clone();
        let compiler = compile_error.clone();
        tokio::spawn(async move { tc.sandbox(&dir, &lang, compiler).await })
    }))
    .await;

    for res in results {
        match res {
            Ok(Err(e)) => return Err(e),
            Ok(Ok(_)) => continue,
            Err(e) => return Err(Output::IO(io::Error::new(io::ErrorKind::Other, e))),
        }
    }
    Ok(Output::Success)
}

#[derive(Deserialize)]
pub struct Submission {
    pub problem: usize,
    pub lang: Language,
    pub program: String,
}

#[derive(Debug)]
pub enum Output {
    IO(io::Error),
    CompileError(String),
    RuntimeError {
        compiler: String,
        input: String,
        output: String,
        error: String,
    },
    TestCaseFailed {
        compiler: String,
        input: String,
        error: String,
        expected: String,
        actual: String,
    },
    HiddenTestCaseFailed(String),
    Success,
}
impl Output {
    pub fn is_success(&self) -> bool {
        matches!(self, Output::Success)
    }
}
impl From<io::Error> for Output {
    fn from(err: io::Error) -> Self {
        Output::IO(err)
    }
}
