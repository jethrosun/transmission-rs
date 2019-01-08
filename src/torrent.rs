use std::ffi;

use serde::Serialize;
use transmission_sys;

use crate::error::{Error, TrResult};

/// The various states that a torrent can be in.
#[derive(Serialize)]
pub enum TorrentState {
    Downloading,
    Seeding,
    Paused,
    Error,
}

/// Representation of a torrent download.
#[derive(Serialize)]
pub struct Torrent {
    tr_torrent: transmission_sys::tr_torrent,
}

impl Torrent {
    /// Create a new torrent from a tr_ctor
    fn new(ctor: transmission_sys::tr_ctor) -> TrResult<Self> {
        let tor;
        let mut error: u32;
        let dupli: &mut transmission_sys::tr_info;
        unsafe {
            error = transmission_sys::tr_torrentParse(&ctor, &mut dupli);
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

/*
pub struct TorrentStats {
    pub id: usize,
    pub activity: tr_torrent_activity,
    pub error: tr_stat_errtype,
    pub errorString: String,
    pub recheckProgress: f32,
    pub percentComplete: f32,
    pub metadataPercentComplete: f32,
    pub percentDone: f32,
    pub seedRatioPercentDone: f32,
    pub rawUploadSpeed_KBps: f32,
    pub rawDownloadSpeed_KBps: f32,
    pub pieceUploadSpeed_KBps: f32,
    pub pieceDownloadSpeed_KBps: f32,
    pub eta: usize,
    pub etaIdle: usize,
    pub peersConnected: usize,
    pub peersFrom: [usize; 7],
    pub peersSendingToUs: usize,
    pub peersGettingFromUs: usize,
    pub webseedsSendingToUs: usize,
    pub sizeWhenDone: u64,
    pub leftUntilDone: u64,
    pub desiredAvailable: u64,
    pub corruptEver: u64,
    pub uploadedEver: u64,
    pub downloadedEver: u64,
    pub haveValid: u64,
    pub haveUnchecked: u64,
    pub manualAnnounceTime: time_t,
    pub ratio: f32,
    pub addedDate: time_t,
    pub doneDate: time_t,
    pub startDate: time_t,
    pub activityDate: time_t,
    pub idleSecs: usize,
    pub secondsDownloading: usize,
    pub secondsSeeding: usize,
    pub finished: bool,
    pub queuePosition: usize,
    pub isStalled: bool,

impl From<transmission_sys::tr_stat> for TorrentStats {
    fn from(stat: transmission_sys::tr_stat) -> Self {
        Self {

        }
    }
}
*/
