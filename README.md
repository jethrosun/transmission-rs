# transmission-rs
[![](https://meritbadge.herokuapp.com/transmission)](https://crates.io/crates/transmission)
[![Released API docs](https://docs.rs/transmission/badge.svg)](https://docs.rs/transmission)
[![pipeline status](https://gitlab.com/tornado-torrent/transmission-rs/badges/master/pipeline.svg)](https://gitlab.com/tornado-torrent/transmission-rs/commits/master)
[![coverage report](https://gitlab.com/tornado-torrent/transmission-rs/badges/master/coverage.svg)](https://gitlab.com/tornado-torrent/transmission-rs/commits/master)

Ergonomic Rust bindings for the [Transmission](https://transmissionbt.com/) BitTorrent client
based on [transmission-sys](https://gitlab.com/tornado-torrent/transmission-sys).

Created and maintained by the [Tornado Project](https://gitlab.com/tornado-torrent/)

Not intended to be used as a remote for the Transmission
daemon, but is a wrapper around `libtransmission`. Though remote support
may be added in the future.

## Building
This library is based on `transmission-sys` which has several external dependencies listed
below. Once those are installed you can build the library with the standard

```
cargo build
```

### Dependencies
- gcc (or Clang)
- cmake
- libclang-devel
- libopenssl-devel
- libcurl-devel
- libevent-devel