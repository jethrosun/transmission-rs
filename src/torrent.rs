use std::ffi;
use std::path::PathBuf;
use std::ptr::{null_mut, NonNull};
use std::sync::RwLock;

use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use transmission_sys;

use crate::error::{Error, TrResult};
use crate::torrentbuilder::TorrentBuilder;
use crate::torrentinfo::TorrentInfo;
use crate::torrentstats::{TorrentState, TorrentStats};

// const MAGIC_NUMBER: u32 = 95549;

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

    pub fn info(&self) -> TorrentInfo {
        let tor = *self.tr_torrent.write().unwrap();
        let info;
        unsafe {
            info = transmission_sys::tr_torrentInfo(tor.as_ref());
        }
        TorrentInfo::from(info)
    }

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
        let mut state = serializer.serialize_struct("Torrent", 2)?;
        state.serialize_field("info", &self.info())?;
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
