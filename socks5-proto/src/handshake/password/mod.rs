//! This module contains the implementation of password authentication method of SOCKS5 protocol handshake.

mod error;
mod request;
mod response;

pub use self::{error::Error, request::Request, response::Response};

pub const SUBNEGOTIATION_VERSION: u8 = 0x01;
