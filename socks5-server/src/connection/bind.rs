use socks5_proto::{Address, Reply, Response};
use std::io::Result;
use tokio::net::TcpStream;

pub struct Bind<S> {
    stream: TcpStream,
    _state: S,
}

pub struct NeedFirstReply;
pub struct NeedSecondReply;
pub struct Ready;

impl Bind<NeedFirstReply> {
    pub(super) fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _state: NeedFirstReply,
        }
    }

    pub async fn reply(mut self, reply: Reply, addr: Address) -> Result<Bind<NeedSecondReply>> {
        let resp = Response::new(reply, addr);
        resp.write_to(&mut self.stream).await?;
        Ok(Bind::<NeedSecondReply>::new(self.stream))
    }
}

impl Bind<NeedSecondReply> {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _state: NeedSecondReply,
        }
    }

    pub async fn reply(mut self, reply: Reply, addr: Address) -> Result<Bind<Ready>> {
        let resp = Response::new(reply, addr);
        resp.write_to(&mut self.stream).await?;
        Ok(Bind::<Ready>::new(self.stream))
    }
}

impl Bind<Ready> {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _state: Ready,
        }
    }
}
