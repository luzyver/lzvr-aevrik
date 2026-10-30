#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Validation(String),
    #[error("Metadata storage: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Invalid stored metadata: {0}")]
    Json(#[from] serde_json::Error),
    #[error("File or process operation: {0}")]
    Io(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
pub(crate) fn invalid(message: impl Into<String>) -> Error {
    Error::Validation(message.into())
}
