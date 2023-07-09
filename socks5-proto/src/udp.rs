use crate::{address::AddressError, Address, Error, ProtocolError};
use bytes::{BufMut, BytesMut};
use std::io::Error as IoError;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// SOCKS5 UDP packet header
///
/// ```plain
/// +-----+------+------+----------+----------+----------+
/// | RSV | FRAG | ATYP | DST.ADDR | DST.PORT |   DATA   |
/// +-----+------+------+----------+----------+----------+
/// |  2  |  1   |  1   | Variable |    2     | Variable |
/// +-----+------+------+----------+----------+----------+
/// ```
#[derive(Clone, Debug)]
pub struct UdpHeader {
    pub frag: u8,
    pub address: Address,
}

impl UdpHeader {
    pub const fn new(frag: u8, address: Address) -> Self {
        Self { frag, address }
    }

    pub async fn read_from<R>(r: &mut R) -> Result<Self, Error>
    where
        R: AsyncRead + Unpin,
    {
        r.read_exact(&mut [0; 2]).await?;

        let frag = r.read_u8().await?;

        let addr = Address::read_from(r).await.map_err(|err| match err {
            AddressError::Io(err) => Error::Io(err),
            AddressError::InvalidType(code) => {
                Error::Protocol(ProtocolError::InvalidAddressTypeInUdpHeader {
                    frag,
                    address_type: code,
                })
            }
        })?;

        Ok(Self::new(frag, addr))
    }

    pub async fn write_to<W>(&self, w: &mut W) -> Result<(), IoError>
    where
        W: AsyncWrite + Unpin,
    {
        let mut buf = BytesMut::with_capacity(self.serialized_len());
        self.write_to_buf(&mut buf);
        w.write_all(&buf).await?;

        Ok(())
    }

    pub fn write_to_buf<B: BufMut>(&self, buf: &mut B) {
        buf.put_bytes(0x00, 2);
        buf.put_u8(self.frag);
        self.address.write_to_buf(buf);
    }

    pub fn serialized_len(&self) -> usize {
        2 + 1 + self.address.serialized_len()
    }
}
