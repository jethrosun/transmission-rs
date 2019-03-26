//! The Torrent struct and related components.
use std::ffi;
use std::mem;
use std::path::PathBuf;
use std::ptr::{null, null_mut, NonNull};
use std::sync::{Arc, RwLock};

use serde::ser::{SerializeStruct, Serializer};
use transmission_sys;

use super::Priority;
use super::TorrentBuilder;
use super::TorrentInfo;
use super::TorrentStats;
use crate::error::{Error, ParseInt, TrResult};

// const MAGIC_NUMBER: u32 = 95549;

/// Representation of a torrent download.
///
/// Can be used to start, stop, or get the information of a torrent.
#[derive(Clone)]
pub struct Torrent {
    tr_torrent: Arc<RwLock<NonNull<transmission_sys::tr_torrent>>>,
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
        Error::from(error as ParseInt).to_result().and_then(|_| {
            Ok(Self {
                tr_torrent: Arc::new(RwLock::new(NonNull::new(tor).unwrap())),
            })
        })
    }

    pub(crate) fn from_tr_torrent(tr_torrent: *mut transmission_sys::tr_torrent) -> TrResult<Self> {
        Ok(Self {
            tr_torrent: Arc::new(RwLock::new(NonNull::new(tr_torrent).unwrap())),
        })
    }

    pub fn parse_torrent_file(path: &str) -> TrResult<TorrentInfo> {
        let path = ffi::CString::new(path).unwrap();
        unsafe {
            let ctor = transmission_sys::tr_ctorNew(null());
            let mut info: transmission_sys::tr_info = mem::uninitialized();
            match transmission_sys::tr_ctorSetMetainfoFromFile(ctor, path.as_ptr()) {
                0 => match transmission_sys::tr_torrentParse(ctor, &mut info) {
                    transmission_sys::tr_parse_result::TR_PARSE_OK => Ok(TorrentInfo::from(info)),
                    _ => Err(Error::ParseErr),
                },
                _ => Err(Error::ParseErr),
            }
        }
    }

    /// Alias to `TorrentBuilder::new()`
    pub fn build() -> TorrentBuilder {
        TorrentBuilder::new()
    }

    /// Start or resume the torrent
    pub fn start(&self) {
        let mut tor = self.tr_torrent.write().unwrap();
        unsafe {
            transmission_sys::tr_torrentStart(tor.as_mut());
        }
    }

    /// Stop (pause) the torrent
    pub fn stop(&self) {
        let mut tor = self.tr_torrent.write().unwrap();
        unsafe {
            transmission_sys::tr_torrentStop(tor.as_mut());
        }
    }

    /// Removes a torrent from the downloads
    /// Consumes the Torrent
    pub fn remove(self, with_data: bool) {
        let mut tor = Arc::try_unwrap(self.tr_torrent)
            .unwrap()
            .into_inner()
            .unwrap();
        unsafe {
            transmission_sys::tr_torrentRemove(tor.as_mut(), with_data, None);
        }
    }

    /// Verify the torrent
    // TODO callback function
    pub fn verify(&self) {
        let mut tor = self.tr_torrent.write().unwrap();
        unsafe { transmission_sys::tr_torrentVerify(tor.as_mut(), None, null_mut()) }
    }

    //# The following functions get information about the torrent

    /// This torrent's name
    pub fn name(&self) -> &str {
        let tor = self.tr_torrent.read().unwrap();
        unsafe {
            let c_str = transmission_sys::tr_torrentName(tor.as_ref());
            ffi::CStr::from_ptr(c_str).to_str().unwrap()
        }
    }

    /// The unique ID of the torrent
    pub fn id(&self) -> i32 {
        let tor = self.tr_torrent.read().unwrap();
        unsafe { transmission_sys::tr_torrentId(tor.as_ref()) }
    }

    /// The stats of the torrent as given by Transmission
    ///
    /// These are only available after the torrent has been added to a session
    pub fn stats(&self) -> TorrentStats {
        let mut tor = self.tr_torrent.write().unwrap();
        unsafe { TorrentStats::from(transmission_sys::tr_torrentStatCached(tor.as_mut())) }
    }

    /// The info of the torrent as given by Transmission
    ///
    /// This is available after the torrent has been parsed and does not need to
    /// be added to a session.
    pub fn info(&self) -> TorrentInfo {
        let tor = self.tr_torrent.read().unwrap();
        let info;
        unsafe {
            info = transmission_sys::tr_torrentInfo(tor.as_ref());
        }
        TorrentInfo::from(unsafe { *info })
    }

    /// Set the seed ratio of the torrent
    pub fn set_ratio(&mut self, limit: f64) {
        let mut tor = self.tr_torrent.write().unwrap();
        // Does ratio mode need to be toggled?
        unsafe {
            transmission_sys::tr_torrentSetRatioLimit(tor.as_mut(), limit);
        }
    }

    /// Set the download directory of the torrent
    pub fn set_download_dir(&mut self, download_dir: PathBuf) {
        let mut tor = self.tr_torrent.write().unwrap();
        let d_dir = ffi::CString::new(download_dir.to_str().unwrap()).unwrap();
        unsafe {
            transmission_sys::tr_torrentSetDownloadDir(tor.as_mut(), d_dir.as_ptr());
        }
    }

    /// Set the priority of the torrent
    ///
    /// See `Priority` for more information
    pub fn set_priority(&mut self, priority: Priority) {
        let mut tor = self.tr_torrent.write().unwrap();
        unsafe {
            transmission_sys::tr_torrentSetPriority(tor.as_mut(), priority as i8);
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
