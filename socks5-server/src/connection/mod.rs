//! This module contains the connection abstraction of the socks5 protocol.
//!
//! [`accept()`](https://docs.rs/socks5-server/latest/socks5_server/struct.Server.html#method.accept) on a [`Server`](https://docs.rs/socks5-server/latest/socks5_server/struct.Server.html) creates a [`IncomingConnection`](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.IncomingConnection.html), which is the entry point of processing a socks5 connection.

use self::{associate::Associate, bind::Bind, connect::Connect};
use crate::AuthAdaptor;
use socks5_proto::{
    handshake::{
        Method as HandshakeMethod, Request as HandshakeRequest, Response as HandshakeResponse,
    },
    Address, Command as ProtocolCommand, Error, ProtocolError, Request,
};
use std::{io::Error as IoError, net::SocketAddr, time::Duration};
use tokio::net::TcpStream;

pub mod associate;
pub mod bind;
pub mod connect;

/// A freshly established TCP connection.
///
/// This may not be a valid socks5 connection. You should call [`authenticate()`](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.IncomingConnection.html#method.authenticate) to perform a socks5 authentication handshake.
///
/// It can also be converted back into a raw [`tokio::TcpStream`](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html) with `From` trait.
pub struct IncomingConnection<O> {
    stream: TcpStream,
    auth: AuthAdaptor<O>,
}

impl<O> IncomingConnection<O> {
    #[inline]
    pub(crate) fn new(stream: TcpStream, auth: AuthAdaptor<O>) -> Self {
        Self { stream, auth }
    }

    /// Perform a socks5 authentication handshake using the given [`Auth`](https://docs.rs/socks5-server/latest/socks5_server/auth/trait.Auth.html) adapter.
    ///
    /// If the handshake succeeds, an [`Authenticated`](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.Authenticated.html) alongs with the output of the [`Auth`](https://docs.rs/socks5-server/latest/socks5_server/auth/trait.Auth.html) adapter is returned. Otherwise, the error and the original [`TcpStream`](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html) is returned.
    pub async fn authenticate(mut self) -> Result<(Authenticated, O), (TcpStream, Error)> {
        let req = match HandshakeRequest::read_from(&mut self.stream).await {
            Ok(req) => req,
            Err(err) => return Err((self.stream, err)),
        };
        let chosen_method = self.auth.as_handshake_method();

        if req.methods.contains(&chosen_method) {
            let resp = HandshakeResponse::new(chosen_method);

            if let Err(err) = resp.write_to(&mut self.stream).await {
                return Err((self.stream, Error::Io(err)));
            }

            let output = self.auth.execute(&mut self.stream).await;

            Ok((Authenticated::new(self.stream), output))
        } else {
            let resp = HandshakeResponse::new(HandshakeMethod::UNACCEPTABLE);

            if let Err(err) = resp.write_to(&mut self.stream).await {
                return Err((self.stream, Error::Io(err)));
            }

            Err((
                self.stream,
                Error::Protocol(ProtocolError::NoAcceptableHandshakeMethod {
                    version: socks5_proto::SOCKS_VERSION,
                    chosen_method,
                    methods: req.methods,
                }),
            ))
        }
    }

    /// Returns the local address that this stream is bound to.
    #[inline]
    pub fn local_addr(&self) -> Result<SocketAddr, IoError> {
        self.stream.local_addr()
    }

    /// Reads the linger duration for this socket by getting the `SO_LINGER` option.
    ///
    /// For more information about this option, see [set_linger](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.IncomingConnection.html#method.set_linger).
    #[inline]
    pub fn linger(&self) -> Result<Option<Duration>, IoError> {
        self.stream.linger()
    }

    /// Sets the linger duration of this socket by setting the `SO_LINGER` option.
    ///
    /// This option controls the action taken when a stream has unsent messages and the stream is closed. If `SO_LINGER` is set, the system shall block the process until it can transmit the data or until the time expires.
    ///
    /// If `SO_LINGER` is not specified, and the stream is closed, the system handles the call in a way that allows the process to continue as quickly as possible.
    #[inline]
    pub fn set_linger(&self, dur: Option<Duration>) -> Result<(), IoError> {
        self.stream.set_linger(dur)
    }

    /// Gets the value of the `TCP_NODELAY` option on this socket.
    ///
    /// For more information about this option, see [set_nodelay](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.IncomingConnection.html#method.set_nodelay).
    #[inline]
    pub fn nodelay(&self) -> Result<bool, IoError> {
        self.stream.nodelay()
    }

    /// Sets the value of the `TCP_NODELAY` option on this socket.
    ///
    /// If set, this option disables the Nagle algorithm. This means that segments are always sent as soon as possible, even if there is only a small amount of data. When not set, data is buffered until there is a sufficient amount to send out, thereby avoiding the frequent sending of small packets.
    pub fn set_nodelay(&self, nodelay: bool) -> Result<(), IoError> {
        self.stream.set_nodelay(nodelay)
    }

