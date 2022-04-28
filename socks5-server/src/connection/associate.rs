use socks5_proto::{Address, Reply, Response};
use std::io::Result;
use tokio::net::{TcpStream, UdpSocket};

pub struct Associate<S> {
    stream: TcpStream,
    _udp_socket: Option<UdpSocket>,
    _state: S,
}

pub struct NeedReply;
pub struct NeedUdpSocket;
pub struct Ready;

impl Associate<NeedReply> {
    pub(super) fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _udp_socket: None,
            _state: NeedReply,
        }
    }

    pub async fn reply(mut self, reply: Reply, addr: Address) -> Result<Associate<NeedUdpSocket>> {
        let resp = Response::new(reply, addr);
        resp.write_to(&mut self.stream).await?;
        Ok(Associate::<NeedUdpSocket>::new(self.stream))
    }
}

impl Associate<NeedUdpSocket> {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _udp_socket: None,
            _state: NeedUdpSocket,
        }
    }

    pub async fn bind(self, udp_socket: UdpSocket) -> Result<Associate<Ready>> {
        Ok(Associate::<Ready>::new(self.stream, udp_socket))
    }
}

impl Associate<Ready> {
    fn new(stream: TcpStream, udp_socket: UdpSocket) -> Self {
        Self {
            stream,
            _udp_socket: Some(udp_socket),
            _state: Ready,
        }
    }
}
