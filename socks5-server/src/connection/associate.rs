//! Socks5 command type `Associate`
//!
//! This module also provides an [`tokio::net::UdpSocket`] wrapper [`AssociatedUdpSocket`], which can be used to send and receive UDP packets without dealing with the SOCKS5 protocol UDP header.

use bytes::{Bytes, BytesMut};
use socks5_proto::{Address, Error as Socks5Error, Reply, Response, UdpHeader};
use std::{
    io::{Cursor, Error},
    marker::PhantomData,
    net::SocketAddr,
    sync::atomic::{AtomicUsize, Ordering},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, UdpSocket},
};

/// Connection state types
pub mod state {
    #[derive(Debug)]
    pub struct NeedReply;

    #[derive(Debug)]
    pub struct Ready;
}

/// Socks5 command type `Associate`
///
/// Reply the client with [`Associate::reply()`] to complete the command negotiation.
#[derive(Debug)]
pub struct Associate<S> {
    stream: TcpStream,
    _state: PhantomData<S>,
}

impl Associate<state::NeedReply> {
    /// Reply to the SOCKS5 client with the given reply and address.
    ///
    /// If encountered an error while writing the reply, the error alongside the original `TcpStream` is returned.
    pub async fn reply(
        mut self,
        reply: Reply,
        addr: Address,
    ) -> Result<Associate<state::Ready>, (Error, TcpStream)> {
        let resp = Response::new(reply, addr);

        if let Err(err) = resp.write_to(&mut self.stream).await {
            return Err((err, self.stream));
        }

        Ok(Associate::new(self.stream))
    }
}

impl Associate<state::Ready> {
    /// Wait until the SOCKS5 client closes this TCP connection.
    ///
    /// Socks5 protocol defines that when the client closes the TCP connection used to send the associate command, the server should release the associated UDP socket.
    pub async fn wait_close(&mut self) -> Result<(), Error> {
        loop {
            match self.stream.read(&mut [0]).await {
                Ok(0) => break Ok(()),
                Ok(_) => {}
                Err(err) => break Err(err),
            }
        }
    }
}

impl<S> Associate<S> {
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

    /// Consumes the [`Associate<S>`] and returns the underlying [`TcpStream`](tokio::net::TcpStream).
    #[inline]
    pub fn into_inner(self) -> TcpStream {
        self.stream
    }
}

/// A wrapper of a tokio UDP socket dealing with SOCKS5 UDP header.
///
/// It only provides handful of methods to send / receive UDP packets with SOCKS5 UDP header. The underlying `UdpSocket` can be accessed with [`AssociatedUdpSocket::get_ref()`] and [`AssociatedUdpSocket::get_mut()`].
#[derive(Debug)]
pub struct AssociatedUdpSocket {
    socket: UdpSocket,
    buf_size: AtomicUsize,
}

impl AssociatedUdpSocket {
    /// Creates a new [`AssociatedUdpSocket`] with a [`UdpSocket`](tokio::net::UdpSocket) and a maximum receiving UDP packet size, with SOCKS5 UDP header included.
    pub fn new(socket: UdpSocket, buf_size: usize) -> Self {
        Self {
            socket,
            buf_size: AtomicUsize::new(buf_size),
        }
    }

    /// Receives a SOCKS5 UDP packet on the socket from the remote address which it is connected.
    ///
    /// On success, it returns the packet payload and the SOCKS5 UDP header. On error, it returns the error alongside an `Option<Vec<u8>>`. If the error occurs before / when receiving the raw UDP packet, the `Option<Vec<u8>>` will be `None`. Otherwise, it will be `Some(Vec<u8>)` containing the received raw UDP packet.
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

    /// Receives a SOCKS5 UDP packet on the socket from a remote address.
    ///
    /// On success, it returns the packet payload, the SOCKS5 UDP header and the source address. On error, it returns the error alongside an `Option<Vec<u8>>`. If the error occurs before / when receiving the raw UDP packet, the `Option<Vec<u8>>` will be `None`. Otherwise, it will be `Some(Vec<u8>)` containing the received raw UDP packet.
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

    /// Sends a UDP packet to the remote address which it is connected. The SOCKS5 UDP header will be added to the packet.
    pub async fn send<P: AsRef<[u8]>>(&self, pkt: P, header: &UdpHeader) -> Result<usize, Error> {
        let mut buf = BytesMut::with_capacity(header.serialized_len() + pkt.as_ref().len());
        header.write_to_buf(&mut buf);
        buf.extend_from_slice(pkt.as_ref());

        self.socket
            .send(&buf)
            .await
            .map(|len| len - header.serialized_len())
    }

    /// Sends a UDP packet to a specified remote address. The SOCKS5 UDP header will be added to the packet.
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

    /// Get the maximum receiving UDP packet size, with SOCKS5 UDP header included.
    #[inline]
    pub fn get_max_pkt_size(&self) -> usize {
        self.buf_size.load(Ordering::Acquire)
    }

    /// Set the maximum receiving UDP packet size, with SOCKS5 UDP header included, for adjusting the receiving buffer size.
    #[inline]
    pub fn set_max_pkt_size(&self, size: usize) {
        self.buf_size.store(size, Ordering::Release);
    }

    /// Returns a shared reference to the underlying socket.
    ///
    /// Note that this may break the encapsulation of the SOCKS5 connection and you should not use this method unless you know what you are doing.
    #[inline]
    pub fn get_ref(&self) -> &UdpSocket {
        &self.socket
    }

    /// Returns a mutable reference to the underlying socket.
    ///
    /// Note that this may break the encapsulation of the SOCKS5 UDP abstraction and you should not use this method unless you know what you are doing.
    #[inline]
    pub fn get_mut(&mut self) -> &mut UdpSocket {
        &mut self.socket
    }

    /// Consumes the [`AssociatedUdpSocket`] and returns the underlying [`UdpSocket`](tokio::net::UdpSocket).
    #[inline]
    pub fn into_inner(self) -> UdpSocket {
        self.socket
    }
}
