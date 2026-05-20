use thiserror::Error;

use crate::domain::common::AppResponse;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found: {entity} '{identifier}'")]
    NotFound { entity: String, identifier: String },

    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("path: {0}")]
    Path(#[from] std::path::StripPrefixError),

    #[error("serialization: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("lock poisoned")]
    LockPoisoned,

    #[error("{0}")]
    Other(String),
}

impl From<AppError> for String {
    fn from(error: AppError) -> String {
        error.to_string()
    }
}

pub fn not_found<T>(entity: &str, identifier: &str) -> AppResponse<T> {
    AppResponse::failure("not_found", format!("{} not found: {}", entity, identifier))
}
