use transmission_sys;

use super::model::TrResult;
use super::torrent::Torrent;

/// Interface into the major functions of Transmission
/// including adding, querying, and removing torrents.
pub struct Client {}

impl Client {
    /// Creates a new `Client` after initializing the necessary things.
    pub fn new() -> Self {
        unimplemented!()
    }

    /// Adds a torrent using either a path or URL to a torrent file.
    pub fn add_torrent_file(&self, file: &str) -> TrResult<Torrent> {
        unimplemented!()
    }

    /// Adds a torrent to download using a magnet link.
    pub fn add_torrent_magnet(&self, link: &str) -> TrResult<Torrent> {
        unimplemented!()
    }

    /// Removes a torrent from the downloads
    pub fn remove_torrent(&self, torrent: Torrent) -> TrResult<()> {
        unimplemented!()
    }

    /// Returns a list of current torrents
    pub fn torrents(&self) -> TrResult<Vec<Torrent>> {
        unimplemented!()
    }
}
