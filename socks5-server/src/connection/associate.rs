use bytes::{Bytes, BytesMut};
use socks5_proto::{Address, Reply, Response, UdpHeader};
use std::{io::Result, net::SocketAddr};
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
    pub(super) fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _state: NeedReply,
        }
    }

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
pub struct AssociateUdpSocket(UdpSocket);

impl AssociateUdpSocket {
    #[inline]
    pub async fn connect<A: ToSocketAddrs>(&self, addr: A) -> Result<()> {
        self.0.connect(addr).await
    }

    #[inline]
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.0.local_addr()
    }

    #[inline]
    pub fn peer_addr(&self) -> Result<SocketAddr> {
        self.0.peer_addr()
    }

    pub async fn recv(&self) -> Result<(Bytes, u8, Address)> {
        loop {
            let mut buf = vec![0; 65535];
            let len = self.0.recv(&mut buf).await?;
            buf.truncate(len);
            let pkt = Bytes::from(buf);

            if let Ok(header) = UdpHeader::read_from(&mut pkt.as_ref()).await {
                return Ok((pkt, header.frag, header.address));
            }
        }
    }

    pub async fn recv_from(&self) -> Result<(Bytes, u8, Address, SocketAddr)> {
        loop {
            let mut buf = vec![0; 65535];
            let (len, src_addr) = self.0.recv_from(&mut buf).await?;
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

        self.0
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

        self.0
            .send_to(&buf, to_addr)
            .await
            .map(|len| len - header.serialized_len())
    }
}

impl From<UdpSocket> for AssociateUdpSocket {
    fn from(socket: UdpSocket) -> Self {
        AssociateUdpSocket(socket)
    }
}

impl From<AssociateUdpSocket> for UdpSocket {
    fn from(associate: AssociateUdpSocket) -> Self {
        associate.0
    }
}
