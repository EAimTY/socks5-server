# socks5-server

This crate provides a relatively low-level asynchronized SOCKS5 server implementation based on tokio.

[![Version](https://img.shields.io/crates/v/socks5-server.svg?style=flat)](https://crates.io/crates/socks5-server)
[![Documentation](https://img.shields.io/badge/docs-release-brightgreen.svg?style=flat)](https://docs.rs/socks5-server)
[![License](https://img.shields.io/crates/l/socks5-server.svg?style=flat)](https://github.com/EAimTY/socks5-server/blob/master/LICENSE)

Check out crate [socks5-proto](https://crates.io/crates/socks5-proto) for an implementation of SOCKS5 fundamental abstractions and async read / write functions.

## Features

- Fully asynchronized
- Supports all SOCKS5 commands
  - CONNECT
  - BIND
  - ASSOCIATE
- Customizable authentication

## Usage

The entry point of this crate is [`socks5_server::Server`](https://docs.rs/socks5-server/latest/socks5_server/struct.Server.html).

Check [examples](https://github.com/EAimTY/socks5-server/tree/master/socks5-server/examples) for usage examples.

## License
GNU General Public License v3.0
