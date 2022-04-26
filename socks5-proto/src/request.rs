use crate::{Address, Command};
use bytes::{BufMut, BytesMut};
use std::io::{Error, ErrorKind, Result};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// SOCKS5 request
///
/// ```plain
/// +-----+-----+-------+------+----------+----------+
/// | VER | CMD |  RSV  | ATYP | DST.ADDR | DST.PORT |
/// +-----+-----+-------+------+----------+----------+
/// |  1  |  1  | X'00' |  1   | Variable |    2     |
/// +-----+-----+-------+------+----------+----------+
/// ```
#[derive(Clone, Debug)]
pub struct Request {
    pub command: Command,
    pub address: Address,
}

impl Request {
    pub fn new(command: Command, address: Address) -> Self {
        Self { command, address }
    }

    pub async fn read_from<R>(r: &mut R) -> Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let ver = r.read_u8().await?;

        if ver != crate::SOCKS_VERSION {
            return Err(Error::new(
                ErrorKind::Unsupported,
                format!("Unsupported SOCKS version {0:#x}", ver),
            ));
        }

        let mut buf = [0; 2];
        r.read_exact(&mut buf).await?;

        let command = Command::try_from(buf[0])?;
        let address = Address::read_from(r).await?;

        Ok(Self { command, address })
    }

    pub async fn write_to<W>(&self, w: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        let mut buf = BytesMut::with_capacity(self.serialized_len());
        self.write_to_buf(&mut buf);
        w.write_all(&buf).await
    }

    pub fn write_to_buf<B: BufMut>(&self, buf: &mut B) {
        buf.put_u8(crate::SOCKS_VERSION);
        buf.put_u8(u8::from(self.command));
        buf.put_u8(0x00);
        self.address.write_to_buf(buf);
    }

    pub fn serialized_len(&self) -> usize {
        3 + self.address.serialized_len()
    }
}
