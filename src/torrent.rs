use std::ffi;
use std::path::PathBuf;
use std::ptr::{null_mut, NonNull};
use std::sync::RwLock;

use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use transmission_sys;

use chrono::prelude::NaiveDateTime;

use crate::error::{Error, TrResult};
use crate::torrentbuilder::TorrentBuilder;

// const MAGIC_NUMBER: u32 = 95549;

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
    /// Convert from the unsafe generated type to the safe library type
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
    /// Estimated time of arrival (completion)
    pub eta: i32,
    pub eta_idle: i32,
    pub peers_connected: i32,
    pub peers_from: [i32; 7],
    pub peers_sending_to_us: i32,
    pub peers_getting_from_us: i32,
    pub webseeds_sending_to_us: i32,
    /// Size in bytes when completed
    pub size_when_done: u64,
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
                    "".to_owned()
                } else {
                    ffi::CStr::from_bytes_with_nul(slice)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_owned()
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

/// Representation of a torrent download.
pub struct Torrent {
    tr_torrent: RwLock<NonNull<transmission_sys::tr_torrent>>,
}

impl<'a> Torrent {
    /// Create a new torrent from a tr_ctor
    pub(crate) fn from_ctor(ctor: *mut transmission_sys::tr_ctor) -> TrResult<Self> {
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
            transmission_sys::tr_parse_result_TR_PARSE_OK => {
                let t = Self {
                    tr_torrent: RwLock::new(NonNull::new(tor).unwrap()),
                };
                Ok(t)
            }
            _ => Err(Error::Unknown),
        }
    }

    pub(crate) fn from_tr_torrent(tr_torrent: *mut transmission_sys::tr_torrent) -> TrResult<Self> {
        Ok(Self {
            tr_torrent: RwLock::new(NonNull::new(tr_torrent).unwrap()),
        })
    }

    /// Alias to `TorrentBuilder::new()`
    pub fn build() -> TorrentBuilder {
        TorrentBuilder::new()
    }

    /// Start or resume the torrent
    pub fn start(&self) {
        let mut tor = *self.tr_torrent.write().unwrap();
        unsafe {
            transmission_sys::tr_torrentStart(tor.as_mut());
        }
    }

    /// Stop (pause) the torrent
    pub fn stop(&self) {
        let mut tor = *self.tr_torrent.write().unwrap();
        unsafe {
            transmission_sys::tr_torrentStop(tor.as_mut());
        }
    }

    /// Removes a torrent from the downloads
    /// Consumes the Torrent
    pub fn remove(self, with_data: bool) {
        let mut tor = *self.tr_torrent.write().unwrap();
        unsafe {
            transmission_sys::tr_torrentRemove(tor.as_mut(), with_data, None);
        }
    }

    /// Verify the torrent
    // TODO callback function
    pub fn verify(&self) {
        let mut tor = *self.tr_torrent.write().unwrap();
        unsafe { transmission_sys::tr_torrentVerify(tor.as_mut(), None, null_mut()) }
    }

    //# The following functions get information about the torrent

    /// This torrent's name
    pub fn name(&self) -> &str {
        let tor = *self.tr_torrent.write().unwrap();
        unsafe {
            let c_str = transmission_sys::tr_torrentName(tor.as_ref());
            ffi::CStr::from_ptr(c_str).to_str().unwrap()
        }
    }

    /// The unique ID of the torrent
    pub fn id(&self) -> i32 {
        let tor = *self.tr_torrent.write().unwrap();
        unsafe { transmission_sys::tr_torrentId(tor.as_ref()) }
    }

    /// All the stats of the torrent as given by Transmission
    pub fn stats(&self) -> TorrentStats {
        let mut tor = *self.tr_torrent.write().unwrap();
        unsafe { TorrentStats::from(transmission_sys::tr_torrentStatCached(tor.as_mut())) }
    }

    // TODO torrent metadata info

    pub fn set_ratio(&self, limit: f64) {
        let mut tor = *self.tr_torrent.write().unwrap();
        // TODO does ratio mode need to be toggled?
        unsafe {
            transmission_sys::tr_torrentSetRatioLimit(tor.as_mut(), limit);
        }
    }

    pub fn set_download_dir(&self, download_dir: PathBuf) {
        let mut tor = *self.tr_torrent.write().unwrap();
        let d_dir = ffi::CString::new(download_dir.to_str().unwrap()).unwrap();
        unsafe {
            transmission_sys::tr_torrentSetDownloadDir(tor.as_mut(), d_dir.as_ptr());
        }
    }
}

impl serde::ser::Serialize for Torrent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Torrent", 1)?;
        state.serialize_field("stats", &self.stats())?;
        state.end()
    }
}

impl AsMut<Torrent> for Torrent {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

unsafe impl std::marker::Send for Torrent {}
unsafe impl std::marker::Sync for Torrent {}
