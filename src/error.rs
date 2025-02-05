#[derive(Debug)]
pub enum TopsyTurvyError {
    EnvVarNotFound(String),
    InvalidLanguage(u16),
    CompilationError(String),
    RuntimeError(String),
    TimeLimitExceeded(String),
    WrongAnswer(String),
    InternalError(String),
    EncodingError,
    NoSuchTestCase,
    Judge0RequestError
}