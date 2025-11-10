use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyHunterError {
    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Curl error: {0}")]
    Curl(#[from] curl::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Invalid API key format: {0}")]
    InvalidKeyFormat(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Search provider error: {0}")]
    SearchProvider(String),

    #[error("Detector error: {0}")]
    Detector(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, KeyHunterError>;
