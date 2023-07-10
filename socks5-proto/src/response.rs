use crate::{address::AddressError, Address, Error, ProtocolError, Reply};
use bytes::{BufMut, BytesMut};
use std::io::Error as IoError;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// SOCKS5 response
///
/// ```plain
/// +-----+-----+-------+------+----------+----------+
/// | VER | REP |  RSV  | ATYP | BND.ADDR | BND.PORT |
/// +-----+-----+-------+------+----------+----------+
/// |  1  |  1  | X'00' |  1   | Variable |    2     |
/// +-----+-----+-------+------+----------+----------+
/// ```
#[derive(Clone, Debug)]
pub struct Response {
    pub reply: Reply,
    pub address: Address,
}

impl Response {
    pub const fn new(reply: Reply, address: Address) -> Self {
        Self { reply, address }
    }

    pub async fn read_from<R>(r: &mut R) -> Result<Self, Error>
    where
        R: AsyncRead + Unpin,
    {
        let ver = r.read_u8().await?;

        if ver != crate::SOCKS_VERSION {
            return Err(Error::Protocol(ProtocolError::ProtocolVersion {
                version: ver,
            }));
        }

        let rep = r.read_u8().await?;
        let rep = Reply::try_from(rep).map_err(|rep| ProtocolError::InvalidReply {
            version: ver,
            reply: rep,
        })?;

        let _ = r.read_u8().await?;

        let addr = Address::read_from(r).await.map_err(|err| match err {
            AddressError::Io(err) => Error::Io(err),
            AddressError::InvalidType(code) => {
                Error::Protocol(ProtocolError::InvalidAddressTypeInResponse {
                    version: ver,
                    reply: rep,
                    address_type: code,
                })
            }
        })?;

        Ok(Self::new(rep, addr))
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
        buf.put_u8(crate::SOCKS_VERSION);
        buf.put_u8(u8::from(self.reply));
        buf.put_u8(0x00);
        self.address.write_to_buf(buf);
    }

    pub fn serialized_len(&self) -> usize {
        1 + 1 + 1 + self.address.serialized_len()
    }
}
