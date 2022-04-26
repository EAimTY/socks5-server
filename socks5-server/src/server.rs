use crate::{Auth, IncomingConnection};
use std::{io::Result, sync::Arc};
use tokio::net::{TcpListener, ToSocketAddrs};

pub struct Server {
    listener: TcpListener,
    auth: Arc<dyn Auth>,
}

impl Server {
    pub fn new(listener: TcpListener, auth: Arc<dyn Auth>) -> Self {
        Server { listener, auth }
    }

    pub async fn bind<A: ToSocketAddrs>(addr: A, auth: Arc<dyn Auth>) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Server::new(listener, auth))
    }

    pub async fn accept(&self) -> Result<IncomingConnection> {
        let (stream, _) = self.listener.accept().await?;
        Ok(IncomingConnection::new(stream, self.auth.clone()))
    }
}
