use std::io::{Error as IoError, ErrorKind};
use thiserror::Error;

/// Errors may occured during SOCKS5 password authentication
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] IoError),

    #[error("Unsupported sub-negotiation version {version:#04x}")]
    SubNegotiationVersion { version: u8 },

    #[error("Unsupported sub-negotiation status {status:#04x}")]
    SubNegotiationStatus { version: u8, status: u8 },
}

impl From<Error> for IoError {
    fn from(err: Error) -> Self {
        match err {
            Error::Io(err) => err,
            err => IoError::new(ErrorKind::Other, err),
        }
    }
}
