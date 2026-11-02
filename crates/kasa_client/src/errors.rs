use thiserror::Error;

#[derive(Debug, Error, uniffi::Error)]
pub enum ClientError {
    #[error("the db is either not initialized or errored")]
    InvalidDb,
    #[error("{0}")]
    Anyhow(String),
}

pub type ClientResult<T> = Result<T, ClientError>;

impl From<anyhow::Error> for ClientError {
    fn from(e: anyhow::Error) -> Self {
        ClientError::Anyhow(format!("{e:#}"))
    }
}

impl From<sqlx::Error> for ClientError {
    fn from(e: sqlx::Error) -> Self {
        Self::from(anyhow::Error::from(e))
    }
}

impl From<std::io::Error> for ClientError {
    fn from(e: std::io::Error) -> Self {
        Self::from(anyhow::Error::from(e))
    }
}
