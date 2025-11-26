use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("sst: {0}")]
    SST(#[from] crate::sst::SSTError),
    #[error("other: {0}")]
    Other(String),
}

pub type DbResult<T> = std::result::Result<T, DbError>;
