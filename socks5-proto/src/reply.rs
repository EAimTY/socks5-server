use std::io::{Error, ErrorKind};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Reply {
    Succeeded,
    GeneralFailure,
    ConnectionNotAllowed,
    NetworkUnreachable,
    HostUnreachable,
    ConnectionRefused,
    TtlExpired,
    CommandNotSupported,
    AddressTypeNotSupported,
}

impl Reply {
    const REPLY_SUCCEEDED: u8 = 0x00;
    const REPLY_GENERAL_FAILURE: u8 = 0x01;
    const REPLY_CONNECTION_NOT_ALLOWED: u8 = 0x02;
    const REPLY_NETWORK_UNREACHABLE: u8 = 0x03;
    const REPLY_HOST_UNREACHABLE: u8 = 0x04;
    const REPLY_CONNECTION_REFUSED: u8 = 0x05;
    const REPLY_TTL_EXPIRED: u8 = 0x06;
    const REPLY_COMMAND_NOT_SUPPORTED: u8 = 0x07;
    const REPLY_ADDRESS_TYPE_NOT_SUPPORTED: u8 = 0x08;
}

impl TryFrom<u8> for Reply {
    type Error = Error;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        match code {
            Self::REPLY_SUCCEEDED => Ok(Reply::Succeeded),
            Self::REPLY_GENERAL_FAILURE => Ok(Reply::GeneralFailure),
            Self::REPLY_CONNECTION_NOT_ALLOWED => Ok(Reply::ConnectionNotAllowed),
            Self::REPLY_NETWORK_UNREACHABLE => Ok(Reply::NetworkUnreachable),
            Self::REPLY_HOST_UNREACHABLE => Ok(Reply::HostUnreachable),
            Self::REPLY_CONNECTION_REFUSED => Ok(Reply::ConnectionRefused),
            Self::REPLY_TTL_EXPIRED => Ok(Reply::TtlExpired),
            Self::REPLY_COMMAND_NOT_SUPPORTED => Ok(Reply::CommandNotSupported),
            Self::REPLY_ADDRESS_TYPE_NOT_SUPPORTED => Ok(Reply::AddressTypeNotSupported),
            code => Err(Error::new(
                ErrorKind::InvalidData,
                format!("Invalid reply {0:#x}", code),
            )),
        }
    }
}

impl From<Reply> for u8 {
    fn from(reply: Reply) -> Self {
        match reply {
            Reply::Succeeded => Reply::REPLY_SUCCEEDED,
            Reply::GeneralFailure => Reply::REPLY_GENERAL_FAILURE,
            Reply::ConnectionNotAllowed => Reply::REPLY_CONNECTION_NOT_ALLOWED,
            Reply::NetworkUnreachable => Reply::REPLY_NETWORK_UNREACHABLE,
            Reply::HostUnreachable => Reply::REPLY_HOST_UNREACHABLE,
            Reply::ConnectionRefused => Reply::REPLY_CONNECTION_REFUSED,
            Reply::TtlExpired => Reply::REPLY_TTL_EXPIRED,
            Reply::CommandNotSupported => Reply::REPLY_COMMAND_NOT_SUPPORTED,
            Reply::AddressTypeNotSupported => Reply::REPLY_ADDRESS_TYPE_NOT_SUPPORTED,
        }
    }
}
