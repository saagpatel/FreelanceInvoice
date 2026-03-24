use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Timer error: {0}")]
    Timer(String),

    #[error("AI estimation error: {0}")]
    AiEstimation(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Template error: {0}")]
    Template(#[from] handlebars::RenderError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[cfg_attr(test, allow(dead_code))]
    #[error("Security error: {0}")]
    Security(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
