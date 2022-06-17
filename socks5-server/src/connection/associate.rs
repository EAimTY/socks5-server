use bytes::{Bytes, BytesMut};
use socks5_proto::{Address, Reply, Response, UdpHeader};
use std::{
    io::Result,
    net::SocketAddr,
    sync::atomic::{AtomicUsize, Ordering},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, ToSocketAddrs, UdpSocket},
};

#[derive(Debug)]
pub struct Associate<S> {
    stream: TcpStream,
    _state: S,
}

#[derive(Debug)]
pub struct NeedReply;

#[derive(Debug)]
pub struct Ready;

impl Associate<NeedReply> {
    #[inline]
    pub(super) fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _state: NeedReply,
        }
    }

    #[inline]
    pub async fn reply(mut self, reply: Reply, addr: Address) -> Result<Associate<Ready>> {
        let resp = Response::new(reply, addr);
        resp.write_to(&mut self.stream).await?;
        Ok(Associate::<Ready>::new(self.stream))
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
    pub async fn shutdown(&mut self) -> Result<()> {
        self.stream.shutdown().await
    }
}

impl Associate<Ready> {
    #[inline]
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _state: Ready,
        }
    }

    pub async fn wait_until_closed(&mut self) -> Result<()> {
        loop {
            match self.stream.read(&mut [0]).await {
                Ok(0) => break Ok(()),
                Ok(_) => {}
                Err(err) => break Err(err),
            }
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
    pub async fn shutdown(&mut self) -> Result<()> {
        self.stream.shutdown().await
    }
}

#[derive(Debug)]
pub struct AssociateUdpSocket {
    socket: UdpSocket,
    buf_size: AtomicUsize,
}

impl AssociateUdpSocket {
    #[inline]
    pub async fn connect<A: ToSocketAddrs>(&self, addr: A) -> Result<()> {
        self.socket.connect(addr).await
    }

    #[inline]
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.socket.local_addr()
    }

    #[inline]
    pub fn peer_addr(&self) -> Result<SocketAddr> {
        self.socket.peer_addr()
    }

    pub fn get_max_packet_size(&self) -> usize {
        self.buf_size.load(Ordering::Relaxed)
    }

    pub fn set_max_packet_size(&self, size: usize) {
        self.buf_size.store(size, Ordering::Release);
    }

    pub async fn recv(&self) -> Result<(Bytes, u8, Address)> {
        loop {
            let max_packet_size = self.buf_size.load(Ordering::Acquire);
            let mut buf = vec![0; max_packet_size];
            let len = self.socket.recv(&mut buf).await?;
            buf.truncate(len);
            let pkt = Bytes::from(buf);

            if let Ok(header) = UdpHeader::read_from(&mut pkt.as_ref()).await {
                return Ok((pkt, header.frag, header.address));
            }
        }
    }

    pub async fn recv_from(&self) -> Result<(Bytes, u8, Address, SocketAddr)> {
        loop {
            let max_packet_size = self.buf_size.load(Ordering::Acquire);
            let mut buf = vec![0; max_packet_size];
            let (len, src_addr) = self.socket.recv_from(&mut buf).await?;
            buf.truncate(len);
            let pkt = Bytes::from(buf);

            if let Ok(header) = UdpHeader::read_from(&mut pkt.as_ref()).await {
                let pkt = pkt.slice(header.serialized_len()..);
                return Ok((pkt, header.frag, header.address, src_addr));
            }
        }
    }

    pub async fn send<P: AsRef<[u8]>>(
        &self,
        pkt: P,
        frag: u8,
        from_addr: Address,
    ) -> Result<usize> {
        let header = UdpHeader::new(frag, from_addr);
        let mut buf = BytesMut::with_capacity(header.serialized_len() + pkt.as_ref().len());
        header.write_to_buf(&mut buf);
        buf.extend_from_slice(pkt.as_ref());

        self.socket
            .send(&buf)
            .await
            .map(|len| len - header.serialized_len())
    }

    pub async fn send_to<P: AsRef<[u8]>>(
        &self,
        pkt: P,
        frag: u8,
        from_addr: Address,
        to_addr: SocketAddr,
    ) -> Result<usize> {
        let header = UdpHeader::new(frag, from_addr);
        let mut buf = BytesMut::with_capacity(header.serialized_len() + pkt.as_ref().len());
        header.write_to_buf(&mut buf);
        buf.extend_from_slice(pkt.as_ref());

        self.socket
            .send_to(&buf, to_addr)
            .await
            .map(|len| len - header.serialized_len())
    }
}

impl From<(UdpSocket, usize)> for AssociateUdpSocket {
    #[inline]
    fn from(from: (UdpSocket, usize)) -> Self {
        AssociateUdpSocket {
            socket: from.0,
            buf_size: AtomicUsize::new(from.1),
        }
    }
}

impl From<AssociateUdpSocket> for UdpSocket {
    #[inline]
    fn from(from: AssociateUdpSocket) -> Self {
        from.socket
    }
}
