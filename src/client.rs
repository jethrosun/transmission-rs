use std::ffi;
use std::fs::canonicalize;
use std::path::PathBuf;

use transmission_sys;

use super::error::TrResult;
use super::torrent::Torrent;

/// Configuration for the torrent client made using a builder pattern.
pub struct ClientConfig {
    app_name: Option<String>,
    config_dir: Option<PathBuf>,
    download_dir: Option<PathBuf>,
}

impl ClientConfig {
    pub fn new() -> Self {
        Self {
            app_name: None,
            config_dir: None,
            download_dir: None,
        }
    }

    pub fn app_name(mut self, name: &str) -> Self {
        self.app_name = Some(String::from(name));
        self
    }

    fn get_app_name(&self) -> &ffi::CStr {
        ffi::CStr::from_bytes_with_nul(self.app_name.unwrap().as_bytes()).unwrap()
    }

    pub fn config_dir(mut self, dir: &str) -> Self {
        self.config_dir = Some(canonicalize(dir).unwrap());
        self
    }

    fn get_config_dir(&self) -> &ffi::CStr {
        let c_dir = self.config_dir.unwrap();
        ffi::CStr::from_bytes_with_nul(c_dir.to_str().unwrap().as_bytes()).unwrap()
    }

    pub fn download_dir(mut self, dir: &str) -> Self {
        self.download_dir = Some(canonicalize(dir).unwrap());
        self
    }

    fn get_download_dir(&self) -> &ffi::CStr {
        let d_dir = self.download_dir.unwrap();
        ffi::CStr::from_bytes_with_nul(d_dir.to_str().unwrap().as_bytes()).unwrap()
    }
}

/// Interface into the major functions of Transmission
/// including adding, querying, and removing torrents.
pub struct Client {
    session: transmission_sys::tr_session,
}

impl Client {
    /// Creates a new [`Client`] after initializing the session.
    /// Takes in a path to the configuration directory.
    pub fn new(config: ClientConfig) -> Self {
        // shadows previous
        unsafe {
            let c_dir = config.get_config_dir();
            let set: &mut transmission_sys::tr_variant;
            transmission_sys::tr_variantInitDict(set, 0);
            transmission_sys::tr_sessionLoadSettings(
                set,
                c_dir.as_ptr(),
                config.get_app_name().as_ptr(),
            );
            let ses = transmission_sys::tr_sessionInit(c_dir.as_ptr(), 0, set);
            Self { session: *ses }
        }
    }

    /// Adds a torrent using either a path or URL to a torrent file.
    pub fn add_torrent_file(&self, file: &str) -> TrResult<Torrent> {
        unimplemented!()
    }

    /// Adds a torrent to download using a magnet link.
    pub fn add_torrent_magnet(&self, link: &str) -> TrResult<Torrent> {
        unimplemented!()
    }

    /// Returns a list of current torrents
    pub fn torrents(&self) -> TrResult<Vec<Torrent>> {
        unimplemented!()
    }
}
