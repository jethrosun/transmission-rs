//! Ergonomic Rust bindings for the [Transmission](https://transmissionbt.com/) BitTorrent client
//! based on [transmission-sys](https://gitlab.com/tornado-torrent/transmission-sys).
//!
//! Most interaction will be done through the `Client` struct.

// Re-exports
pub mod client;
pub mod error;
pub mod torrent;

pub use client::{Client, ClientConfig};
pub use torrent::{Torrent, TorrentBuilder};
