use std::fmt::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ErrorType {
    ParseError,
    ThirdPartyError,
    MissingConfig,
    DatabaseError,
    InvalidConfig,
    GrpcError,
    CacheError,
    CacheMiss,
    UnknownError
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    pub error_type: ErrorType,
    pub message: String,
}
impl Error {
    pub fn new(error_type: ErrorType, message: String) -> Self {
        Error {
            error_type,
            message,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.error_type, self.message)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.message
    }
}
