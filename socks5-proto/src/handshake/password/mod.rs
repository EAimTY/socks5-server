mod error;
mod request;
mod response;

pub use self::{error::Error, request::Request, response::Response};

pub const SUBNEGOTIATION_VERSION: u8 = 0x01;
