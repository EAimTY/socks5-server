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

pub struct None;

#[async_trait]
impl Auth for None {
    fn as_handshake_method(&self) -> HandshakeMethod {
        HandshakeMethod::None
    }

    async fn execute(&self, _stream: &mut TcpStream) -> Result<()> {
        Ok(())
    }
}

pub struct Password {
    username: Vec<u8>,
    password: Vec<u8>,
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
