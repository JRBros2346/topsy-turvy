use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize)]
pub struct SubmissionRequest {
    pub source_code: String,
    pub language_id: u16,
    pub problem_id: String,
}


#[derive(Serialize, Deserialize)]
pub struct SubmissionResponse {
    pub submission_tokens: Vec<SubmissionToken>
}

impl SubmissionResponse {
    pub fn new(submission_tokens: Vec<SubmissionToken>) -> Self {
        SubmissionResponse {
            submission_tokens
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SubmissionToken {
    token: String
}

#[derive(Serialize, Deserialize)]
pub struct Judge0Submissions {
    pub submissions: Vec<Judge0Submission>
}

impl Judge0Submissions {
    pub fn new() -> Self {
        Judge0Submissions {
            submissions: vec![]
        }
    }
    pub fn insert(&mut self, submission: Judge0Submission) {
        &self.submissions.push(submission);
    }
}

#[derive(Serialize, Deserialize)]
pub struct Judge0Submission {
    pub source_code: String,
    pub language_id: u16,
    pub stdin: String,
    pub expected_output: String
}