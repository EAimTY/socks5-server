# socks5-server

Fine-grained relatively low-level asynchronized SOCKS5 server library based on tokio.

[![Version](https://img.shields.io/crates/v/socks5-server.svg?style=flat)](https://crates.io/crates/socks5-server)
[![Documentation](https://img.shields.io/badge/docs-release-brightgreen.svg?style=flat)](https://docs.rs/socks5-server)
[![License](https://img.shields.io/crates/l/socks5-server.svg?style=flat)](https://github.com/EAimTY/socks5-server/blob/master/LICENSE)

This crate is based on abstraction provided by crate [socks5-proto](https://crates.io/crates/socks5-proto). Check it out for more information.

## Features

- All protocol details defined in [RFC 1928](https://tools.ietf.org/html/rfc1928) are implemented
- Fully asynchronized
- Customizable authentication

## Usage

Create a [`socks5_server::Server`](https://docs.rs/socks5-server/latest/socks5_server/struct.Server.html) and `accept()` on it.

Check [examples](https://github.com/EAimTY/socks5-server/tree/master/socks5-server/examples) for usage examples.

## License
GNU General Public License v3.0
