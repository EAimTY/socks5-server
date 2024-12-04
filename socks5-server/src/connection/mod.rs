//! Connection abstraction of the SOCKS5 protocol

use self::{associate::Associate, bind::Bind, connect::Connect};
use crate::AuthAdaptor;
use socks5_proto::{
    handshake::{
        Method as HandshakeMethod, Request as HandshakeRequest, Response as HandshakeResponse,
    },
    Address, Command as ProtocolCommand, Error, ProtocolError, Request,
};
use std::{fmt::Debug, io::Error as IoError, marker::PhantomData, net::SocketAddr};
use tokio::{io::AsyncWriteExt, net::TcpStream};

pub mod associate;
pub mod bind;
pub mod connect;

/// Incoming connection state types
pub mod state {
    #[derive(Debug)]
    pub struct NeedAuthenticate;

    #[derive(Debug)]
    pub struct NeedCommand;
}

/// An incoming SOCKS5 connection.
///
/// This may not be a valid SOCKS5 connection. You should call [`IncomingConnection::authenticate()`] and [`IncomingConnection::wait()`] to perform a SOCKS5 connection negotiation.
pub struct IncomingConnection<A, S> {
    stream: TcpStream,
    auth: AuthAdaptor<A>,
    _state: PhantomData<S>,
}

impl<A> IncomingConnection<A, state::NeedAuthenticate> {
    /// Perform a SOCKS5 authentication handshake using the given [`Auth`](crate::Auth) adapter.
    ///
    /// If the handshake succeeds, an [`IncomingConnection<A, state::NeedCommand>`] alongs with the output of the [`Auth`](crate::Auth) adapter `A` is returned. Otherwise, the error and the underlying [`TcpStream`](tokio::net::TcpStream) is returned.
    ///
    /// Note that this method will not implicitly close the connection even if the handshake failed.
    pub async fn authenticate(
        mut self,
    ) -> Result<(IncomingConnection<A, state::NeedCommand>, A), (Error, TcpStream)> {
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

            Ok((IncomingConnection::new(self.stream, self.auth), output))
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
}

impl<A> IncomingConnection<A, state::NeedCommand> {
    /// Waits the SOCKS5 client to send a request.
    ///
    /// This method will return a [`Command`] if the client sends a valid command.
    ///
    /// When encountering an error, the stream will be returned alongside the error.
    ///
    /// Note that this method will not implicitly close the connection even if the client sends an invalid command.
    pub async fn wait(mut self) -> Result<Command, (Error, TcpStream)> {
        let req = match Request::read_from(&mut self.stream).await {
            Ok(req) => req,
            Err(err) => return Err((err, self.stream)),
        };

        match req.command {
            ProtocolCommand::Associate => {
                Ok(Command::Associate(Associate::new(self.stream), req.address))
            }
            ProtocolCommand::Bind => Ok(Command::Bind(Bind::new(self.stream), req.address)),
            ProtocolCommand::Connect => {
                Ok(Command::Connect(Connect::new(self.stream), req.address))
            }
        }
    }
}

impl<A, S> IncomingConnection<A, S> {
    #[inline]
    pub fn new(stream: TcpStream, auth: AuthAdaptor<A>) -> Self {
        Self {
            stream,
            auth,
            _state: PhantomData,
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

    /// Consumes the [`IncomingConnection`] and returns the underlying [`TcpStream`](tokio::net::TcpStream).
    #[inline]
    pub fn into_inner(self) -> TcpStream {
        self.stream
    }
}

impl<A, S> Debug for IncomingConnection<A, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IncomingConnection")
            .field("stream", &self.stream)
            .finish()
    }
}

/// A command sent from the SOCKS5 client.
#[derive(Debug)]
pub enum Command {
    Associate(Associate<associate::state::NeedReply>, Address),
    Bind(Bind<bind::state::NeedFirstReply>, Address),
    Connect(Connect<connect::state::NeedReply>, Address),
}
