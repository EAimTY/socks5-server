use async_trait::async_trait;
use socks5_proto::handshake::{
    password::{Error as PasswordError, Request as PasswordRequest, Response as PasswordResponse},
    Method,
};
use tokio::net::TcpStream;

/// This trait is for defining the socks5 authentication method.
///
/// Pre-defined authentication methods can be found in the [`auth`](https://docs.rs/socks5-server/latest/socks5_server/auth/index.html) module.
///
/// You can create your own authentication method by implementing this trait. Note that this library will not implicitly close any connection if authentication failed. You should close the connection in `execute()` / on the `TcpStream` attached to the error returned by `Authenticating::auth()`.
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
///     type Output = usize;
///
///     fn as_handshake_method(&self) -> HandshakeMethod {
///         HandshakeMethod(0x80)
///     }
///
///     async fn execute(&self, stream: &mut TcpStream) -> Result<usize> {
///         // do something
///         Ok(123)
///     }
/// }
/// ```
#[async_trait]
pub trait Auth {
    type Output;

    fn as_handshake_method(&self) -> Method;
    async fn execute(&self, stream: &mut TcpStream) -> Self::Output;
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
    type Output = ();

    fn as_handshake_method(&self) -> Method {
        Method::NONE
    }

    async fn execute(&self, _: &mut TcpStream) {}
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
    type Output = Result<bool, PasswordError>;

    fn as_handshake_method(&self) -> Method {
        Method::PASSWORD
    }

    async fn execute(&self, stream: &mut TcpStream) -> Result<bool, PasswordError> {
        let req = PasswordRequest::read_from(stream).await?;

        if (&req.username, &req.password) == (&self.username, &self.password) {
            let resp = PasswordResponse::new(true);
            resp.write_to(stream).await?;
            Ok(true)
        } else {
            let resp = PasswordResponse::new(false);
            resp.write_to(stream).await?;
            Ok(false)
        }
    }
}
