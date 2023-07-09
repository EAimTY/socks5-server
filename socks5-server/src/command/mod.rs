use self::{associate::Associate, bind::Bind, connect::Connect};
use crate::Auth;
use socks5_proto::{
    Address, Command as ProtocolCommand, HandshakeMethod, HandshakeRequest, HandshakeResponse,
    Request,
};
use std::{
    io::{Error, ErrorKind},
    sync::Arc,
};
use tokio::net::TcpStream;

pub mod associate;
pub mod bind;
pub mod connect;

pub struct Authenticating<O> {
    stream: TcpStream,
    auth: Arc<dyn Auth<Output = O> + Send + Sync>,
}

impl<O> Authenticating<O> {
    #[inline]
    pub(crate) fn new(stream: TcpStream, auth: Arc<dyn Auth<Output = O> + Send + Sync>) -> Self {
        Self { stream, auth }
    }

    pub async fn authenticate(mut self) -> Result<(WaitingCommand, O), (TcpStream, Error)> {
        let req = match HandshakeRequest::read_from(&mut self.stream).await {
            Ok(req) => req,
            Err(err) => return Err((self.stream, err)),
        };
        let chosen_method = self.auth.as_handshake_method();

        if req.methods.contains(&chosen_method) {
            let resp = HandshakeResponse::new(chosen_method);

            if let Err(err) = resp.write_to(&mut self.stream).await {
                return Err((self.stream, err));
            }

            let output = match self.auth.execute(&mut self.stream).await {
                Ok(req) => req,
                Err(err) => return Err((self.stream, err)),
            };

            Ok((WaitingCommand::new(self.stream), output))
        } else {
            let resp = HandshakeResponse::new(HandshakeMethod::Unacceptable);

            if let Err(err) = resp.write_to(&mut self.stream).await {
                return Err((self.stream, err));
            }

            let err = Error::new(
                ErrorKind::Unsupported,
                "No available handshake method provided by client",
            );

            Err((self.stream, err))
        }
    }
}

pub struct WaitingCommand(TcpStream);

impl WaitingCommand {
    #[inline]
    fn new(stream: TcpStream) -> Self {
        Self(stream)
    }

    pub async fn wait_request(mut self) -> Result<Command, (TcpStream, Error)> {
        let req = match Request::read_from(&mut self.0).await {
            Ok(req) => req,
            Err(err) => return Err((self.0, err)),
        };

        match req.command {
            ProtocolCommand::Associate => Ok(Command::Associate(
                Associate::<associate::NeedReply>::new(self.0),
                req.address,
            )),
            ProtocolCommand::Bind => Ok(Command::Bind(
                Bind::<bind::NeedFirstReply>::new(self.0),
                req.address,
            )),
            ProtocolCommand::Connect => Ok(Command::Connect(
                Connect::<connect::NeedReply>::new(self.0),
                req.address,
            )),
        }
    }
}

pub enum Command {
    Associate(Associate<associate::NeedReply>, Address),
    Bind(Bind<bind::NeedFirstReply>, Address),
    Connect(Connect<connect::NeedReply>, Address),
}
