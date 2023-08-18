//! Socks5 command type `Connect`

use socks5_proto::{Address, Reply, Response};
use std::{
    io::Error,
    marker::PhantomData,
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf},
    net::TcpStream,
};

/// Connection state types
pub mod state {
    #[derive(Debug)]
    pub struct NeedReply;

    #[derive(Debug)]
    pub struct Ready;
}

/// Socks5 command type `Connect`
///
/// Reply the client with [`Connect::reply()`] to complete the command negotiation.
#[derive(Debug)]
pub struct Connect<S> {
    stream: TcpStream,
    _state: PhantomData<S>,
}

impl Connect<state::NeedReply> {
    /// Reply to the SOCKS5 client with the given reply and address.
    ///
    /// If encountered an error while writing the reply, the error alongside the original `TcpStream` is returned.
    pub async fn reply(
        mut self,
        reply: Reply,
        addr: Address,
    ) -> Result<Connect<state::Ready>, (Error, TcpStream)> {
        let resp = Response::new(reply, addr);

        if let Err(err) = resp.write_to(&mut self.stream).await {
            return Err((err, self.stream));
        }

        Ok(Connect::new(self.stream))
    }
}

impl<S> Connect<S> {
    #[inline]
    pub(super) fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _state: PhantomData,
        }
    }

    /// Causes the other peer to receive a read of length 0, indicating that no more data will be sent. This only closes the stream in one direction.
    #[inline]
    pub async fn close(&mut self) -> Result<(), Error> {
        self.stream.shutdown().await
    }

    /// Returns the local address that this stream is bound to.
    #[inline]
    pub fn local_addr(&self) -> Result<SocketAddr, Error> {
        self.stream.local_addr()
    }

    /// Returns the remote address that this stream is connected to.
    #[inline]
    pub fn peer_addr(&self) -> Result<SocketAddr, Error> {
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

    /// Consumes the [`Connect<S>`] and returns the underlying [`TcpStream`](tokio::net::TcpStream).
    #[inline]
    pub fn into_inner(self) -> TcpStream {
        self.stream
    }
}

impl AsyncRead for Connect<state::Ready> {
    #[inline]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), Error>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for Connect<state::Ready> {
    #[inline]
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    #[inline]
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    #[inline]
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}
