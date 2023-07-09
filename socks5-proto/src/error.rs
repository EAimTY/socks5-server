use crate::{Command, Reply};
use std::io::{Error as IoError, ErrorKind};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Unsupported SOCKS version {version:#x}")]
    ProtocolVersion { version: u8 },

    #[error("Unsupported command {command:#x}")]
    InvalidCommand { version: u8, command: u8 },

    #[error("Unsupported reply {reply:#x}")]
    InvalidReply { version: u8, reply: u8 },

    #[error("Unsupported address type in request {address_type:#x}")]
    InvalidAddressTypeInRequest {
        version: u8,
        command: Command,
        address_type: u8,
    },

    #[error("Unsupported address type in response {address_type:#x}")]
    InvalidAddressTypeInResponse {
        version: u8,
        reply: Reply,
        address_type: u8,
    },

    #[error("Unsupported address type in UDP Header {address_type:#x}")]
    InvalidAddressTypeInUdpHeader { frag: u8, address_type: u8 },
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
    #[error(transparent)]
    Io(#[from] IoError),
}

impl From<Error> for IoError {
    fn from(err: Error) -> Self {
        match err {
            Error::Io(err) => err,
            err => IoError::new(ErrorKind::Other, err),
        }
    }
}
