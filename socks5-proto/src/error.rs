use std::io::Error as IoError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] IoError),
    #[error("Unsupported socks5 version {0:#x}")]
    UnsupportedSocks5Version(u8),
    #[error("Unsupported sub-negotiation version {0:#x}")]
    UnsupportedSubnegotiationVersion(u8),
    #[error("Unsupported command {0:#x}")]
    UnsupportedCommand(u8),
    #[error("Unsupported address type {0:#x}")]
    UnsupportedAddressType(u8),
    #[error("Address domain name must be in UTF-8")]
    AddressInvalidEncoding,
    #[error("Invalid reply {0:#x}")]
    InvalidReply(u8),
    #[error("Invalid sub-negotiation status {0:#x}")]
    InvalidSubnegotiationStatus(u8),
}
