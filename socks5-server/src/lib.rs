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

pub struct Server<A> {
    listener: TcpListener,
    auth: Arc<A>,
}

impl<A> Server<A>
where
    A: Auth + Send + 'static,
{
    pub fn new(listener: TcpListener, auth: A) -> Self {
        let auth = Arc::new(auth);
        Server { listener, auth }
    }

    pub async fn bind<T: ToSocketAddrs>(addr: T, auth: A) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Server::new(listener, auth))
    }

    pub async fn accept(&self) -> Result<(IncomingConnection<A>, SocketAddr)> {
        let (stream, addr) = self.listener.accept().await?;
        Ok((IncomingConnection::new(stream, self.auth.clone()), addr))
    }
}
