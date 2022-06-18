#![doc = include_str!("../README.md")]

use std::{io::Result, net::SocketAddr, sync::Arc};
use tokio::net::{TcpListener, ToSocketAddrs};

pub mod auth;
pub mod connection;

pub use crate::{
    auth::Auth,
    connection::{
        associate::{Associate, AssociatedUdpSocket},
        bind::Bind,
        connect::Connect,
        Connection, IncomingConnection,
    },
};

/// The socks5 server itself.
///
/// The server can be constructed on a given socket address, or be created on a existing TcpListener.
///
/// The authentication method can be configured with the [`Auth`](https://docs.rs/socks5-server/latest/socks5_server/auth/trait.Auth.html) trait.
pub struct Server {
    listener: TcpListener,
    auth: Arc<dyn Auth + Send + Sync>,
}

impl Server {
    /// Create a new socks5 server with the given TCP listener and authentication method.
    #[inline]
    pub fn new(listener: TcpListener, auth: Arc<dyn Auth + Send + Sync>) -> Self {
        Self { listener, auth }
    }

    /// Create a new socks5 server on the given socket address and authentication method.
    #[inline]
    pub async fn bind<T: ToSocketAddrs>(
        addr: T,
        auth: Arc<dyn Auth + Send + Sync>,
    ) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Self::new(listener, auth))
    }

    /// Accept an [`IncomingConnection`](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.IncomingConnection.html). The connection may not be a valid socks5 connection. You need to call [`IncomingConnection::handshake()`](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.IncomingConnection.html#method.handshake) to hand-shake it into a proper socks5 connection.
    #[inline]
    pub async fn accept(&self) -> Result<(IncomingConnection, SocketAddr)> {
        let (stream, addr) = self.listener.accept().await?;
        Ok((IncomingConnection::new(stream, self.auth.clone()), addr))
    }

    /// Get the the local socket address binded to this server
    #[inline]
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.listener.local_addr()
    }
}
