use thiserror::Error;

#[derive(Error, Debug)]
pub enum CanaryError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("DTC code not found: {0}")]
    DtcNotFound(String),

    #[error("Invalid DTC format")]
    InvalidDtcFormat,

    #[error("Procedure not found: {0}")]
    ProcedureNotFound(String),

    #[error("Unsupported protocol: {0}")]
    UnsupportedProtocol(String),

    #[error("Protocol decode error: {0}")]
    ProtocolError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),

    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, CanaryError>;
