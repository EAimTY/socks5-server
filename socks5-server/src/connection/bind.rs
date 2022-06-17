use socks5_proto::{Address, Reply, Response};
use std::{
    io::{IoSlice, Result},
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf},
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpStream,
    },
};

/// Socks5 connection type `Bind`
///
/// This connection can be used as a regular async TCP stream after replying the client.
#[derive(Debug)]
pub struct Bind<S> {
    stream: TcpStream,
    _state: S,
}

#[derive(Debug)]
pub struct NeedFirstReply;

#[derive(Debug)]
pub struct NeedSecondReply;

#[derive(Debug)]
pub struct Ready;

impl Bind<NeedFirstReply> {
    #[inline]
    pub(super) fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _state: NeedFirstReply,
        }
    }

    /// Reply to the client the first time.
    #[inline]
    pub async fn reply(mut self, reply: Reply, addr: Address) -> Result<Bind<NeedSecondReply>> {
        let resp = Response::new(reply, addr);
        resp.write_to(&mut self.stream).await?;
        Ok(Bind::<NeedSecondReply>::new(self.stream))
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
}

impl Bind<NeedSecondReply> {
    #[inline]
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _state: NeedSecondReply,
        }
    }

    /// Reply to the client the second time.
    #[inline]
    pub async fn reply(mut self, reply: Reply, addr: Address) -> Result<Bind<Ready>> {
        let resp = Response::new(reply, addr);
        resp.write_to(&mut self.stream).await?;
        Ok(Bind::<Ready>::new(self.stream))
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
}

impl Bind<Ready> {
    #[inline]
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _state: Ready,
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

    /// Split the connection into a read and a write half.
    #[inline]
    pub fn split(&mut self) -> (ReadHalf, WriteHalf) {
        self.stream.split()
    }

    /// Shutdown the TCP stream.
    #[inline]
    pub async fn shutdown(&mut self) -> Result<()> {
        self.stream.shutdown().await
    }
}

impl AsyncRead for Bind<Ready> {
    #[inline]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for Bind<Ready> {
    #[inline]
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    #[inline]
    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<Result<usize>> {
        Pin::new(&mut self.stream).poll_write_vectored(cx, bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.stream.is_write_vectored()
    }

    #[inline]
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    #[inline]
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}
