//! This module defines trait [`Auth`] and some pre-defined authentication adaptors.
//!
//! The process of SOCKS5 authentication can be customized by implementing [`Auth`] trait on your own types.

use async_trait::async_trait;
use socks5_proto::handshake::{
    password::{Error as PasswordError, Request as PasswordRequest, Response as PasswordResponse},
    Method,
};
use tokio::net::TcpStream;

/// This trait is for defining the customized process of SOCKS5 authentication.
///
/// You can create your own authentication method by implementing this trait. Associate type `Output` indicates the result of authenticating. Note that this library will not implicitly close any connection even if the authentication failed.
///
/// # Example
/// ```rust
/// use async_trait::async_trait;
/// use std::io::Result;
/// use socks5_proto::handshake::Method;
/// use socks5_server::Auth;
/// use tokio::net::TcpStream;
///
/// pub struct MyAuth;
///
/// #[async_trait]
/// impl Auth for MyAuth {
///     type Output = Result<usize>;
///
///     fn as_handshake_method(&self) -> Method {
///         Method(0xfe)
///     }
///
///     async fn execute(&self, stream: &mut TcpStream) -> Self::Output {
///         // do something on stream
///         Ok(1145141919810)
///     }
/// }
/// ```
#[async_trait]
pub trait Auth {
    type Output;

    fn as_handshake_method(&self) -> Method;
    async fn execute(&self, stream: &mut TcpStream) -> Self::Output;
}

/// Not authenticate at all.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoAuth;

impl NoAuth {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Auth for NoAuth {
    type Output = Result<bool, PasswordError>;

    fn as_handshake_method(&self) -> Method {
        Method::NONE
    }

    async fn execute(&self, _: &mut TcpStream) -> Self::Output {
        Ok(true)
    }
}

/// Using username and password to authenticate.
///
/// The boolean value in associate type `Auth::Output` indicates whether the authentication is successful.
#[derive(Clone, Debug)]
pub struct Password {
    pub username: Vec<u8>,
    pub password: Vec<u8>,
}

impl Password {
    /// Create a new `Password` authentication adaptor.
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

    async fn execute(&self, stream: &mut TcpStream) -> Self::Output {
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
