use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

use crate::error::TopsyTurvyError;

#[derive(Deserialize, Serialize, Debug)]
pub struct TestCase {
    pub id: u8,
    pub stdin: String,
    pub expected_output: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TestCases {
    pub test_cases: Vec<TestCase>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TestCaseFile {
    #[serde(flatten)]
    problems: HashMap<String, TestCases>
}

impl TestCaseFile {
    pub fn get(&self, problem_id: &str) -> Result<&TestCases, TopsyTurvyError> {
        let test_cases = self.problems.get(problem_id);
        if let Some(test_cases) = test_cases {
            Ok(test_cases)
        } else {
            Err(TopsyTurvyError::NoSuchTestCase)
        }
    }
}