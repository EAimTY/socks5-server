use crate::Auth;
use socks5_proto::{
    Address, Command, HandshakeMethod, HandshakeRequest, HandshakeResponse, Reply, Request,
    Response,
};
use std::{
    io::{IoSlice, Result},
    net::{Ipv4Addr, SocketAddr},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::{TcpStream, UdpSocket},
};

pub struct IncomingConnection {
    stream: TcpStream,
    auth: Arc<dyn Auth>,
}

impl IncomingConnection {
    pub(crate) fn new(stream: TcpStream, auth: Arc<dyn Auth>) -> Self {
        IncomingConnection { stream, auth }
    }

    pub async fn handshake(mut self) -> Result<Connection> {
        let hs_req = HandshakeRequest::read_from(&mut self.stream).await?;
        let chosen_method = self.auth.as_handshake_method();

        if hs_req.methods.contains(&chosen_method) {
            let hs_resp = HandshakeResponse::new(chosen_method);
            hs_resp.write_to(&mut self.stream).await?;
            self.auth.execute(&mut self.stream).await?;
        } else {
            let hs_resp = HandshakeResponse::new(HandshakeMethod::Unacceptable);
            hs_resp.write_to(&mut self.stream).await?;
        }

        let req = match Request::read_from(&mut self.stream).await {
            Ok(req) => req,
            Err(err) => {
                let resp = Response::new(
                    Reply::GeneralFailure,
                    Address::SocketAddress(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0))),
                );
                resp.write_to(&mut self.stream).await?;
                return Err(err);
            }
        };

        match req.command {
            Command::Connect => Ok(Connection::Connect(
                Connect::<NeedResponse>::new(self.stream),
                req.address,
            )),
            Command::Bind => Ok(Connection::Bind(
                Bind::<NeedResponse>::new(self.stream),
                req.address,
            )),
            Command::Associate => Ok(Connection::Associate(
                Associate::<NeedResponse>::new(self.stream),
                req.address,
            )),
        }
    }
}

pub enum Connection {
    Connect(Connect<NeedResponse>, Address),
    Bind(Bind<NeedResponse>, Address),
    Associate(Associate<NeedResponse>, Address),
}

pub struct NeedResponse;
pub struct Ready;

pub struct Connect<T> {
    stream: TcpStream,
    _marker: T,
}

impl Connect<NeedResponse> {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _marker: NeedResponse,
        }
    }

    pub async fn response(mut self, reply: Reply, addr: Address) -> Result<Connect<Ready>> {
        let resp = Response::new(reply, addr);
        resp.write_to(&mut self.stream).await?;

        Ok(Connect::<Ready>::new(self.stream))
    }
}

impl Connect<Ready> {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _marker: Ready,
        }
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

pub struct Bind<T> {
    stream: TcpStream,
    _marker: T,
}

impl Bind<NeedResponse> {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _marker: NeedResponse,
        }
    }

    pub async fn response(mut self, reply: Reply, addr: Address) -> Result<Bind<Ready>> {
        let resp = Response::new(reply, addr);
        resp.write_to(&mut self.stream).await?;

        Ok(Bind::<Ready>::new(self.stream))
    }
}

impl Bind<Ready> {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _marker: Ready,
        }
    }
}

pub struct Associate<T> {
    stream: TcpStream,
    _udp_socket: Option<UdpSocket>,
    _marker: T,
}

impl Associate<NeedResponse> {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _udp_socket: None,
            _marker: NeedResponse,
        }
    }

    pub async fn response(
        mut self,
        reply: Reply,
        addr: Address,
        udp_socket: UdpSocket,
    ) -> Result<Associate<Ready>> {
        let resp = Response::new(reply, addr);
        resp.write_to(&mut self.stream).await?;

        Ok(Associate::<Ready>::new(self.stream, udp_socket))
    }
}

impl Associate<Ready> {
    fn new(stream: TcpStream, udp_socket: UdpSocket) -> Self {
        Self {
            stream,
            _udp_socket: Some(udp_socket),
            _marker: Ready,
        }
    }
}
