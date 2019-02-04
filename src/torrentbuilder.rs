use std::ffi;
use std::io;
use std::mem;
use std::path::PathBuf;
use std::ptr;

use transmission_sys;

use crate::error::{Error, TrResult};
use crate::torrentinfo::TorrentInfo;

/// Used to create a new torrent in a builder pattern
#[derive(Default)]
pub struct TorrentBuilder {
    comment: Option<String>,
    trackers: Vec<String>,
    file: PathBuf,
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
            webseeds: None,
            is_private: false,
        }
    }

    /// Consume the builder and return the created torrent or an error
    pub fn build(self) -> TrResult<TorrentInfo> {
        let file_c_string = ffi::CString::new(self.file.to_str().unwrap()).unwrap();
        let mut tr_trackers: Vec<transmission_sys::tr_tracker_info> = Vec::new();
        let meta_builder;
        let tr_comment = if let Some(cmt) = self.comment {
            ffi::CString::new(cmt).unwrap()
        } else {
            ffi::CString::new("").unwrap()
        };
        /* Dump the trackers into a struct transmission can understand */
        let mut i = 0;
        for tracker in self.trackers {
            let tracker_announce = ffi::CString::new(format!("{}/announce", tracker)).unwrap();
            let tracker_scrape = ffi::CString::new(format!("{}/scrape", tracker)).unwrap();
            tr_trackers.push(transmission_sys::tr_tracker_info {
                tier: i as i32,
                announce: tracker_announce.into_raw(),
                scrape: tracker_scrape.into_raw(),
                id: i as u32,
            });
            i += 1;
        }
        /* torrent_path is the output file that will become the torrent */
        let torrent_path = ffi::CString::new(format!("{}.torrent", self.file.display())).unwrap();
        unsafe {
            meta_builder = transmission_sys::tr_metaInfoBuilderCreate(file_c_string.as_ptr());
            transmission_sys::tr_makeMetaInfo(
                meta_builder,
                ptr::null(),
                tr_trackers.as_ptr(),
                tr_trackers.len() as i32,
                tr_comment.as_ptr(),
                self.is_private,
            );

            let ctor = transmission_sys::tr_ctorNew(ptr::null());
            let mut info: transmission_sys::tr_info = mem::uninitialized();
            match transmission_sys::tr_ctorSetMetainfoFromFile(ctor, torrent_path.as_ptr()) {
                0 => match transmission_sys::tr_torrentParse(ctor, &mut info) {
                    0 => Ok(TorrentInfo::from(info)),
                    _ => Err(Error::ParseErr),
                },
                _ => Err(Error::ParseErr),
            }
        }
    }

    /// Set the file or folder the torrent is serving
    pub fn set_file(mut self, file: &str) -> io::Result<Self> {
        self.file = PathBuf::from(file).canonicalize()?;
        Ok(self)
    }

    /// Add a tracker to the torrent
    pub fn set_tracker(mut self, tracker: &str) -> Self {
        self.trackers.push(tracker.to_owned());
        self
    }

    /// Set the comment of the torrent
    pub fn set_comment(mut self, comment: &str) -> Self {
        self.comment = Some(comment.to_owned());
        self
    }

    /// Add a webseed to the torrent
    pub fn set_webseed(mut self, webseed: &str) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn build_torrent() {
        let file_to_build_from = String::from("./Cargo.toml");
        let tracker_url = String::from("udp://tracker.opentrackr.org:1337");
        let info = TorrentBuilder::new()
            .set_file(&file_to_build_from)
            .unwrap()
            .set_tracker(&tracker_url)
            .set_comment("Test torrent")
            .build()
            .unwrap();
        println!("{:#?}", info);
    }
}
