use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("Invalid transation type {0}")]
    InvalidTransactionType(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Csv(#[from] csv::Error),
}
