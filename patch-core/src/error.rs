use thiserror::Error;

pub type PatchResult<T> = Result<T, PatchError>;

#[derive(Debug, Error)]
pub enum PatchError {
    #[error("unsupported client version: {0}")]
    UnsupportedClientVersion(String),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
