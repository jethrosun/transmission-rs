//! Contains the TorrentStats struct.
use chrono::prelude::NaiveDateTime;
use serde::{Deserialize, Serialize};
use transmission_sys;

use crate::error::Error;

/// The various states that a torrent can be in.
#[derive(Debug, Serialize, Deserialize)]
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
    /// Convert from the unsafe generated type to the safe library type
    fn from(act: transmission_sys::tr_torrent_activity) -> Self {
        match act {
            transmission_sys::tr_torrent_activity::TR_STATUS_DOWNLOAD => TorrentState::Downloading,
            transmission_sys::tr_torrent_activity::TR_STATUS_DOWNLOAD_WAIT => {
                TorrentState::DownloadingWait
            }
            transmission_sys::tr_torrent_activity::TR_STATUS_SEED => TorrentState::Seeding,
            transmission_sys::tr_torrent_activity::TR_STATUS_SEED_WAIT => TorrentState::SeedingWait,
            transmission_sys::tr_torrent_activity::TR_STATUS_CHECK => TorrentState::Checking,
            transmission_sys::tr_torrent_activity::TR_STATUS_CHECK_WAIT => {
                TorrentState::CheckingWait
            }
            transmission_sys::tr_torrent_activity::TR_STATUS_STOPPED => TorrentState::Stopped,
        }
    }
}

/// The stats of a torrent.
/// These only exist for a torrent that has been added to a session.
///
/// You can get it with `Torrent::stats()`.
#[derive(Debug, Serialize, Deserialize)]
pub struct TorrentStats {
    /// The ID of the torrent.
    pub id: i32,
    /// The state of the torrent. Internally Transmission calls this the "activity",
    pub state: TorrentState,
    /// The error state (if any).
    pub error: Error,
    /// A string describing the above error if any.
    pub error_string: String,
    /// Progress rechecking a torrent.
    pub recheck_progress: f32,
    /// Percent of the total download completed.
    pub percent_complete: f32,
    /// Percent of the metadata download completed.
    pub metadata_percent_complete: f32,
    /// Percent of the desired download completed.
    /// This differs from [`torrent::TorrentStats::percent_complete`] if the user only wants some of a torrent's files.
    pub percent_done: f32,
    /// Percent of the seed ratio uploaded. 1 if completed or infinite.
    pub seed_ratio_percent_done: f32,
    /// The raw upload speed.
    pub raw_upload_speed_kbps: f32,
    /// The raw download speed.
    pub raw_download_speed_kbps: f32,
    /// The actual piece upload speed.
    pub piece_upload_speed_kbps: f32,
    /// The actual piece download speed.
    pub piece_download_speed_kbps: f32,
    /// Estimated time of arrival (completion)
    pub eta: i32,
    pub eta_idle: i32,
    /// Number of peers connected for this torrent.
    pub peers_connected: i32,
    pub peers_from: [i32; 7],
    /// Peers we are downloading from.
    pub peers_sending_to_us: i32,
    /// Peers we are uploading to.
    pub peers_getting_from_us: i32,
    /// Webseeds we are downlading from.
    pub webseeds_sending_to_us: i32,
    /// Size in bytes when completed.
    pub size_when_done: u64,
    /// Bytes until download is finished.
    pub left_until_done: u64,
    pub desired_available: u64,
    pub corrupt_ever: u64,
    pub uploaded_ever: u64,
    pub downloaded_ever: u64,
    pub have_valid: u64,
    pub have_unchecked: u64,
    pub manual_announce_time: NaiveDateTime,
    /// Seed ratio
    pub ratio: f32,
    /// Date and time added
    pub added_date: NaiveDateTime,
    /// Date and time finished
    pub done_date: NaiveDateTime,
    /// Date and time started
    pub start_date: NaiveDateTime,
    /// Date and time of last activity
    pub activity_date: NaiveDateTime,
    /// How long it has been idle
    pub idle_secs: i32,
    /// How long it has been downloading
    pub seconds_downloading: i32,
    /// How log it has been seeding
    pub seconds_seeding: i32,
    /// Is the torrent finished
    pub finished: bool,
    /// What position in the queue is the torrent
    pub queue_position: i32,
    /// Is the torrent stalled
    pub is_stalled: bool,
}

/// Converts tr_stat into TorrentStats
#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl From<*const transmission_sys::tr_stat> for TorrentStats {
    /// Convert from the unsafe generated type to the safe library type
    fn from(stat: *const transmission_sys::tr_stat) -> Self {
        let stat = unsafe { *stat };
        Self {
            id: stat.id,
            state: TorrentState::from(stat.activity),
            error: Error::from(stat.error),
            // Strings in C are awful and force use to do things like this
            error_string: {
                let slice = unsafe { &*(&stat.errorString[0..] as *const _ as *const [u8]) };
                if slice[0] == 0 {
                    String::new()
                } else {
                    String::from_utf8(slice.to_owned()).unwrap()
                }
            },
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
