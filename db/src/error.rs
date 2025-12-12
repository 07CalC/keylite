use thiserror::Error;

#[derive(Debug, Error)]
pub enum DocError {
    #[error("collection {0} not found")]
    CollectionNotFound(String),
    #[error("document not found")]
    DocumentNotFound(String),
    #[error("index {0} not found")]
    IndexNotFound(String),

    #[error("unique constraint violation with {field} = {value}")]
    UniqueConstraintViolation { field: String, value: String },
    #[error("invalid document id: {0}")]
    InvalidDocumentId(String),

    #[error("required field `{0}` missing")]
    MissingRequiredField(String),
    #[error("invalid value for field {field}: {reason}")]
    InvalidFieldValue { field: String, reason: String },

    #[error("version mismatch: expected {expected} found {actual}")]
    VersionMismatch { expected: u64, actual: u64 },

    #[error("storage error: {0}")]
    StorageError(keylite_kv::error::DbError),

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("deserialization error: {0}")]
    DeserializationError(String),

    #[error("invalid UTF-8 in data: {0}")]
    InvalidUtf8(String),
}

pub type Result<T> = std::result::Result<T, DocError>;

impl From<keylite_kv::error::DbError> for DocError {
    fn from(err: keylite_kv::error::DbError) -> Self {
        DocError::StorageError(err)
    }
}

impl From<rmp_serde::encode::Error> for DocError {
    fn from(err: rmp_serde::encode::Error) -> Self {
        DocError::SerializationError(err.to_string())
    }
}

impl From<rmp_serde::decode::Error> for DocError {
    fn from(err: rmp_serde::decode::Error) -> Self {
        DocError::DeserializationError(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for DocError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        DocError::InvalidUtf8(err.to_string())
    }
}
