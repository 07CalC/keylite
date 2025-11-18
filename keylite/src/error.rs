use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("sst: {0}")]
    SST(#[from] crate::sst::SSTError),
}

pub type DbResult<T> = std::result::Result<T, DbError>;
