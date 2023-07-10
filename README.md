# socks5-server

Fine-grained relatively low-level asynchronized SOCKS5 server library based on tokio.

[![Version](https://img.shields.io/crates/v/socks5-server.svg?style=flat)](https://crates.io/crates/socks5-server)
[![Documentation](https://img.shields.io/badge/docs-release-brightgreen.svg?style=flat)](https://docs.rs/socks5-server)
[![License](https://img.shields.io/crates/l/socks5-server.svg?style=flat)](https://github.com/EAimTY/socks5-server/blob/master/LICENSE)

This repo includes two crates:
- [socks5-server](https://github.com/EAimTY/socks5-server/tree/master/socks5-server) - Provides a fine-grained, relatively low-level asynchronized SOCKS5 server library based on tokio
- [socks5-proto](https://github.com/EAimTY/socks5-server/tree/master/socks5-proto) - Provides fundamental abstractions and async read / write functions for SOCKS5 protocol

Due to the long-term evolution, the implementation of the socks5 protocol varies greatly according to the requirements of different usage scenarios. Therefore, this library abstracts the socks5 protocol from a lower level so that it can be adapted to more usage scenarios.

## License
GNU General Public License v3.0
