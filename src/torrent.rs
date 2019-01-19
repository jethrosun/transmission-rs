use std::ffi;

use serde::Serialize;
use transmission_sys;

use chrono::prelude::NaiveDateTime;

use crate::error::{Error, TrResult};

/// The various states that a torrent can be in.
#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct TorrentStats {
    /// The ID of the torrent
    pub id: i32,
    /// The state of the torrent. Internally Transmission calls this the "activity",
    pub state: TorrentState,
    /// The error state (if any).
    pub error: Error,
    /// A string describing the above error if any
    pub error_string: String,
    /// Progress rechecking a torrent
    pub recheck_progress: f32,
    /// Percent of the total download completed
    pub percent_complete: f32,
    /// Percent of the metadata download completed
    pub metadata_percent_complete: f32,
    /// Percent of the desired download completed.
    /// This differs from [`torrent::TorrentStats::percent_complete`] if the user only wants some of a torrent's files.
    pub percent_done: f32,
    /// Percent of the seed ratio uploaded. 1 if completed or infinite.
    pub seed_ratio_percent_done: f32,
    pub raw_upload_speed_kbps: f32,
    pub raw_download_speed_kbps: f32,
    pub piece_upload_speed_kbps: f32,
    pub piece_download_speed_kbps: f32,
    pub eta: i32,
    pub eta_idle: i32,
    pub peers_connected: i32,
    pub peers_from: [i32; 7],
    pub peers_sending_to_us: i32,
    pub peers_getting_from_us: i32,
    pub webseeds_sending_to_us: i32,
    pub size_when_done: u64,
    pub left_until_done: u64,
    pub desired_available: u64,
    pub corrupt_ever: u64,
    pub uploaded_ever: u64,
    pub downloaded_ever: u64,
    pub have_valid: u64,
    pub have_unchecked: u64,
    pub manual_announce_time: NaiveDateTime,
    pub ratio: f32,
    pub added_date: NaiveDateTime,
    pub done_date: NaiveDateTime,
    pub start_date: NaiveDateTime,
    pub activity_date: NaiveDateTime,
    pub idle_secs: i32,
    pub seconds_downloading: i32,
    pub seconds_seeding: i32,
    pub finished: bool,
    pub queue_position: i32,
    pub is_stalled: bool,
}

/// Converts tr_stat into TorrentStats
impl From<transmission_sys::tr_stat> for TorrentStats {
    fn from(stat: transmission_sys::tr_stat) -> Self {
        Self {
            id: stat.id,
            state: TorrentState::from(stat.activity),
            error: Error::from(stat.error),
            // Strings in C are awful and force use to do things like this
            error_string: ffi::CStr::from_bytes_with_nul(unsafe {
                &*(&stat.errorString[0..] as *const _ as *const [u8])
            })
            .unwrap()
            .to_str()
            .unwrap()
            .into(),
            recheck_progress: stat.recheckProgress,
            percent_complete: stat.percentComplete,
            metadata_percent_complete: stat.metadataPercentComplete,
            percent_done: stat.percentDone,
            seed_ratio_percent_done: stat.seedRatioPercentDone,
            raw_upload_speed_kbps: stat.rawUploadSpeed_KBps,
            raw_download_speed_kbps: stat.rawDownloadSpeed_KBps,
            piece_upload_speed_kbps: stat.pieceUploadSpeed_KBps,
            piece_download_speed_kbps: stat.pieceDownloadSpeed_KBps,
            eta: stat.eta,
            eta_idle: stat.etaIdle,
            peers_connected: stat.peersConnected,
            peers_from: stat.peersFrom,
            peers_sending_to_us: stat.peersSendingToUs,
            peers_getting_from_us: stat.peersGettingFromUs,
            webseeds_sending_to_us: stat.webseedsSendingToUs,
            size_when_done: stat.sizeWhenDone,
            left_until_done: stat.leftUntilDone,
            desired_available: stat.desiredAvailable,
            corrupt_ever: stat.corruptEver,
            uploaded_ever: stat.uploadedEver,
            downloaded_ever: stat.downloadedEver,
            have_valid: stat.haveValid,
            have_unchecked: stat.haveUnchecked,
            manual_announce_time: NaiveDateTime::from_timestamp(stat.manualAnnounceTime, 0),
            ratio: stat.ratio,
            added_date: NaiveDateTime::from_timestamp(stat.addedDate, 0),
            done_date: NaiveDateTime::from_timestamp(stat.doneDate, 0),
            start_date: NaiveDateTime::from_timestamp(stat.startDate, 0),
            activity_date: NaiveDateTime::from_timestamp(stat.activityDate, 0),
            idle_secs: stat.idleSecs,
            seconds_downloading: stat.secondsDownloading,
            seconds_seeding: stat.secondsSeeding,
            finished: stat.finished,
            queue_position: stat.queuePosition,
            is_stalled: stat.isStalled,
        }
    }
}

/// Representation of a torrent download.
pub struct Torrent {
    tr_torrent: transmission_sys::tr_torrent,
}

impl Torrent {
    /// Create a new torrent from a tr_ctor
    pub fn from_ctor(ctor: *mut transmission_sys::tr_ctor) -> TrResult<Self> {
        let tor;
        let mut error = 0;
        let mut dupli = 0;
        unsafe {
            tor = transmission_sys::tr_torrentNew(ctor, &mut error, &mut dupli);
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
    pub fn remove(&mut self, with_data: bool) {
        unsafe {
            transmission_sys::tr_torrentRemove(&mut self.tr_torrent, with_data, None);
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
