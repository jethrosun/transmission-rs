use std::ffi;

use transmission_sys;

use crate::error::{Error, TrResult};

/// The various states that a torrent can be in.
pub enum TorrentState {
    Downloading,
    Seeding,
    Paused,
    Error,
}

/// Representation of a torrent download.
pub struct Torrent {
    tr_torrent: transmission_sys::tr_torrent,
}

impl Torrent {
    /// Create a new torrent from a tr_ctor
    fn new(ctor: transmission_sys::tr_ctor) -> TrResult<Self> {
        let tor;
        let mut error: i32 = 0;
        let mut dupli: i32 = 0;
        unsafe {
            tor = transmission_sys::tr_torrentNew(&ctor, &mut error, &mut dupli);
        }
        // Match the possible errors from torrentNew
        match error as u32 {
            transmission_sys::tr_parse_result_TR_PARSE_ERR => Err(Error::ParseErr),
            transmission_sys::tr_parse_result_TR_PARSE_DUPLICATE => {
                Err(Error::ParseDuplicate(dupli))
            }
            transmission_sys::tr_parse_result_TR_PARSE_OK => unsafe {
                Ok(Self { tr_torrent: *tor })
            },
            _ => Err(Error::Unknown),
        }
    }
    /// Start or resume the torrent
    pub fn start(&mut self) {
        unsafe {
            transmission_sys::tr_torrentStart(&mut self.tr_torrent);
        }
    }
    /// Stop (pause) the torrent
    pub fn stop(&mut self) {
        unsafe {
            transmission_sys::tr_torrentStop(&mut self.tr_torrent);
        }
    }

    /// Removes a torrent from the downloads
    pub fn remove(&mut self) {
        unsafe {
            transmission_sys::tr_torrentRemove(&mut self.tr_torrent, 0, None);
        }
    }

    //# The following functions get information about the torrent

    /// All the information about a torrent
    pub fn stat(&mut self) -> TorrentStats {
        unsafe { transmission_sys::tr_torrentStat(&mut self.tr_torrent) }
    }

    /// This torrent's name
    pub fn name(&self) -> &str {
        unsafe {
            let c_str = transmission_sys::tr_torrentName(&self.tr_torrent);
            ffi::CStr::from_ptr(c_str).to_str().unwrap()
        }
    }

    /// The unique ID of the torrent
    pub fn id(&self) -> usize {
        let c_id;
        unsafe {
            c_id = transmission_sys::tr_torrentId(&self.tr_torrent);
        }
        c_id as usize
    }
}

pub struct TorrentStats {}

impl From<transmission_sys::tr_stat> for TorrentStats {
    fn from(stat: transmission_sys::tr_stat) -> Self {}
}
