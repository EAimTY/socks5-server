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
    const SUCCEEDED: u8 = 0x00;
    const GENERAL_FAILURE: u8 = 0x01;
    const CONNECTION_NOT_ALLOWED: u8 = 0x02;
    const NETWORK_UNREACHABLE: u8 = 0x03;
    const HOST_UNREACHABLE: u8 = 0x04;
    const CONNECTION_REFUSED: u8 = 0x05;
    const TTL_EXPIRED: u8 = 0x06;
    const COMMAND_NOT_SUPPORTED: u8 = 0x07;
    const ADDRESS_TYPE_NOT_SUPPORTED: u8 = 0x08;
}

impl TryFrom<u8> for Reply {
    type Error = u8;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        match code {
            Self::SUCCEEDED => Ok(Self::Succeeded),
            Self::GENERAL_FAILURE => Ok(Self::GeneralFailure),
            Self::CONNECTION_NOT_ALLOWED => Ok(Self::ConnectionNotAllowed),
            Self::NETWORK_UNREACHABLE => Ok(Self::NetworkUnreachable),
            Self::HOST_UNREACHABLE => Ok(Self::HostUnreachable),
            Self::CONNECTION_REFUSED => Ok(Self::ConnectionRefused),
            Self::TTL_EXPIRED => Ok(Self::TtlExpired),
            Self::COMMAND_NOT_SUPPORTED => Ok(Self::CommandNotSupported),
            Self::ADDRESS_TYPE_NOT_SUPPORTED => Ok(Self::AddressTypeNotSupported),
            code => Err(code),
        }
    }
}

impl From<Reply> for u8 {
    fn from(reply: Reply) -> Self {
        match reply {
            Reply::Succeeded => Reply::SUCCEEDED,
            Reply::GeneralFailure => Reply::GENERAL_FAILURE,
            Reply::ConnectionNotAllowed => Reply::CONNECTION_NOT_ALLOWED,
            Reply::NetworkUnreachable => Reply::NETWORK_UNREACHABLE,
            Reply::HostUnreachable => Reply::HOST_UNREACHABLE,
            Reply::ConnectionRefused => Reply::CONNECTION_REFUSED,
            Reply::TtlExpired => Reply::TTL_EXPIRED,
            Reply::CommandNotSupported => Reply::COMMAND_NOT_SUPPORTED,
            Reply::AddressTypeNotSupported => Reply::ADDRESS_TYPE_NOT_SUPPORTED,
        }
    }
}
