use std::io::{Error, ErrorKind};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Command {
    Connect,
    Bind,
    Associate,
}

impl Command {
    const CMD_CONNECT: u8 = 0x01;
    const CMD_BIND: u8 = 0x02;
    const CMD_ASSOCIATE: u8 = 0x03;
}

impl TryFrom<u8> for Command {
    type Error = Error;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        match code {
            Self::CMD_CONNECT => Ok(Command::Connect),
            Self::CMD_BIND => Ok(Command::Bind),
            Self::CMD_ASSOCIATE => Ok(Command::Associate),
            code => Err(Error::new(
                ErrorKind::Unsupported,
                format!("Unsupported command {0:#x}", code),
            )),
        }
    }
}

impl From<Command> for u8 {
    fn from(cmd: Command) -> Self {
        match cmd {
            Command::Connect => Command::CMD_CONNECT,
            Command::Bind => Command::CMD_BIND,
            Command::Associate => Command::CMD_ASSOCIATE,
        }
    }
}
