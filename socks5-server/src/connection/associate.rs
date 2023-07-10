//! Socks5 command type `Associate`
//!
//! This module also provides an [`UdpSocket`](https://docs.rs/tokio/latest/tokio/net/struct.UdpSocket.html) wrapper [`AssociatedUdpSocket`](https://docs.rs/socks5-server/latest/socks5_server/connection/associate/struct.AssociatedUdpSocket.html), which can be used to send and receive UDP packets without dealing with the socks5 protocol UDP header.

use bytes::{Bytes, BytesMut};
use socks5_proto::{Address, Error as Socks5Error, Reply, Response, UdpHeader};
use std::{
    io::{Cursor, Error},
    marker::PhantomData,
    net::SocketAddr,
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::atomic::{AtomicUsize, Ordering},
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf},
    net::{TcpStream, UdpSocket},
};

/// Socks5 command type `Associate`
///
/// By [`wait_request()`](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.Authenticated.html#method.wait_request) on an [`Authenticated`](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.Authenticated.html) from socks5 client, you may get a `Associate<NeedReply>`. After replying the client using [`reply()`](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.Associate.html#method.reply), you will get a `Associate<Ready>`, which can be used as a regular async TCP stream.
///
/// A `Associate<S>` can be converted to a regular tokio [`TcpStream`](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html) by using the `From` trait.
///
/// This module also provides an [`UdpSocket`](https://docs.rs/tokio/latest/tokio/net/struct.UdpSocket.html) wrapper [`AssociatedUdpSocket`](https://docs.rs/socks5-server/latest/socks5_server/connection/associate/struct.AssociatedUdpSocket.html), which can be used to send and receive UDP packets without dealing with the socks5 protocol UDP header.
#[derive(Debug)]
pub struct Associate<S> {
    stream: TcpStream,
    _state: PhantomData<S>,
}

/// Marker type indicating that the connection needs to be replied.
#[derive(Debug)]
pub struct NeedReply;

/// Marker type indicating that the connection is ready to use as a regular TCP stream.
#[derive(Debug)]
pub struct Ready;

impl Associate<NeedReply> {
    #[inline]
    pub(super) fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _state: PhantomData,
        }
    }

    /// Reply to the socks5 client with the given reply and address.
    ///
    /// If encountered an error while writing the reply, the error alongside the original `TcpStream` is returned.
    pub async fn reply(
        mut self,
        reply: Reply,
        addr: Address,
    ) -> Result<Associate<Ready>, (Error, TcpStream)> {
        let resp = Response::new(reply, addr);

        if let Err(err) = resp.write_to(&mut self.stream).await {
            return Err((err, self.stream));
        }

        Ok(Associate::<Ready>::new(self.stream))
    }

    /// Causes the other peer to receive a read of length 0, indicating that no more data will be sent. This only closes the stream in one direction.
    #[inline]
    pub async fn shutdown(&mut self) -> Result<(), Error> {
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

    /// Reads the linger duration for this socket by getting the `SO_LINGER` option.
    ///
    /// For more information about this option, see [set_linger](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.Connect.html#method.set_linger).
    #[inline]
    pub fn linger(&self) -> Result<Option<Duration>, Error> {
        self.stream.linger()
    }

    /// Sets the linger duration of this socket by setting the `SO_LINGER` option.
    ///
    /// This option controls the action taken when a stream has unsent messages and the stream is closed. If `SO_LINGER` is set, the system shall block the process until it can transmit the data or until the time expires.
    ///
    /// If `SO_LINGER` is not specified, and the stream is closed, the system handles the call in a way that allows the process to continue as quickly as possible.
    #[inline]
    pub fn set_linger(&self, dur: Option<Duration>) -> Result<(), Error> {
        self.stream.set_linger(dur)
    }

    /// Gets the value of the `TCP_NODELAY` option on this socket.
    ///
    /// For more information about this option, see [set_nodelay](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.Connect.html#method.set_nodelay).
    #[inline]
    pub fn nodelay(&self) -> Result<bool, Error> {
        self.stream.nodelay()
    }

    /// Sets the value of the `TCP_NODELAY` option on this socket.
    ///
    /// If set, this option disables the Nagle algorithm. This means that segments are always sent as soon as possible, even if there is only a small amount of data. When not set, data is buffered until there is a sufficient amount to send out, thereby avoiding the frequent sending of small packets.
    pub fn set_nodelay(&self, nodelay: bool) -> Result<(), Error> {
        self.stream.set_nodelay(nodelay)
    }

    /// Gets the value of the `IP_TTL` option for this socket.
    ///
    /// For more information about this option, see [set_ttl](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.Connect.html#method.set_ttl).
    pub fn ttl(&self) -> Result<u32, Error> {
        self.stream.ttl()
    }

    /// Sets the value for the `IP_TTL` option on this socket.
    ///
    /// This value sets the time-to-live field that is used in every packet sent from this socket.
    pub fn set_ttl(&self, ttl: u32) -> Result<(), Error> {
        self.stream.set_ttl(ttl)
    }
}

impl Associate<Ready> {
    #[inline]
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _state: PhantomData,
        }
    }

    /// Wait until the socks5 client closes this TCP connection.
    ///
    /// Socks5 protocol defines that when the client closes the TCP connection used to send the associate command, the server should release the associated UDP socket.
    pub async fn wait_until_closed(&mut self) -> Result<(), Error> {
        loop {
            match self.stream.read(&mut [0]).await {
                Ok(0) => break Ok(()),
                Ok(_) => {}
                Err(err) => break Err(err),
            }
        }
    }
}

