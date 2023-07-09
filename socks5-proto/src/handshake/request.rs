use super::Method;
use crate::{Error, ProtocolError};
use bytes::{BufMut, BytesMut};
use std::{
    io::Error as IoError,
    mem::{self, ManuallyDrop},
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// SOCKS5 handshake request
///
/// ```plain
/// +-----+----------+----------+
/// | VER | NMETHODS | METHODS  |
/// +-----+----------+----------+
/// |  1  |    1     | 1 to 255 |
/// +-----+----------+----------|
/// ```
#[derive(Clone, Debug)]
pub struct Request {
    pub methods: Vec<Method>,
}

impl Request {
    pub const fn new(methods: Vec<Method>) -> Self {
        Self { methods }
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

        let mlen = r.read_u8().await?;
        let mut methods = vec![0; mlen as usize];
        r.read_exact(&mut methods).await?;

        let methods = unsafe {
            let mut methods = ManuallyDrop::new(methods);

            Vec::from_raw_parts(
                methods.as_mut_ptr() as *mut Method,
                methods.len(),
                methods.capacity(),
            )
        };

        Ok(Self::new(methods))
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
        buf.put_u8(self.methods.len() as u8);

        let methods = unsafe { mem::transmute(self.methods.as_slice()) };
        buf.put_slice(methods);
    }

    pub fn serialized_len(&self) -> usize {
        1 + 1 + self.methods.len()
    }
}
