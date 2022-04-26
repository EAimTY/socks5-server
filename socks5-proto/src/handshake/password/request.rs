use bytes::{BufMut, BytesMut};
use std::io::{Error, ErrorKind, Result};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// SOCKS5 password handshake request
///
/// ```plain
/// +-----+------+----------+------+----------+
/// | VER | ULEN |  UNAME   | PLEN |  PASSWD  |
/// +-----+------+----------+------+----------+
/// |  1  |  1   | 1 to 255 |  1   | 1 to 255 |
/// +-----+------+----------+------+----------+
/// ```

#[derive(Clone, Debug)]
pub struct Request {
    pub username: Vec<u8>,
    pub password: Vec<u8>,
}

impl Request {
    pub fn new(username: Vec<u8>, password: Vec<u8>) -> Self {
        Self { username, password }
    }

    pub async fn read_from<R>(r: &mut R) -> Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let ver = r.read_u8().await?;

        if ver != super::SUBNEGOTIATION_VERSION {
            return Err(Error::new(
                ErrorKind::Unsupported,
                format!("Unsupported sub-negotiation version {0:#x}", ver),
            ));
        }

        let ulen = r.read_u8().await?;
        let mut buf = vec![0; ulen as usize + 1];
        r.read_exact(&mut buf).await?;

        let plen = buf[ulen as usize];
        buf.truncate(ulen as usize);
        let username = buf;

        let mut password = vec![0; plen as usize];
        r.read_exact(&mut password).await?;

        Ok(Self { username, password })
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
        buf.put_u8(super::SUBNEGOTIATION_VERSION);

        buf.put_u8(self.username.len() as u8);
        buf.put_slice(&self.username);

        buf.put_u8(self.password.len() as u8);
        buf.put_slice(&self.password);
    }

    pub fn serialized_len(&self) -> usize {
        3 + self.username.len() + self.password.len()
    }
}
