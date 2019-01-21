use std::io;
use std::path::PathBuf;

use transmission_sys;

use crate::error::{Error, TrResult};
use crate::torrent::Torrent;

/// Used to create a new torrent in a builder pattern
pub struct TorrentBuilder {
    comment: Option<String>,
    trackers: Vec<String>,
    file: PathBuf,
    webseeds: Option<Vec<String>>,
    piece_size_kib: u32,
}

impl TorrentBuilder {
    /// Create a new blank TorrentBuilder
    pub fn new() -> Self {
        Self {
            comment: None,
            trackers: Vec::new(),
            file: PathBuf::new(),
            webseeds: None,
            piece_size_kib: 1024, // 1 Megabyte
        }
    }

    /// Consume the builder and return the created torrent or an error
    pub fn build(self) -> TrResult<Torrent> {
        unimplemented!()
    }

    /// Set the file or folder the torrent is serving
    pub fn file(mut self, file: &str) -> io::Result<Self> {
        self.file = PathBuf::from(file).canonicalize()?;
        Ok(self)
    }

    /// Add a tracker to the torrent
    pub fn tracker(mut self, tracker: &str) -> Self {
        self.trackers.push(tracker.to_owned());
        self
    }

    /// Set the comment of the torrent
    pub fn comment(mut self, comment: &str) -> Self {
        self.comment = Some(comment.to_owned());
        self
    }

    /// The piece size in kilobytes. Default is 1024 (1 megabyte)
    pub fn piece_size(mut self, kilobytes: u32) -> Self {
        self.piece_size_kib = kilobytes;
        self
    }

    /// Add a webseed to the torrent
    pub fn webseed(mut self, webseed: &str) -> Self {
        if self.webseeds.is_some() {
            let mut wbs = self.webseeds.unwrap().clone();
            wbs.push(webseed.to_owned());
            self.webseeds = Some(wbs);
        } else {
            self.webseeds = Some(vec![webseed.to_owned()]);
        }
        self
    }
}
