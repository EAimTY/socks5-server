mod request;
mod response;

pub use self::{request::Request, response::Response};

pub const SUBNEGOTIATION_VERSION: u8 = 0x01;