impl Deref for Associate<Ready> {
    type Target = TcpStream;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl DerefMut for Associate<Ready> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
    }
}

impl AsyncRead for Associate<Ready> {
    #[inline]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), Error>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for Associate<Ready> {
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

impl<S> From<Associate<S>> for TcpStream {
    #[inline]
    fn from(conn: Associate<S>) -> Self {
        conn.stream
    }
}

/// A wrapper of a tokio UDP socket dealing with socks5 UDP header.
///
/// `(UdpSocket, usize)` and `AssociatedUdpSocket` can be converted to each other with `From` trait, in which `usize` is the maximum receiving UDP packet size, with socks5 UDP header included.
///
/// It only provides handful of methods to send / receive UDP packets with socks5 UDP header. However, the underlying `UdpSocket` can be accessed with `AsRef` and `AsMut` trait, so you can use all methods provided by `UdpSocket`.
#[derive(Debug)]
pub struct AssociatedUdpSocket {
    socket: UdpSocket,
    buf_size: AtomicUsize,
}

impl AssociatedUdpSocket {
    /// Get the maximum receiving UDP packet size, with socks5 UDP header included.
    #[inline]
    pub fn get_max_pkt_size(&self) -> usize {
        self.buf_size.load(Ordering::Relaxed)
    }

    /// Set the maximum receiving UDP packet size, with socks5 UDP header included, for adjusting the receiving buffer size.
    #[inline]
    pub fn set_max_pkt_size(&self, size: usize) {
        self.buf_size.store(size, Ordering::Release);
    }

    /// Receives a socks5 UDP packet on the socket from the remote address which it is connected.
    ///
    /// On success, it returns the packet payload and the socks5 UDP header. On error, it returns the error alongside an `Option<Vec<u8>>`. If the error occurs before / when receiving the raw UDP packet, the `Option<Vec<u8>>` will be `None`. Otherwise, it will be `Some(Vec<u8>)` containing the received raw UDP packet.
    pub async fn recv(&self) -> Result<(Bytes, UdpHeader), (Socks5Error, Option<Vec<u8>>)> {
        let max_pkt_size = self.buf_size.load(Ordering::Acquire);
        let mut buf = vec![0; max_pkt_size];

        let len = match self.socket.recv(&mut buf).await {
            Ok(len) => len,
            Err(err) => return Err((Socks5Error::Io(err), None)),
        };

        buf.truncate(len);

        let header = match UdpHeader::read_from(&mut Cursor::new(buf.as_slice())).await {
            Ok(header) => header,
            Err(err) => return Err((err, Some(buf))),
        };

        let pkt = Bytes::from(buf).slice(header.serialized_len()..);

        Ok((pkt, header))
    }

    /// Receives a socks5 UDP packet on the socket from a remote address.
    ///
    /// On success, it returns the packet payload, the socks5 UDP header and the source address. On error, it returns the error alongside an `Option<Vec<u8>>`. If the error occurs before / when receiving the raw UDP packet, the `Option<Vec<u8>>` will be `None`. Otherwise, it will be `Some(Vec<u8>)` containing the received raw UDP packet.
    pub async fn recv_from(
        &self,
    ) -> Result<(Bytes, UdpHeader, SocketAddr), (Socks5Error, Option<Vec<u8>>)> {
        let max_pkt_size = self.buf_size.load(Ordering::Acquire);
        let mut buf = vec![0; max_pkt_size];

        let (len, addr) = match self.socket.recv_from(&mut buf).await {
            Ok(res) => res,
            Err(err) => return Err((Socks5Error::Io(err), None)),
        };

        buf.truncate(len);

        let header = match UdpHeader::read_from(&mut Cursor::new(buf.as_slice())).await {
            Ok(header) => header,
            Err(err) => return Err((err, Some(buf))),
        };

        let pkt = Bytes::from(buf).slice(header.serialized_len()..);

        Ok((pkt, header, addr))
    }

    /// Sends a UDP packet to the remote address which it is connected. The socks5 UDP header will be added to the packet.
    pub async fn send<P: AsRef<[u8]>>(&self, pkt: P, header: &UdpHeader) -> Result<usize, Error> {
        let mut buf = BytesMut::with_capacity(header.serialized_len() + pkt.as_ref().len());
        header.write_to_buf(&mut buf);
        buf.extend_from_slice(pkt.as_ref());

        self.socket
            .send(&buf)
            .await
            .map(|len| len - header.serialized_len())
    }

    /// Sends a UDP packet to a specified remote address. The socks5 UDP header will be added to the packet.
    pub async fn send_to<P: AsRef<[u8]>>(
        &self,
        pkt: P,
        header: &UdpHeader,
        addr: SocketAddr,
    ) -> Result<usize, Error> {
        let mut buf = BytesMut::with_capacity(header.serialized_len() + pkt.as_ref().len());
        header.write_to_buf(&mut buf);
        buf.extend_from_slice(pkt.as_ref());

        self.socket
            .send_to(&buf, addr)
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

impl From<AssociatedUdpSocket> for (UdpSocket, usize) {
    #[inline]
    fn from(from: AssociatedUdpSocket) -> Self {
        (from.socket, from.buf_size.load(Ordering::Relaxed))
    }
}

impl AsRef<UdpSocket> for AssociatedUdpSocket {
    #[inline]
    fn as_ref(&self) -> &UdpSocket {
        &self.socket
    }
}

impl AsMut<UdpSocket> for AssociatedUdpSocket {
    #[inline]
    fn as_mut(&mut self) -> &mut UdpSocket {
        &mut self.socket
    }
}
