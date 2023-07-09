#![doc = include_str!("../README.md")]

use std::{
    io::Error,
    net::SocketAddr,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::net::TcpListener;

pub mod auth;
pub mod connection;

pub use crate::{
    auth::Auth,
    connection::{
        associate::{Associate, AssociatedUdpSocket},
        bind::Bind,
        connect::Connect,
        Authenticated, Command, IncomingConnection,
    },
};

pub(crate) type AuthAdaptor<O> = Arc<dyn Auth<Output = O> + Send + Sync>;

/// A socks5 server listener
///
/// This server listens on a socket and treats incoming connections as socks5 connections.
///
/// A `(TcpListener, Arc<dyn Auth<Output = O> + Send + Sync>)` can be converted into a `Server<O>` with `From` trait. Also, a `Server<O>` can be converted back.
///
/// Generic type `<O>` is the output type of the authentication adapter. See module [`auth`](https://docs.rs/socks5-server/latest/socks5_server/auth/index.html).
pub struct Server<O> {
    listener: TcpListener,
    auth: AuthAdaptor<O>,
}

impl<O> Server<O> {
    /// Accept an [`IncomingConnection<O>`](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.IncomingConnection.html).
    ///
    /// The connection is only a freshly created TCP connection and may not be a valid socks5 connection. You should call [`IncomingConnection::authenticate()`](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.IncomingConnection.html#method.authenticate) to perform a socks5 authentication handshake.
    #[inline]
    pub async fn accept(&self) -> Result<(IncomingConnection<O>, SocketAddr), Error> {
        let (stream, addr) = self.listener.accept().await?;
        Ok((IncomingConnection::new(stream, self.auth.clone()), addr))
    }

    /// Polls to accept an [`IncomingConnection<O>`](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.IncomingConnection.html).
    ///
    /// The connection is only a freshly created TCP connection and may not be a valid socks5 connection. You should call [`IncomingConnection::authenticate()`](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.IncomingConnection.html#method.authenticate) to perform a socks5 authentication handshake.
    ///
    /// If there is no connection to accept, Poll::Pending is returned and the current task will be notified by a waker. Note that on multiple calls to poll_accept, only the Waker from the Context passed to the most recent call is scheduled to receive a wakeup.
    #[inline]
    pub fn poll_accept(
        &self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(IncomingConnection<O>, SocketAddr), Error>> {
        self.listener
            .poll_accept(cx)
            .map_ok(|(stream, addr)| (IncomingConnection::new(stream, self.auth.clone()), addr))
    }

    /// Returns the local address that this server is bound to.
    ///
    /// This can be useful, for example, when binding to port 0 to figure out which port was actually bound.
    #[inline]
    pub fn local_addr(&self) -> Result<SocketAddr, Error> {
        self.listener.local_addr()
    }

    /// Sets the value for the `IP_TTL` option on this socket.
    ///
    /// This value sets the time-to-live field that is used in every packet sent from this socket.
    #[inline]
    pub fn set_ttl(&self, ttl: u32) -> Result<(), Error> {
        self.listener.set_ttl(ttl)
    }

    /// Gets the value of the `IP_TTL` option for this socket.
    ///
    /// For more information about this option, see [set_ttl](https://docs.rs/socks5-server/latest/socks5_server/struct.Server.html#method.set_ttl).
    #[inline]
    pub fn ttl(&self) -> Result<u32, Error> {
        self.listener.ttl()
    }
}

impl<O> From<(TcpListener, AuthAdaptor<O>)> for Server<O> {
    #[inline]
    fn from((listener, auth): (TcpListener, AuthAdaptor<O>)) -> Self {
        Self { listener, auth }
    }
}

impl<O> From<Server<O>> for (TcpListener, AuthAdaptor<O>) {
    #[inline]
    fn from(server: Server<O>) -> Self {
        (server.listener, server.auth)
    }
}
