use byteorder::{BigEndian, ReadBytesExt};
use bytes::BufMut;
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    io::{Cursor, Error, ErrorKind, Result},
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    vec,
};
use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Address {
    SocketAddress(SocketAddr),
    DomainAddress(String, u16),
}

impl Address {
    const ATYP_IPV4: u8 = 0x01;
    const ATYP_FQDN: u8 = 0x03;
    const ATYP_IPV6: u8 = 0x04;

    pub fn unspecified() -> Self {
        Address::SocketAddress(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))
    }

    pub fn to_socket_addr(&self) -> Result<SocketAddr> {
        match self {
            Address::SocketAddress(addr) => Ok(*addr),
            Address::DomainAddress(addr, port) => {
                if let Ok(addr) = addr.parse::<Ipv4Addr>() {
                    Ok(SocketAddr::from((addr, *port)))
                } else if let Ok(addr) = addr.parse::<Ipv6Addr>() {
                    Ok(SocketAddr::from((addr, *port)))
                } else {
                    Err(Error::new(
                        ErrorKind::Unsupported,
                        format!("domain address {addr} is not supported"),
                    ))
                }
            }
        }
    }

    pub async fn read_from<R>(stream: &mut R) -> Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let atyp = stream.read_u8().await?;

        match atyp {
            Self::ATYP_IPV4 => {
                let mut buf = [0; 6];
                stream.read_exact(&mut buf).await?;
                let mut rdr = Cursor::new(buf);

                let addr = Ipv4Addr::new(
                    ReadBytesExt::read_u8(&mut rdr).unwrap(),
                    ReadBytesExt::read_u8(&mut rdr).unwrap(),
                    ReadBytesExt::read_u8(&mut rdr).unwrap(),
                    ReadBytesExt::read_u8(&mut rdr).unwrap(),
                );

                let port = ReadBytesExt::read_u16::<BigEndian>(&mut rdr).unwrap();

                Ok(Self::SocketAddress(SocketAddr::from((addr, port))))
            }
            Self::ATYP_FQDN => {
                let len = stream.read_u8().await? as usize;

                let mut buf = vec![0; len + 2];
                stream.read_exact(&mut buf).await?;

                let port = ReadBytesExt::read_u16::<BigEndian>(&mut &buf[len..]).unwrap();
                buf.truncate(len);

                let addr = match String::from_utf8(buf) {
                    Ok(addr) => addr,
                    Err(err) => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            format!("Invalid address encoding: {err}"),
                        ))
                    }
                };

                Ok(Self::DomainAddress(addr, port))
            }
            Self::ATYP_IPV6 => {
                let mut buf = [0; 18];
                stream.read_exact(&mut buf).await?;
                let mut rdr = Cursor::new(buf);

                let addr = Ipv6Addr::new(
                    ReadBytesExt::read_u16::<BigEndian>(&mut rdr).unwrap(),
                    ReadBytesExt::read_u16::<BigEndian>(&mut rdr).unwrap(),
                    ReadBytesExt::read_u16::<BigEndian>(&mut rdr).unwrap(),
                    ReadBytesExt::read_u16::<BigEndian>(&mut rdr).unwrap(),
                    ReadBytesExt::read_u16::<BigEndian>(&mut rdr).unwrap(),
                    ReadBytesExt::read_u16::<BigEndian>(&mut rdr).unwrap(),
                    ReadBytesExt::read_u16::<BigEndian>(&mut rdr).unwrap(),
                    ReadBytesExt::read_u16::<BigEndian>(&mut rdr).unwrap(),
                );

                let port = ReadBytesExt::read_u16::<BigEndian>(&mut rdr).unwrap();

                Ok(Self::SocketAddress(SocketAddr::from((addr, port))))
            }
            atyp => Err(Error::new(
                ErrorKind::Unsupported,
                format!("Unsupported address type {0:#x}", atyp),
            )),
        }
    }

    pub fn write_to_buf<B: BufMut>(&self, buf: &mut B) {
        match self {
            Self::SocketAddress(addr) => match addr {
                SocketAddr::V4(addr) => {
                    buf.put_u8(Self::ATYP_IPV4);
                    buf.put_slice(&addr.ip().octets());
                    buf.put_u16(addr.port());
                }
                SocketAddr::V6(addr) => {
                    buf.put_u8(Self::ATYP_IPV6);
                    for seg in addr.ip().segments() {
                        buf.put_u16(seg);
                    }
                    buf.put_u16(addr.port());
                }
            },
            Self::DomainAddress(addr, port) => {
                buf.put_u8(Self::ATYP_FQDN);
                buf.put_u8(addr.len() as u8);
                buf.put_slice(addr.as_bytes());
                buf.put_u16(*port);
            }
        }
    }

    pub fn serialized_len(&self) -> usize {
        1 + match self {
            Address::SocketAddress(addr) => match addr {
                SocketAddr::V4(_) => 6,
                SocketAddr::V6(_) => 18,
            },
            Address::DomainAddress(addr, _) => 1 + addr.len() + 2,
        }
    }

    pub const fn max_serialized_len() -> usize {
        1 + 1 + u8::MAX as usize + 2
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Address::DomainAddress(hostname, port) => write!(f, "{hostname}:{port}"),
            Address::SocketAddress(socket_addr) => write!(f, "{socket_addr}"),
        }
    }
}

impl From<SocketAddr> for Address {
    fn from(addr: SocketAddr) -> Self {
        Address::SocketAddress(addr)
    }
}

impl From<(Ipv4Addr, u16)> for Address {
    fn from((addr, port): (Ipv4Addr, u16)) -> Self {
        Address::SocketAddress(SocketAddr::from((addr, port)))
    }
}

impl From<(Ipv6Addr, u16)> for Address {
    fn from((addr, port): (Ipv6Addr, u16)) -> Self {
        Address::SocketAddress(SocketAddr::from((addr, port)))
    }
}

impl From<(String, u16)> for Address {
    fn from((addr, port): (String, u16)) -> Self {
        Address::DomainAddress(addr, port)
    }
}

impl From<(&str, u16)> for Address {
    fn from((addr, port): (&str, u16)) -> Self {
        Address::DomainAddress(addr.to_owned(), port)
    }
}
