use async_trait::async_trait;
use socks5_proto::{
    handshake::password::{Request as PasswordRequest, Response as PasswordResponse},
    HandshakeMethod,
};
use std::io::{Error, ErrorKind, Result};
use tokio::net::TcpStream;

/// This trait is for defining the socks5 authentication method.
///
/// Pre-defined authentication methods can be found in the [`auth`](https://docs.rs/socks5-server/latest/socks5_server/auth/index.html) module.
///
/// You can create your own authentication method by implementing this trait. Since GAT is not stabled yet, [async_trait](https://docs.rs/async-trait/latest/async_trait/index.html) needs to be used.
///
/// # Example
/// ```rust
/// use async_trait::async_trait;
/// use std::io::Result;
/// use socks5_proto::HandshakeMethod;
/// use socks5_server::Auth;
/// use tokio::net::TcpStream;
///
/// pub struct MyAuth;
///
/// #[async_trait]
/// impl Auth for MyAuth {
///     fn as_handshake_method(&self) -> HandshakeMethod {
///         HandshakeMethod(0x80)
///     }
///
///     async fn execute(&self, stream: &mut TcpStream) -> Result<()> {
///         // do something
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait Auth {
    fn as_handshake_method(&self) -> HandshakeMethod;
    async fn execute(&self, stream: &mut TcpStream) -> Result<()>;
}

/// No authentication as the socks5 handshake method.
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

/// Username and password as the socks5 handshake method.
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
