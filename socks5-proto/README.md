# socks5-proto

This crate provides fundamental abstractions and async read / write functions for SOCKS5 protocol.

[![Version](https://img.shields.io/crates/v/socks5-proto.svg?style=flat)](https://crates.io/crates/socks5-proto)
[![Documentation](https://img.shields.io/badge/docs-release-brightgreen.svg?style=flat)](https://docs.rs/socks5-proto)
[![License](https://img.shields.io/crates/l/socks5-proto.svg?style=flat)](https://github.com/EAimTY/socks5-server/blob/master/LICENSE)

Check out crate [socks5-server](https://crates.io/crates/socks5-server) for a complete SOCKS5 server implementation.

## Example

```rust no_run
use socks5_proto::{
    Address, HandshakeMethod, HandshakeRequest, HandshakeResponse, Reply, Request, Response,
};
use std::{
    io,
    net::{Ipv4Addr, SocketAddr},
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:5000").await?;
    let (mut stream, _) = listener.accept().await?;

    let hs_req = HandshakeRequest::read_from(&mut stream).await?;

    if hs_req.methods.contains(&HandshakeMethod::None) {
        let hs_resp = HandshakeResponse::new(HandshakeMethod::None);
        hs_resp.write_to(&mut stream).await?;
    } else {
        let hs_resp = HandshakeResponse::new(HandshakeMethod::Unacceptable);
        hs_resp.write_to(&mut stream).await?;
    }

    let req = match Request::read_from(&mut stream).await {
        Ok(req) => req,
        Err(err) => {
            let resp = Response::new(
                Reply::GeneralFailure,
                Address::SocketAddress(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0))),
            );
            resp.write_to(&mut stream).await?;
            return Err(err);
        }
    };

    match req.command {
        _ => {} // process request
    }

    Ok(())
}
```

## License
GNU General Public License v3.0
