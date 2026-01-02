use std::fmt;

pub mod bundle;
pub mod check;
pub mod compile;
pub mod config;
pub mod error;
pub mod format;
pub mod helper;
pub mod lint;
pub mod run;

#[derive(Debug)]
pub enum CliError {
    Io(std::io::Error),
    Json(serde_json::Error),
    TestExecution(String),
    Config(String),
    InvalidPath(String),
    Timeout(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Io(err) => write!(f, "I/O error: {err}"),
            CliError::Json(err) => write!(f, "JSON error: {err}"),
            CliError::TestExecution(msg) => write!(f, "Test execution error: {msg}"),
            CliError::Config(msg) => write!(f, "Configuration error: {msg}"),
            CliError::InvalidPath(msg) => write!(f, "Invalid path: {msg}"),
            CliError::Timeout(msg) => write!(f, "Timeout: {msg}"),
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CliError::Io(err) => Some(err),
            CliError::Json(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        CliError::Io(err)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> Self {
        CliError::Json(err)
    }
}

pub type CliResult<T> = Result<T, CliError>;