    /// Gets the value of the `IP_TTL` option for this socket.
    ///
    /// For more information about this option, see [set_ttl](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.IncomingConnection.html#method.set_ttl).
    pub fn ttl(&self) -> Result<u32, IoError> {
        self.stream.ttl()
    }

    /// Sets the value for the `IP_TTL` option on this socket.
    ///
    /// This value sets the time-to-live field that is used in every packet sent from this socket.
    pub fn set_ttl(&self, ttl: u32) -> Result<(), IoError> {
        self.stream.set_ttl(ttl)
    }
}

impl<O> From<IncomingConnection<O>> for TcpStream {
    #[inline]
    fn from(conn: IncomingConnection<O>) -> Self {
        conn.stream
    }
}

/// A TCP stream that has been authenticated.
///
/// To get the command from the socks5 client, use [`wait_request`](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.Authenticated.html#method.wait_request).
///
/// It can also be converted back into a raw [`tokio::TcpStream`](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html) with `From` trait.
pub struct Authenticated(TcpStream);

impl Authenticated {
    #[inline]
    fn new(stream: TcpStream) -> Self {
        Self(stream)
    }

    /// Waits the socks5 client to send a command.
    ///
    /// This method will return a [`Command`](https://docs.rs/socks5-server/latest/socks5_server/connection/enum.Command.html) if the client sends a valid command.
    ///
    /// When encountering an error, the stream will be returned alongside the error.
    pub async fn wait_request(mut self) -> Result<Command, (TcpStream, Error)> {
        let req = match Request::read_from(&mut self.0).await {
            Ok(req) => req,
            Err(err) => return Err((self.0, err)),
        };

        match req.command {
            ProtocolCommand::Associate => Ok(Command::Associate(
                Associate::<associate::NeedReply>::new(self.0),
                req.address,
            )),
            ProtocolCommand::Bind => Ok(Command::Bind(
                Bind::<bind::NeedFirstReply>::new(self.0),
                req.address,
            )),
            ProtocolCommand::Connect => Ok(Command::Connect(
                Connect::<connect::NeedReply>::new(self.0),
                req.address,
            )),
        }
    }

    /// Returns the local address that this stream is bound to.
    #[inline]
    pub fn local_addr(&self) -> Result<SocketAddr, IoError> {
        self.0.local_addr()
    }

    /// Reads the linger duration for this socket by getting the `SO_LINGER` option.
    ///
    /// For more information about this option, see [set_linger](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.IncomingConnection.html#method.set_linger).
    #[inline]
    pub fn linger(&self) -> Result<Option<Duration>, IoError> {
        self.0.linger()
    }

    /// Sets the linger duration of this socket by setting the `SO_LINGER` option.
    ///
    /// This option controls the action taken when a stream has unsent messages and the stream is closed. If `SO_LINGER` is set, the system shall block the process until it can transmit the data or until the time expires.
    ///
    /// If `SO_LINGER` is not specified, and the stream is closed, the system handles the call in a way that allows the process to continue as quickly as possible.
    #[inline]
    pub fn set_linger(&self, dur: Option<Duration>) -> Result<(), IoError> {
        self.0.set_linger(dur)
    }

    /// Gets the value of the `TCP_NODELAY` option on this socket.
    ///
    /// For more information about this option, see [set_nodelay](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.IncomingConnection.html#method.set_nodelay).
    #[inline]
    pub fn nodelay(&self) -> Result<bool, IoError> {
        self.0.nodelay()
    }

    /// Sets the value of the `TCP_NODELAY` option on this socket.
    ///
    /// If set, this option disables the Nagle algorithm. This means that segments are always sent as soon as possible, even if there is only a small amount of data. When not set, data is buffered until there is a sufficient amount to send out, thereby avoiding the frequent sending of small packets.
    pub fn set_nodelay(&self, nodelay: bool) -> Result<(), IoError> {
        self.0.set_nodelay(nodelay)
    }

    /// Gets the value of the `IP_TTL` option for this socket.
    ///
    /// For more information about this option, see [set_ttl](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.IncomingConnection.html#method.set_ttl).
    pub fn ttl(&self) -> Result<u32, IoError> {
        self.0.ttl()
    }

    /// Sets the value for the `IP_TTL` option on this socket.
    ///
    /// This value sets the time-to-live field that is used in every packet sent from this socket.
    pub fn set_ttl(&self, ttl: u32) -> Result<(), IoError> {
        self.0.set_ttl(ttl)
    }
}

impl From<Authenticated> for TcpStream {
    #[inline]
    fn from(conn: Authenticated) -> Self {
        conn.0
    }
}

/// A command sent from the socks5 client.
pub enum Command {
    Associate(Associate<associate::NeedReply>, Address),
    Bind(Bind<bind::NeedFirstReply>, Address),
    Connect(Connect<connect::NeedReply>, Address),
}
