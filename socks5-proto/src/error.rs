use crate::{handshake::Method, Command, Reply};
use std::io::{Error as IoError, ErrorKind};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Unsupported SOCKS version {version:#04x}")]
    ProtocolVersion { version: u8 },

    #[error("No acceptable handshake method")]
    NoAcceptableHandshakeMethod {
        version: u8,
        chosen_method: Method,
        methods: Vec<Method>,
    },

    #[error("Unsupported command {command:#04x}")]
    InvalidCommand { version: u8, command: u8 },

    #[error("Unsupported reply {reply:#04x}")]
    InvalidReply { version: u8, reply: u8 },

    #[error("Unsupported address type in request {address_type:#04x}")]
    InvalidAddressTypeInRequest {
        version: u8,
        command: Command,
        address_type: u8,
    },

    #[error("Unsupported address type in response {address_type:#04x}")]
    InvalidAddressTypeInResponse {
        version: u8,
        reply: Reply,
        address_type: u8,
    },

    #[error("Unsupported address type in UDP Header {address_type:#04x}")]
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
