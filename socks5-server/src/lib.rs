#![doc = include_str!("../README.md")]

use std::{
    fmt::Debug,
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
        Command, IncomingConnection,
    },
};

pub(crate) type AuthAdaptor<A> = Arc<dyn Auth<Output = A> + Send + Sync>;

type ServerAcceptResult<A> = Result<
    (
        IncomingConnection<A, connection::state::NeedAuthenticate>,
        SocketAddr,
    ),
    Error,
>;

/// A SOCKS5 server listener
///
/// This server listens on a socket and treats incoming connections as SOCKS5 connections.
///
/// Generic `<A>` is the output type of the authentication adapter. See trait [`Auth`].
///
/// # Example
///
/// ```rust
/// use socks5_server::{auth::NoAuth, Server};
/// use std::sync::Arc;
/// use tokio::net::TcpListener;
///
/// async fn listen() {
///     let listener = TcpListener::bind("127.0.0.1:5000").await.unwrap();
///     let auth = Arc::new(NoAuth) as Arc<_>;
///
///     let server = Server::new(listener, auth);
///
///     while let Ok((conn, _)) = server.accept().await {
///         tokio::spawn(async move {
///             todo!();
///         });
///     }
/// }
/// ```
pub struct Server<A> {
    listener: TcpListener,
    auth: AuthAdaptor<A>,
}

impl<A> Server<A> {
    /// Creates a new [`Server<A>`] with a [`TcpListener`](tokio::net::TcpListener) and an `Arc<dyn Auth<Output = A> + Send + Sync>`.
    #[inline]
    pub fn new(listener: TcpListener, auth: AuthAdaptor<A>) -> Self {
        Self { listener, auth }
    }

    /// Accept an [`IncomingConnection`].
    ///
    /// The connection is only a freshly created TCP connection and may not be a valid SOCKS5 connection. You should call [`IncomingConnection::authenticate()`] to perform a SOCKS5 authentication handshake.
    #[inline]
    pub async fn accept(&self) -> ServerAcceptResult<A> {
        let (stream, addr) = self.listener.accept().await?;
        Ok((IncomingConnection::new(stream, self.auth.clone()), addr))
    }

    /// Polls to accept an [`IncomingConnection`].
    ///
    /// The connection is only a freshly created TCP connection and may not be a valid SOCKS5 connection. You should call [`IncomingConnection::authenticate()`] to perform a SOCKS5 authentication handshake.
    ///
    /// If there is no connection to accept, Poll::Pending is returned and the current task will be notified by a waker. Note that on multiple calls to poll_accept, only the Waker from the Context passed to the most recent call is scheduled to receive a wakeup.
    #[inline]
    pub fn poll_accept(&self, cx: &mut Context<'_>) -> Poll<ServerAcceptResult<A>> {
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

    /// Returns a shared reference to the listener.
    ///
    /// Note that this may break the encapsulation of the [`Server`] and you should not use this method unless you know what you are doing.
    #[inline]
    pub fn get_ref(&self) -> &TcpListener {
        &self.listener
    }

    /// Returns a mutable reference to the listener.
    ///
    /// Note that this may break the encapsulation of the [`Server`] and you should not use this method unless you know what you are doing.
    #[inline]
    pub fn get_mut(&mut self) -> &mut TcpListener {
        &mut self.listener
    }

    /// Consumes the [`Server<A>`] and returns the underlying [`TcpListener`](tokio::net::TcpListener) and `Arc<dyn Auth<Output = A> + Send + Sync>`.
    #[inline]
    pub fn into_inner(self) -> (TcpListener, AuthAdaptor<A>) {
        (self.listener, self.auth)
    }
}

impl<A> Debug for Server<A> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Server")
            .field("listener", &self.listener)
            .finish()
    }
}
