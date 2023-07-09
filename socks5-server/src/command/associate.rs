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

/// Socks5 connection type `Associate`
///
/// [`AssociatedUdpSocket`](https://docs.rs/socks5-server/latest/socks5_server/connection/associate/struct.AssociatedUdpSocket.html) can be used as the associated UDP socket.
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

    /// Reply the associated UDP socket address to the client.
    #[inline]
    pub async fn reply(mut self, reply: Reply, addr: Address) -> Result<Associate<Ready>> {
        let resp = Response::new(reply, addr);
        resp.write_to(&mut self.stream).await?;
        Ok(Associate::<Ready>::new(self.stream))
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

impl Associate<Ready> {
    #[inline]
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _state: Ready,
        }
    }

    /// Wait until the client closes this TCP connection.
    ///
    /// Socks5 protocol defines that when the client closes the TCP connection used to send the associate command, the server should release the associated UDP socket.
    pub async fn wait_until_closed(&mut self) -> Result<()> {
        loop {
            match self.stream.read(&mut [0]).await {
                Ok(0) => break Ok(()),
                Ok(_) => {}
                Err(err) => break Err(err),
            }
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
}

/// This is a helper for managing the associated UDP socket.
///
/// It will add the socks5 UDP header to every UDP packet it sends, also try to parse the socks5 UDP header from any UDP packet received.
///
/// The receiving buffer size for each UDP packet can be set with [`set_recv_buffer_size()`](#method.set_recv_buffer_size), and be read with [`get_max_packet_size()`](#method.get_recv_buffer_size).
///
/// You can create this struct by using [`AssociatedUdpSocket::from::<(UdpSocket, usize)>()`](#impl-From<UdpSocket>), the first element of the tuple is the UDP socket, the second element is the receiving buffer size.
///
/// This struct can also be revert into a raw tokio UDP socket with [`UdpSocket::from::<AssociatedUdpSocket>()`](#impl-From<AssociatedUdpSocket>).
#[derive(Debug)]
pub struct AssociatedUdpSocket {
    socket: UdpSocket,
    buf_size: AtomicUsize,
}

impl AssociatedUdpSocket {
    /// Connects the UDP socket setting the default destination for send() and limiting packets that are read via recv from the address specified in addr.
    #[inline]
    pub async fn connect<A: ToSocketAddrs>(&self, addr: A) -> Result<()> {
        self.socket.connect(addr).await
    }

    /// Returns the local address that this socket is bound to.
    #[inline]
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.socket.local_addr()
    }

    /// Returns the socket address of the remote peer this socket was connected to.
    #[inline]
    pub fn peer_addr(&self) -> Result<SocketAddr> {
        self.socket.peer_addr()
    }

    /// Get the maximum UDP packet size, with socks5 UDP header included.
    pub fn get_max_packet_size(&self) -> usize {
        self.buf_size.load(Ordering::Relaxed)
    }

    /// Set the maximum UDP packet size, with socks5 UDP header included, for adjusting the receiving buffer size.
    pub fn set_max_packet_size(&self, size: usize) {
        self.buf_size.store(size, Ordering::Release);
    }

    /// Receives a socks5 UDP relay packet on the socket from the remote address to which it is connected. On success, returns the packet itself, the fragment number and the remote target address.
    ///
    /// The [`connect`](#method.connect) method will connect this socket to a remote address. This method will fail if the socket is not connected.
    pub async fn recv(&self) -> Result<(Bytes, u8, Address)> {
        loop {
            let max_packet_size = self.buf_size.load(Ordering::Acquire);
            let mut buf = vec![0; max_packet_size];
            let len = self.socket.recv(&mut buf).await?;
            buf.truncate(len);
            let pkt = Bytes::from(buf);

            if let Ok(header) = UdpHeader::read_from(&mut pkt.as_ref()).await {
                let pkt = pkt.slice(header.serialized_len()..);
                return Ok((pkt, header.frag, header.address));
            }
        }
    }

    /// Receives a socks5 UDP relay packet on the socket from the any remote address. On success, returns the packet itself, the fragment number, the remote target address and the source address.
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

    /// Sends a UDP relay packet to the remote address to which it is connected. The socks5 UDP header will be added to the packet.
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

    /// Sends a UDP relay packet to a specified remote address to which it is connected. The socks5 UDP header will be added to the packet.
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

impl From<(UdpSocket, usize)> for AssociatedUdpSocket {
    #[inline]
    fn from(from: (UdpSocket, usize)) -> Self {
        AssociatedUdpSocket {
            socket: from.0,
            buf_size: AtomicUsize::new(from.1),
        }
    }
}

impl From<AssociatedUdpSocket> for UdpSocket {
    #[inline]
    fn from(from: AssociatedUdpSocket) -> Self {
        from.socket
    }
}
