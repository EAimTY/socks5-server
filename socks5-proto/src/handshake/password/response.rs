use bytes::{BufMut, BytesMut};
use std::io::{Error, ErrorKind, Result};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// SOCKS5 password handshake response
///
/// ```plain
/// +-----+--------+
/// | VER | STATUS |
/// +-----+--------+
/// |  1  |   1    |
/// +-----+--------+
/// ```

#[derive(Clone, Debug)]
pub struct Response {
    pub status: bool,
}

impl Response {
    const STATUS_FAILED: u8 = 0xff;
    const STATUS_SUCCEEDED: u8 = 0x00;

    pub fn new(status: bool) -> Self {
        Self { status }
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

        let status = match r.read_u8().await? {
            Self::STATUS_FAILED => false,
            Self::STATUS_SUCCEEDED => true,
            code => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Invalid sub-negotiation status {0:#x}", code),
                ))
            }
        };

        Ok(Self { status })
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

        if self.status {
            buf.put_u8(Self::STATUS_SUCCEEDED);
        } else {
            buf.put_u8(Self::STATUS_FAILED);
        }
    }

    pub fn serialized_len(&self) -> usize {
        2
    }
}
