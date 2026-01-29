use paperclip_parser::ParseError;
use thiserror::Error;

/// Common error type that can hold any paperclip error
#[derive(Error, Debug)]
pub enum CommonError {
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Generic error: {0}")]
    Generic(String),
}

impl From<String> for CommonError {
    fn from(s: String) -> Self {
        CommonError::Generic(s)
    }
}

impl From<&str> for CommonError {
    fn from(s: &str) -> Self {
        CommonError::Generic(s.to_string())
    }
}
