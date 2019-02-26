//! Ergonomic Rust bindings for the [Transmission](https://transmissionbt.com/) BitTorrent client
//! based on [transmission-sys](https://gitlab.com/tornado-torrent/transmission-sys).
//!
//! Most interaction will be done through the `Client` struct.

// Re-exports
pub mod client;
pub mod error;
pub mod torrent;
pub mod torrentbuilder;
pub mod torrentinfo;
pub mod torrentstats;

pub use crate::client::Client;
pub use crate::client::ClientConfig;
pub use crate::error::Error;
pub use crate::torrent::Torrent;
pub use crate::torrentbuilder::TorrentBuilder;
