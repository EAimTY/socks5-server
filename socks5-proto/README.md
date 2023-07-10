# socks5-proto

This crate provides fundamental abstractions and async read / write functions for SOCKS5 protocol.

[![Version](https://img.shields.io/crates/v/socks5-proto.svg?style=flat)](https://crates.io/crates/socks5-proto)
[![Documentation](https://img.shields.io/badge/docs-release-brightgreen.svg?style=flat)](https://docs.rs/socks5-proto)
[![License](https://img.shields.io/crates/l/socks5-proto.svg?style=flat)](https://github.com/EAimTY/socks5-server/blob/master/LICENSE)

Check out crate [socks5-server](https://crates.io/crates/socks5-server) for a complete SOCKS5 server implementation.

## Example

```rust no_run
use socks5_proto::{
    handshake::{
        Method as HandshakeMethod, Request as HandshakeRequest, Response as HandshakeResponse,
    },
    Address, Error, ProtocolError, Reply, Request, Response,
};
use tokio::{io::AsyncWriteExt, net::TcpListener};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let listener = TcpListener::bind("127.0.0.1:5000").await?;
    let (mut stream, _) = listener.accept().await?;

    let hs_req = HandshakeRequest::read_from(&mut stream).await?;

    if hs_req.methods.contains(&HandshakeMethod::NONE) {
        let hs_resp = HandshakeResponse::new(HandshakeMethod::NONE);
        hs_resp.write_to(&mut stream).await?;
    } else {
        let hs_resp = HandshakeResponse::new(HandshakeMethod::UNACCEPTABLE);
        hs_resp.write_to(&mut stream).await?;
        let _ = stream.shutdown().await;
        return Err(Error::Protocol(
            ProtocolError::NoAcceptableHandshakeMethod {
                version: socks5_proto::SOCKS_VERSION,
                chosen_method: HandshakeMethod::NONE,
                methods: hs_req.methods,
            },
        ));
    }

    let req = match Request::read_from(&mut stream).await {
        Ok(req) => req,
        Err(err) => {
            let resp = Response::new(Reply::GeneralFailure, Address::unspecified());
            resp.write_to(&mut stream).await?;
            let _ = stream.shutdown().await;
            return Err(err);
        }
    };

    match req.command {
        _ => todo!(), // process request
    }
}
```

## License
GNU General Public License v3.0
