use crate::{Address, Error, Reply};
use bytes::{BufMut, BytesMut};
use std::io::Result as IoResult;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Response
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
    pub fn new(reply: Reply, address: Address) -> Self {
        Self { reply, address }
    }

    pub async fn read_from<R>(r: &mut R) -> Result<Self, Error>
    where
        R: AsyncRead + Unpin,
    {
        let ver = r.read_u8().await?;

        if r.read_u8().await? != crate::SOCKS_VERSION {
            return Err(Error::UnsupportedSocks5Version(ver));
        }

        let mut buf = [0; 2];
        r.read_exact(&mut buf).await?;

        let reply = Reply::try_from(buf[0])?;
        let address = Address::read_from(r).await?;

        Ok(Self { reply, address })
    }

    pub async fn write_to<W>(&self, w: &mut W) -> IoResult<()>
    where
        W: AsyncWrite + Unpin,
    {
        let mut buf = BytesMut::with_capacity(self.serialized_len());
        self.write_to_buf(&mut buf);
        w.write_all(&buf).await
    }

    pub fn write_to_buf<B: BufMut>(&self, buf: &mut B) {
        buf.put_u8(crate::SOCKS_VERSION);
        buf.put_u8(u8::from(self.reply));
        buf.put_u8(0x00);
        self.address.write_to_buf(buf);
    }

    pub fn serialized_len(&self) -> usize {
        3 + self.address.serialized_len()
    }
}
