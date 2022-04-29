use socks5_proto::{Address, Reply, Response};
use std::{
    io::{IoSlice, Result},
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpStream,
    },
};

pub struct Connect<S> {
    stream: TcpStream,
    _state: S,
}

pub struct NeedReply;
pub struct Ready;

impl Connect<NeedReply> {
    pub(super) fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _state: NeedReply,
        }
    }

    pub async fn reply(mut self, reply: Reply, addr: Address) -> Result<Connect<Ready>> {
        let resp = Response::new(reply, addr);
        resp.write_to(&mut self.stream).await?;
        Ok(Connect::<Ready>::new(self.stream))
    }
}

impl Connect<Ready> {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _state: Ready,
        }
    }

    #[inline]
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.stream.local_addr()
    }

    #[inline]
    pub fn peer_addr(&self) -> Result<SocketAddr> {
        self.stream.peer_addr()
    }

    #[inline]
    pub fn split(&mut self) -> (ReadHalf, WriteHalf) {
        self.stream.split()
    }
}

impl AsyncRead for Connect<Ready> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for Connect<Ready> {
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
