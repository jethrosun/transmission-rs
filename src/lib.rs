// Re-exports
pub mod client;
pub mod error;
pub mod torrent;
pub mod torrentbuilder;

pub use crate::client::Client;
pub use crate::client::ClientConfig;
pub use crate::torrent::Torrent;
pub use crate::torrentbuilder::TorrentBuilder;
