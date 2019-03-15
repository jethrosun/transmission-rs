//! Interface for creating a new torrent.
use std::ffi;
use std::io;
use std::path::PathBuf;

use transmission_sys;

use crate::error::{Error, TrResult};

/// Used to create a new torrent in a builder pattern.
///
/// ```
/// use transmission::Torrent;
/// use transmission::TorrentBuilder;
///
/// let file_to_build_from = String::from("./Cargo.toml");
/// let tracker_url = String::from("udp://tracker.opentrackr.org:1337");
///
/// let path = TorrentBuilder::new()
///     .set_file(&file_to_build_from)
///     .unwrap()
///     .add_tracker(&tracker_url)
///     .set_comment("Test torrent")
///     .build()
///     .expect("Failed to build torrent");
///
/// # std::fs::remove_file(path).unwrap();
/// ```
#[derive(Default)]
pub struct TorrentBuilder {
    comment: Option<String>,
    trackers: Vec<String>,
    file: PathBuf,
    output_file: Option<PathBuf>,
    webseeds: Option<Vec<String>>,
    is_private: bool,
}

impl TorrentBuilder {
    /// Create a new blank TorrentBuilder
    pub fn new() -> Self {
        Self {
            comment: None,
            trackers: Vec::new(),
            file: PathBuf::new(),
            output_file: None,
            webseeds: None,
            is_private: false,
        }
    }

    /// Consume the builder and return the created torrent or an error
    #[allow(clippy::while_immutable_condition)]
    pub fn build(self) -> TrResult<String> {
        let file_c_string = ffi::CString::new(self.file.to_str().unwrap()).unwrap();
        let mut tr_trackers: Vec<transmission_sys::tr_tracker_info> = Vec::new();
        let meta_builder;
        let tr_comment = if let Some(cmt) = self.comment {
            ffi::CString::new(cmt).unwrap()
        } else {
            ffi::CString::new("").unwrap()
        };

        // Dump the trackers into a struct transmission can understand
        for (i, tracker) in self.trackers.iter().enumerate() {
            let tracker_announce = ffi::CString::new(format!("{}/announce", tracker)).unwrap();
            let tracker_scrape = ffi::CString::new(format!("{}/scrape", tracker)).unwrap();
            tr_trackers.push(transmission_sys::tr_tracker_info {
                tier: i as i32,
                announce: tracker_announce.into_raw(),
                scrape: tracker_scrape.into_raw(),
                id: i as u32,
            });
        }

        // The path the .torrent file is
        let output_path = if let Some(ofile) = self.output_file {
            ffi::CString::new(format!("{}.torrent", ofile.display())).unwrap()
        } else {
            ffi::CString::new(format!("{}.torrent", self.file.display())).unwrap()
        };
        let error: Error;

        unsafe {
            // Start building metainfo from target file
            meta_builder = transmission_sys::tr_metaInfoBuilderCreate(file_c_string.as_ptr());

            // Finish building metainfo
            transmission_sys::tr_makeMetaInfo(
                meta_builder,
                output_path.as_ptr(), // `null` is the same as when output_file is None
                tr_trackers.as_ptr(),
                tr_trackers.len() as i32,
                tr_comment.as_ptr(),
                self.is_private,
            );

            // Wait for the builder to finish
            while !(*meta_builder).isDone {}

            // Get the error from the builder
            error = Error::from((*meta_builder).result);

            transmission_sys::tr_metaInfoBuilderFree(meta_builder);
        }

        // If there was no error return the output_path
        error
            .to_result()
            .and_then(|_| Ok(output_path.to_str().unwrap().to_owned()))
    }

    /// Set the file or folder the torrent is serving.
    ///
    /// Takes the path to the file which **must exist**.
    pub fn set_file(mut self, file: &str) -> io::Result<Self> {
        self.file = PathBuf::from(file).canonicalize()?;
        Ok(self)
    }

    /// Set's the full path of to the .torrent file that will be created.
    pub fn set_output_file(mut self, file: &str) -> Self {
        self.output_file = Some(PathBuf::from(file));
        self
    }

    /// Add a tracker to the torrent
    pub fn add_tracker(mut self, tracker: &str) -> Self {
        self.trackers.push(tracker.to_owned());
        self
    }

    /// Set all the trackers on the torrent, replacing existing.
    pub fn set_trackers(mut self, trackers: Vec<&str>) -> Self {
        self.trackers = trackers.iter().map(|s| String::from(*s)).collect();
        self
    }

    /// Set the comment of the torrent
    pub fn set_comment(mut self, comment: &str) -> Self {
        self.comment = Some(comment.to_owned());
        self
    }

    /// Add a webseed to the torrent
    pub fn add_webseed(mut self, webseed: &str) -> Self {
        if self.webseeds.is_some() {
            let mut wbs = self.webseeds.unwrap().clone();
            wbs.push(webseed.to_owned());
            self.webseeds = Some(wbs);
        } else {
            self.webseeds = Some(vec![webseed.to_owned()]);
        }
        self
    }

    /// Set all the webseeds on the torrent, replacing existing.
    pub fn set_webseeds(mut self, webseeds: Vec<&str>) -> Self {
        if webseeds.is_empty() {
            self.webseeds = Some(webseeds.iter().map(|s| String::from(*s)).collect());
        } else {
            self.webseeds = None;
        }
        self
    }
}
