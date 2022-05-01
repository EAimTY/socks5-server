#![doc = include_str!("../README.md")]

use std::{io::Result, net::SocketAddr, sync::Arc};
use tokio::net::{TcpListener, ToSocketAddrs};

pub mod auth;
pub mod connection;

pub use crate::{
    auth::Auth,
    connection::{
        associate::Associate, bind::Bind, connect::Connect, Connection, IncomingConnection,
    },
};

pub struct Server {
    listener: TcpListener,
    auth: Arc<dyn Auth + Send + Sync + 'static>,
}

impl Server {
    pub fn new(listener: TcpListener, auth: Arc<dyn Auth + Send + Sync + 'static>) -> Self {
        Server { listener, auth }
    }

    pub async fn bind<T: ToSocketAddrs>(
        addr: T,
        auth: Arc<dyn Auth + Send + Sync + 'static>,
    ) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Server::new(listener, auth))
    }

    pub async fn accept(&self) -> Result<(IncomingConnection, SocketAddr)> {
        let (stream, addr) = self.listener.accept().await?;
        Ok((IncomingConnection::new(stream, self.auth.clone()), addr))
    }
}
