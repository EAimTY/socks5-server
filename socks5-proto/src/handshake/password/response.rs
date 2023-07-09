use super::Error;
use bytes::{BufMut, BytesMut};
use std::io::Error as IoError;
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
    const FAILED: u8 = 0xff;
    const SUCCEEDED: u8 = 0x00;

    pub fn new(status: bool) -> Self {
        Self { status }
    }

    pub async fn read_from<R>(r: &mut R) -> Result<Self, Error>
    where
        R: AsyncRead + Unpin,
    {
        let ver = r.read_u8().await?;

        if ver != super::SUBNEGOTIATION_VERSION {
            return Err(Error::SubNegotiationVersion { version: ver });
        }

        let status = match r.read_u8().await? {
            Self::FAILED => false,
            Self::SUCCEEDED => true,
            code => {
                return Err(Error::SubNegotiationStatus {
                    version: ver,
                    status: code,
                });
            }
        };

        Ok(Self::new(status))
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
        buf.put_u8(super::SUBNEGOTIATION_VERSION);

        if self.status {
            buf.put_u8(Self::SUCCEEDED);
        } else {
            buf.put_u8(Self::FAILED);
        }
    }

    pub fn serialized_len(&self) -> usize {
        1 + 1
    }
}
