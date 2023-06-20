use self::{associate::Associate, bind::Bind, connect::Connect};
use crate::Auth;
use socks5_proto::{
    Address, Command, HandshakeMethod, HandshakeRequest, HandshakeResponse, Reply, Request,
    Response,
};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    io::{Error, ErrorKind, Result},
    net::SocketAddr,
    sync::Arc,
};
use tokio::{io::AsyncWriteExt, net::TcpStream};

pub mod associate;
pub mod bind;
pub mod connect;

/// A Incoming connection. This may not be a valid socks5 connection. You need to call [`handshake()`](#method.handshake) to perform the socks5 handshake. It will be converted to a proper socks5 connection after the handshake succeeds.
pub struct IncomingConnection<O> {
    stream: TcpStream,
    auth: Arc<dyn Auth<Output = O> + Send + Sync>,
}

impl<O> IncomingConnection<O> {
    #[inline]
    pub(crate) fn new(stream: TcpStream, auth: Arc<dyn Auth<Output = O> + Send + Sync>) -> Self {
        IncomingConnection { stream, auth }
    }

    /// Perform the socks5 handshake on this connection.
    pub async fn handshake(mut self) -> Result<Connection<O>> {
        let auth_output = match self.auth().await {
            Ok(output) => output,
            Err(err) => {
                let _ = self.stream.shutdown().await;
                return Err(err);
            }
        };

        let req = match Request::read_from(&mut self.stream).await {
            Ok(req) => req,
            Err(err) => {
                let resp = Response::new(Reply::GeneralFailure, Address::unspecified());
                resp.write_to(&mut self.stream).await?;
                let _ = self.stream.shutdown().await;
                return Err(err);
            }
        };

        match req.command {
            Command::Associate => Ok(Connection::Associate(
                Associate::<associate::NeedReply>::new(self.stream),
                req.address,
                auth_output,
            )),
            Command::Bind => Ok(Connection::Bind(
                Bind::<bind::NeedFirstReply>::new(self.stream),
                req.address,
                auth_output,
            )),
            Command::Connect => Ok(Connection::Connect(
                Connect::<connect::NeedReply>::new(self.stream),
                req.address,
                auth_output,
            )),
        }
    }

    /// Returns the local address that this stream is bound to.
    #[inline]
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.stream.local_addr()
    }

    /// Returns the remote address that this stream is connected to.
    #[inline]
    pub fn peer_addr(&self) -> Result<SocketAddr> {
        self.stream.peer_addr()
    }

    /// Shutdown the TCP stream.
    #[inline]
    pub async fn shutdown(&mut self) -> Result<()> {
        self.stream.shutdown().await
    }

    #[inline]
    async fn auth(&mut self) -> Result<O> {
        let hs_req = HandshakeRequest::read_from(&mut self.stream).await?;
        let chosen_method = self.auth.as_handshake_method();

        if hs_req.methods.contains(&chosen_method) {
            let hs_resp = HandshakeResponse::new(chosen_method);
            hs_resp.write_to(&mut self.stream).await?;
            self.auth.execute(&mut self.stream).await
        } else {
            let hs_resp = HandshakeResponse::new(HandshakeMethod::Unacceptable);
            hs_resp.write_to(&mut self.stream).await?;

            Err(Error::new(
                ErrorKind::Unsupported,
                "No available handshake method provided by client",
            ))
        }
    }
}

impl<T> Debug for IncomingConnection<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("IncomingConnection")
            .field("stream", &self.stream)
            .finish()
    }
}

/// After the socks5 handshake succeeds, the connection may become:
///
/// - Associate
/// - Bind
/// - Connect
#[derive(Debug)]
pub enum Connection<T> {
    Associate(Associate<associate::NeedReply>, Address, T),
    Bind(Bind<bind::NeedFirstReply>, Address, T),
    Connect(Connect<connect::NeedReply>, Address, T),
}
