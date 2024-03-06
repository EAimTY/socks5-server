use bytes::BufMut;
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IoError,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    vec,
};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt};

/// SOCKS5 address
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Address {
    SocketAddress(SocketAddr),
    DomainAddress(Vec<u8>, u16),
}

impl Address {
    const ATYP_IPV4: u8 = 0x01;
    const ATYP_FQDN: u8 = 0x03;
    const ATYP_IPV6: u8 = 0x04;

    pub(crate) async fn read_from<R>(stream: &mut R) -> Result<Self, AddressError>
    where
        R: AsyncRead + Unpin,
    {
        let atyp = stream.read_u8().await?;

        match atyp {
            Self::ATYP_IPV4 => {
                let mut buf = [0; 6];
                stream.read_exact(&mut buf).await?;

                let addr = Ipv4Addr::new(buf[0], buf[1], buf[2], buf[3]);

                let port = u16::from_be_bytes([buf[4], buf[5]]);

                Ok(Self::SocketAddress(SocketAddr::from((addr, port))))
            }
            Self::ATYP_FQDN => {
                let len = stream.read_u8().await? as usize;

                let mut buf = vec![0; len + 2];
                stream.read_exact(&mut buf).await?;

                let port = u16::from_be_bytes([buf[len], buf[len + 1]]);
                buf.truncate(len);

                Ok(Self::DomainAddress(buf, port))
            }
            Self::ATYP_IPV6 => {
                let mut buf = [0; 18];
                stream.read_exact(&mut buf).await?;

                let addr = Ipv6Addr::new(
                    u16::from_be_bytes([buf[0], buf[1]]),
                    u16::from_be_bytes([buf[2], buf[3]]),
                    u16::from_be_bytes([buf[4], buf[5]]),
                    u16::from_be_bytes([buf[6], buf[7]]),
                    u16::from_be_bytes([buf[8], buf[9]]),
                    u16::from_be_bytes([buf[10], buf[11]]),
                    u16::from_be_bytes([buf[12], buf[13]]),
                    u16::from_be_bytes([buf[14], buf[15]]),
                );

                let port = u16::from_be_bytes([buf[16], buf[17]]);

                Ok(Self::SocketAddress(SocketAddr::from((addr, port))))
            }
            atyp => Err(AddressError::InvalidType(atyp)),
        }
    }

    pub(crate) fn write_to_buf<B: BufMut>(&self, buf: &mut B) {
        match self {
            Self::SocketAddress(SocketAddr::V4(addr)) => {
                buf.put_u8(Self::ATYP_IPV4);
                buf.put_slice(&addr.ip().octets());
                buf.put_u16(addr.port());
            }
            Self::SocketAddress(SocketAddr::V6(addr)) => {
                buf.put_u8(Self::ATYP_IPV6);
                for seg in addr.ip().segments() {
                    buf.put_u16(seg);
                }
                buf.put_u16(addr.port());
            }
            Self::DomainAddress(addr, port) => {
                buf.put_u8(Self::ATYP_FQDN);
                buf.put_u8(addr.len() as u8);
                buf.put_slice(addr);
                buf.put_u16(*port);
            }
        }
    }

    pub fn unspecified() -> Self {
        Address::SocketAddress(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0))
    }

    pub fn serialized_len(&self) -> usize {
        1 + match self {
            Address::SocketAddress(SocketAddr::V4(_)) => 6,
            Address::SocketAddress(SocketAddr::V6(_)) => 18,
            Address::DomainAddress(addr, _) => 1 + addr.len() + 2,
        }
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Address::DomainAddress(hostname, port) => write!(
                f,
                "{hostname}:{port}",
                hostname = String::from_utf8_lossy(hostname),
            ),
            Address::SocketAddress(addr) => write!(f, "{addr}"),
        }
    }
}

#[derive(Debug, Error)]
pub(crate) enum AddressError {
    #[error(transparent)]
    Io(#[from] IoError),
    #[error("Invalid address type {0:#04x}")]
    InvalidType(u8),
}
