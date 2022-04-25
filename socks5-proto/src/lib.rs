mod address;
mod command;
mod error;
mod reply;
mod request;
mod response;
mod udp;

pub mod handshake;

pub use self::{
    address::Address,
    command::Command,
    error::Error,
    handshake::{HandshakeMethod, HandshakeRequest, HandshakeResponse},
    reply::Reply,
    request::Request,
    response::Response,
    udp::UdpHeader,
};

pub const SOCKS_VERSION: u8 = 0x05;
