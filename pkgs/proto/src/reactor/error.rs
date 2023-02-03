use redis::RedisError;
use std::fmt::Debug;
use std::num::ParseFloatError;
use std::num::ParseIntError;
use std::str::Utf8Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error(transparent)]
    Utf8(#[from] Utf8Error),
    #[error(transparent)]
    ParseFloat(#[from] ParseFloatError),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Redis(#[from] RedisError),
}

#[derive(Debug, Error)]
pub enum TaskError<T: Send + Debug> {
    /// Bypass error handling and delete stream entry via XDEL
    #[error("Stream entry marked for deletion")]
    Delete,
    /// Skip stream entry acknowledgement
    #[error("Stream entry acknowledgment skipped")]
    SkipAcknowledgement,
    /// Pass through of original error
    #[error(transparent)]
    Error(#[from] T),
}

impl From<ParseError> for RedisError {
    fn from(err: ParseError) -> Self {
        use redis::ErrorKind::TypeError;
        use ParseError::*;
        let (err_msg, err_str) = match err {
            Utf8(err) => ("Invalid utf8 value", err.to_string()),
            ParseFloat(err) => ("Invalid float value", err.to_string()),
            ParseInt(err) => ("Invalid integer value", err.to_string()),
            Serde(err) => ("Invalid JSON value", err.to_string()),
            Redis(err) => return err,
        };

        RedisError::from((TypeError, err_msg, err_str))
    }
}
