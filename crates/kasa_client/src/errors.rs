use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("the db is either not initialized or errored")]
    InvalidDb,
}
