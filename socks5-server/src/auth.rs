use async_trait::async_trait;
use socks5_proto::{
    handshake::password::{Request as PasswordRequest, Response as PasswordResponse},
    HandshakeMethod,
};
use std::io::{Error, ErrorKind, Result};
use tokio::net::TcpStream;

#[async_trait]
pub trait Auth {
    fn as_handshake_method(&self) -> HandshakeMethod;
    async fn execute(&self, stream: &mut TcpStream) -> Result<()>;
}

pub struct NoAuth;

impl NoAuth {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Auth for NoAuth {
    fn as_handshake_method(&self) -> HandshakeMethod {
        HandshakeMethod::None
    }

    async fn execute(&self, _: &mut TcpStream) -> Result<()> {
        Ok(())
    }
}

impl Default for NoAuth {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Password {
    username: Vec<u8>,
    password: Vec<u8>,
}

impl Password {
    pub fn new(username: Vec<u8>, password: Vec<u8>) -> Self {
        Self { username, password }
    }
}

#[async_trait]
impl Auth for Password {
    fn as_handshake_method(&self) -> HandshakeMethod {
        HandshakeMethod::Password
    }

    async fn execute(&self, stream: &mut TcpStream) -> Result<()> {
        let req = PasswordRequest::read_from(stream).await?;

        if (&req.username, &req.password) == (&self.username, &self.password) {
            let resp = PasswordResponse::new(true);
            resp.write_to(stream).await?;
            Ok(())
        } else {
            let resp = PasswordResponse::new(false);
            resp.write_to(stream).await?;
            Err(Error::new(
                ErrorKind::InvalidData,
                "SOCKS5 username / password authentication failed",
            ))
        }
    }
}
