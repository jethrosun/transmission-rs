use std::ffi;

use serde::Serialize;
use transmission_sys;

use chrono::prelude::NaiveDateTime;

use crate::error::{Error, TrResult};

/// The various states that a torrent can be in.
#[derive(Serialize)]
pub enum TorrentState {
    /// The torrent is downloading
    Downloading,
    /// The torrent is waiting to download
    DownloadingWait,
    /// The torrent is seeding
    Seeding,
    /// The torrent is waiting to seed
    SeedingWait,
    /// The torrent is stopped
    Stopped,
    /// The torrent is being checked
    Checking,
    /// The torrent is waiting to be checked
    CheckingWait,
    /// The torrent has errored
    Error,
}

impl From<transmission_sys::tr_torrent_activity> for TorrentState {
    fn from(act: transmission_sys::tr_torrent_activity) -> Self {
        match act {
            transmission_sys::tr_torrent_activity_TR_STATUS_DOWNLOAD => TorrentState::Downloading,
            transmission_sys::tr_torrent_activity_TR_STATUS_DOWNLOAD_WAIT => {
                TorrentState::DownloadingWait
            }
            transmission_sys::tr_torrent_activity_TR_STATUS_SEED => TorrentState::Seeding,
            transmission_sys::tr_torrent_activity_TR_STATUS_SEED_WAIT => TorrentState::SeedingWait,
            transmission_sys::tr_torrent_activity_TR_STATUS_CHECK => TorrentState::Checking,
            transmission_sys::tr_torrent_activity_TR_STATUS_CHECK_WAIT => {
                TorrentState::CheckingWait
            }
            transmission_sys::tr_torrent_activity_TR_STATUS_STOPPED => TorrentState::Stopped,
            _ => TorrentState::Error,
        }
    }
}

#[derive(Serialize)]
pub struct TorrentStats {
    /// The ID of the torrent
    pub id: i32,
    /// The state of the torrent. Internally Transmission calls this the "activity",
    pub state: TorrentState,
    /// The error state (if any).
    pub error: Error,
    /// A string describing the above error if any
    pub errorString: String,
    /// Progress rechecking a torrent
    pub recheckProgress: f32,
    /// Percent of the total download completed
    pub percentComplete: f32,
    /// Percent of the metadata download completed
    pub metadataPercentComplete: f32,
    /// Percent of the desired download completed.
    /// This differs from [`torrent::TorrentStats::percentComplete`] if the user only wants some of a torrent's files.
    pub percentDone: f32,
    /// Percent of the seed ratio uploaded. 1 if completed or infinite.
    pub seedRatioPercentDone: f32,
    pub rawUploadSpeed_KBps: f32,
    pub rawDownloadSpeed_KBps: f32,
    pub pieceUploadSpeed_KBps: f32,
    pub pieceDownloadSpeed_KBps: f32,
    pub eta: i32,
    pub etaIdle: i32,
    pub peersConnected: i32,
    pub peersFrom: [i32; 7],
    pub peersSendingToUs: i32,
    pub peersGettingFromUs: i32,
    pub webseedsSendingToUs: i32,
    pub sizeWhenDone: u64,
    pub leftUntilDone: u64,
    pub desiredAvailable: u64,
    pub corruptEver: u64,
    pub uploadedEver: u64,
    pub downloadedEver: u64,
    pub haveValid: u64,
    pub haveUnchecked: u64,
    pub manualAnnounceTime: NaiveDateTime,
    pub ratio: f32,
    pub addedDate: NaiveDateTime,
    pub doneDate: NaiveDateTime,
    pub startDate: NaiveDateTime,
    pub activityDate: NaiveDateTime,
    pub idleSecs: i32,
    pub secondsDownloading: i32,
    pub secondsSeeding: i32,
    pub finished: bool,
    pub queuePosition: i32,
    pub isStalled: bool,
}

/// Converts tr_stat into TorrentStats
impl From<transmission_sys::tr_stat> for TorrentStats {
    fn from(stat: transmission_sys::tr_stat) -> Self {
        Self {
            id: stat.id,
            state: TorrentState::from(stat.activity),
            error: Error::from(stat.error),
            // Strings in C are awful and force use to do things like this
            errorString: ffi::CStr::from_bytes_with_nul(unsafe {
                &*(&stat.errorString[0..] as *const _ as *const [u8])
            })
            .unwrap()
            .to_str()
            .unwrap()
            .into(),
            recheckProgress: stat.recheckProgress,
            percentComplete: stat.percentComplete,
            metadataPercentComplete: stat.metadataPercentComplete,
            percentDone: stat.percentDone,
            seedRatioPercentDone: stat.seedRatioPercentDone,
            rawUploadSpeed_KBps: stat.rawUploadSpeed_KBps,
            rawDownloadSpeed_KBps: stat.rawDownloadSpeed_KBps,
            pieceUploadSpeed_KBps: stat.pieceUploadSpeed_KBps,
            pieceDownloadSpeed_KBps: stat.pieceDownloadSpeed_KBps,
            eta: stat.eta,
            etaIdle: stat.etaIdle,
            peersConnected: stat.peersConnected,
            peersFrom: stat.peersFrom,
            peersSendingToUs: stat.peersSendingToUs,
            peersGettingFromUs: stat.peersGettingFromUs,
            webseedsSendingToUs: stat.webseedsSendingToUs,
            sizeWhenDone: stat.sizeWhenDone,
            leftUntilDone: stat.leftUntilDone,
            desiredAvailable: stat.desiredAvailable,
            corruptEver: stat.corruptEver,
            uploadedEver: stat.uploadedEver,
            downloadedEver: stat.downloadedEver,
            haveValid: stat.haveValid,
            haveUnchecked: stat.haveUnchecked,
            manualAnnounceTime: NaiveDateTime::from_timestamp(stat.manualAnnounceTime, 0),
            ratio: stat.ratio,
            addedDate: NaiveDateTime::from_timestamp(stat.addedDate, 0),
            doneDate: NaiveDateTime::from_timestamp(stat.doneDate, 0),
            startDate: NaiveDateTime::from_timestamp(stat.startDate, 0),
            activityDate: NaiveDateTime::from_timestamp(stat.activityDate, 0),
            idleSecs: stat.idleSecs,
            secondsDownloading: stat.secondsDownloading,
            secondsSeeding: stat.secondsSeeding,
            finished: stat.finished == 0,
            queuePosition: stat.queuePosition,
            isStalled: stat.isStalled == 0,
        }
    }
}

/// Representation of a torrent download.
pub struct Torrent {
    tr_torrent: transmission_sys::tr_torrent,
}

impl Torrent {
    /// Create a new torrent from a tr_ctor
    fn new(ctor: transmission_sys::tr_ctor) -> TrResult<Self> {
        let tor;
        let mut error = 0;
        let mut dupli = 0;
        unsafe {
            tor = transmission_sys::tr_torrentNew(&ctor, &mut error, &mut dupli);
        }
        // Match the possible errors from torrentNew
        match error as u32 {
            transmission_sys::tr_parse_result_TR_PARSE_ERR => Err(Error::ParseErr),
            transmission_sys::tr_parse_result_TR_PARSE_DUPLICATE => Err(Error::ParseDuplicate),
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

    // All the stats of the torrent as given by Transmission
    pub fn stats(&mut self) -> TorrentStats {
        let stats;
        unsafe {
            stats = transmission_sys::tr_torrentStat(&mut self.tr_torrent);
        }
        TorrentStats::from(unsafe { *stats })
    }
}
