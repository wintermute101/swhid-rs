use thiserror::Error;

/// Errors that may occur while parsing SWHIDs or computing hashes.
#[derive(Debug, Error)]
pub enum SwhidError {
    #[error("invalid SWHID format: {0}")]
    InvalidFormat(String),

    #[error("invalid URI scheme (expected `swh`): {0}")]
    InvalidScheme(String),

    #[error("unsupported SWHID version: {0}")]
    InvalidVersion(String),

    #[error("invalid object type: {0}")]
    InvalidObjectType(String),

    #[error("invalid digest (expected 40 hex chars): {0}")]
    InvalidDigest(String),

    #[error("invalid qualifier key: {0}")]
    InvalidQualifierKey(String),

    #[error("invalid qualifier value for `{key}`: {value}")]
    InvalidQualifierValue { key: String, value: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
