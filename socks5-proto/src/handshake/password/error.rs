use std::io::Error as IoError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] IoError),

    #[error("Unsupported sub-negotiation version {version:#x}")]
    SubNegotiationVersion { version: u8 },

    #[error("Unsupported sub-negotiation status {status:#x}")]
    SubNegotiationStatus { version: u8, status: u8 },
}
