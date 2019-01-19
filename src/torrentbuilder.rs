use transmission_sys;

use crate::error::{Error, TrResult};
use crate::torrent::Torrent;

/// Used to create a new torrent in a builder pattern
pub struct TorrentBuilder {
    // TODO fill this out
}

impl TorrentBuilder {
    /// Create a new blank TorrentBuilder
    pub fn new() -> Self {
        unimplemented!()
    }

    /// Consume the builder and return the created torrent or an error
    pub fn build(self) -> TrResult<Torrent> {
        unimplemented!()
    }

    // TODO fill this out
}
