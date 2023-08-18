//! This module contains the connection abstraction of the SOCKS5 protocol.
//!
//! [`Server::accept()`] creates an [`IncomingConnection`], which is the entry point of processing a SOCKS5 connection.

use self::{associate::Associate, bind::Bind, connect::Connect};
use crate::AuthAdaptor;
use socks5_proto::{
    handshake::{
        Method as HandshakeMethod, Request as HandshakeRequest, Response as HandshakeResponse,
    },
    Address, Command as ProtocolCommand, Error, ProtocolError, Request,
};
use std::{io::Error as IoError, net::SocketAddr};
use tokio::{io::AsyncWriteExt, net::TcpStream};

pub mod associate;
pub mod bind;
pub mod connect;

/// A freshly established TCP connection.
///
/// This may not be a valid SOCKS5 connection. You should call [`IncomingConnection::authenticate()`] to perform a SOCKS5 authentication handshake.
pub struct IncomingConnection<O> {
    stream: TcpStream,
    auth: AuthAdaptor<O>,
}

impl<O> IncomingConnection<O> {
    #[inline]
    pub(crate) fn new(stream: TcpStream, auth: AuthAdaptor<O>) -> Self {
        Self { stream, auth }
    }

    /// Perform a SOCKS5 authentication handshake using the given [`Auth`] adapter.
    ///
    /// If the handshake succeeds, an [`Authenticated`] alongs with the output of the [`Auth`] adapter is returned. Otherwise, the error and the underlying [`tokio::net::TcpStream`] is returned.
    ///
    /// Note that this method will not implicitly close the connection even if the handshake failed.
    pub async fn authenticate(mut self) -> Result<(Authenticated, O), (Error, TcpStream)> {
        let req = match HandshakeRequest::read_from(&mut self.stream).await {
            Ok(req) => req,
            Err(err) => return Err((err, self.stream)),
        };
        let chosen_method = self.auth.as_handshake_method();

        if req.methods.contains(&chosen_method) {
            let resp = HandshakeResponse::new(chosen_method);

            if let Err(err) = resp.write_to(&mut self.stream).await {
                return Err((Error::Io(err), self.stream));
            }

            let output = self.auth.execute(&mut self.stream).await;

            Ok((Authenticated::new(self.stream), output))
        } else {
            let resp = HandshakeResponse::new(HandshakeMethod::UNACCEPTABLE);

            if let Err(err) = resp.write_to(&mut self.stream).await {
                return Err((Error::Io(err), self.stream));
            }

            Err((
                Error::Protocol(ProtocolError::NoAcceptableHandshakeMethod {
                    version: socks5_proto::SOCKS_VERSION,
                    chosen_method,
                    methods: req.methods,
                }),
                self.stream,
            ))
        }
    }

    /// Causes the other peer to receive a read of length 0, indicating that no more data will be sent. This only closes the stream in one direction.
    #[inline]
    pub async fn close(&mut self) -> Result<(), IoError> {
        self.stream.shutdown().await
    }

    /// Returns the local address that this stream is bound to.
    #[inline]
    pub fn local_addr(&self) -> Result<SocketAddr, IoError> {
        self.stream.local_addr()
    }

    /// Returns the remote address that this stream is connected to.
    #[inline]
    pub fn peer_addr(&self) -> Result<SocketAddr, IoError> {
        self.stream.peer_addr()
    }

    /// Returns a shared reference to the underlying stream.
    ///
    /// Note that this may break the encapsulation of the SOCKS5 connection and you should not use this method unless you know what you are doing.
    #[inline]
    pub fn get_ref(&self) -> &TcpStream {
        &self.stream
    }

    /// Returns a mutable reference to the underlying stream.
    ///
    /// Note that this may break the encapsulation of the SOCKS5 connection and you should not use this method unless you know what you are doing.
    #[inline]
    pub fn get_mut(&mut self) -> &mut TcpStream {
        &mut self.stream
    }

    /// Consumes the [`IncomingConnection`] and returns the underlying [`tokio::net::TcpStream`].
    #[inline]
    pub fn into_inner(self) -> TcpStream {
        self.stream
    }
}

/// A TCP stream that has been authenticated.
///
/// To get the command from the SOCKS5 client, use [`Authenticated::wait_request()`].
pub struct Authenticated(TcpStream);

impl Authenticated {
    #[inline]
    fn new(stream: TcpStream) -> Self {
        Self(stream)
    }

    /// Waits the SOCKS5 client to send a request.
    ///
    /// This method will return a [`Command`] if the client sends a valid command.
    ///
    /// When encountering an error, the stream will be returned alongside the error.
    ///
    /// Note that this method will not implicitly close the connection even if the client sends an invalid request.
    pub async fn wait_request(mut self) -> Result<Command, (Error, TcpStream)> {
        let req = match Request::read_from(&mut self.0).await {
            Ok(req) => req,
            Err(err) => return Err((err, self.0)),
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

    /// Causes the other peer to receive a read of length 0, indicating that no more data will be sent. This only closes the stream in one direction.
    #[inline]
    pub async fn close(&mut self) -> Result<(), IoError> {
        self.0.shutdown().await
    }

    /// Returns the local address that this stream is bound to.
    #[inline]
    pub fn local_addr(&self) -> Result<SocketAddr, IoError> {
        self.0.local_addr()
    }

    /// Returns the remote address that this stream is connected to.
    #[inline]
    pub fn peer_addr(&self) -> Result<SocketAddr, IoError> {
        self.0.peer_addr()
    }

    /// Returns a shared reference to the underlying stream.
    ///
    /// Note that this may break the encapsulation of the SOCKS5 connection and you should not use this method unless you know what you are doing.
    #[inline]
    pub fn get_ref(&self) -> &TcpStream {
        &self.0
    }

    /// Returns a mutable reference to the underlying stream.
    ///
    /// Note that this may break the encapsulation of the SOCKS5 connection and you should not use this method unless you know what you are doing.
    #[inline]
    pub fn get_mut(&mut self) -> &mut TcpStream {
        &mut self.0
    }

    /// Consumes the [`Authenticated`] and returns the underlying [`tokio::net::TcpStream`].
    #[inline]
    pub fn into_inner(self) -> TcpStream {
        self.0
    }
}

/// A command sent from the SOCKS5 client.
pub enum Command {
    Associate(Associate<associate::NeedReply>, Address),
    Bind(Bind<bind::NeedFirstReply>, Address),
    Connect(Connect<connect::NeedReply>, Address),
}
