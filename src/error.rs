use std::{
    error,
    fmt::{self, Debug, Display},
    result,
};

/// Alias for a `Result` with our own error type.
pub type Result<T> = result::Result<T, Error>;

/// Custom error type for what went wrong
#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl error::Error for Error {}

impl From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Self {
        Self {
            message: e.to_string(),
        }
    }
}

impl From<hyper::header::InvalidHeaderName> for Error {
    fn from(e: hyper::http::header::InvalidHeaderName) -> Self {
        Self {
            message: e.to_string(),
        }
    }
}

impl From<hyper::header::InvalidHeaderValue> for Error {
    fn from(e: hyper::http::header::InvalidHeaderValue) -> Self {
        Self {
            message: e.to_string(),
        }
    }
}

impl From<hyper::http::uri::InvalidUri> for Error {
    fn from(e: hyper::http::uri::InvalidUri) -> Self {
        Self {
            message: e.to_string(),
        }
    }
}

impl From<hyper::header::ToStrError> for Error {
    fn from(e: hyper::header::ToStrError) -> Self {
        Self {
            message: e.to_string(),
        }
    }
}
