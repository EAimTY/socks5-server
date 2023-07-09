use socks5_proto::{Address, Reply};
use socks5_server::{auth::NoAuth, command::Authenticating, Command, Server};
use std::{io::Result, sync::Arc};
use tokio::{
    io::{self, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::main]
async fn main() -> Result<()> {
    let server = Server::bind("127.0.0.1:5000", Arc::new(NoAuth)).await?;

    while let Ok((conn, _)) = server.accept().await {
        tokio::spawn(async move {
            match handle(conn).await {
                Ok(()) => {}
                Err(err) => eprintln!("{err}"),
            }
        });
    }

    Ok(())
}

async fn handle(conn: Authenticating<()>) -> Result<()> {
    let conn = match conn.authenticate().await {
        Ok((conn, _)) => conn,
        Err((mut conn, err)) => {
            let _ = conn.shutdown().await;
            return Err(err);
        }
    };

    match conn.wait_request().await {
        Ok(Command::Associate(associate, _)) => {
            let mut conn = associate
                .reply(Reply::CommandNotSupported, Address::unspecified())
                .await?;
            let _ = conn.shutdown().await;
        }
        Ok(Command::Bind(bind, _)) => {
            let mut conn = bind
                .reply(Reply::CommandNotSupported, Address::unspecified())
                .await?;
            let _ = conn.shutdown().await;
        }
        Ok(Command::Connect(connect, addr)) => {
            let target = match addr {
                Address::DomainAddress(domain, port) => TcpStream::connect((domain, port)).await,
                Address::SocketAddress(addr) => TcpStream::connect(addr).await,
            };

            if let Ok(mut target) = target {
                let mut conn = connect
                    .reply(Reply::Succeeded, Address::unspecified())
                    .await?;
                io::copy_bidirectional(&mut target, &mut conn).await?;
            } else {
                let mut conn = connect
                    .reply(Reply::HostUnreachable, Address::unspecified())
                    .await?;
                let _ = conn.shutdown().await;
            }
        }
        Err((mut conn, err)) => {
            let _ = conn.shutdown().await;
            return Err(err);
        }
    }

    Ok(())
}
