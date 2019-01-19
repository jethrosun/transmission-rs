use std::ffi;
use std::fs::canonicalize;
use std::mem;
use std::path::PathBuf;

use transmission_sys;

use super::error::{Error, TrResult};
use super::torrent::Torrent;

/// Configuration for the torrent client made using a builder pattern.
pub struct ClientConfig {
    app_name: Option<String>,
    config_dir: Option<PathBuf>,
    download_dir: Option<PathBuf>,
    use_utp: bool,
    log_level: u32,
}

impl ClientConfig {
    pub fn new() -> Self {
        Self {
            app_name: None,
            config_dir: None,
            download_dir: None,
            use_utp: true,
            log_level: 0,
        }
    }

    pub fn app_name(mut self, name: &str) -> Self {
        self.app_name = Some(String::from(name));
        self
    }

    pub fn config_dir(mut self, dir: &str) -> Self {
        self.config_dir = Some(canonicalize(dir).unwrap());
        self
    }

    pub fn download_dir(mut self, dir: &str) -> Self {
        self.download_dir = Some(canonicalize(dir).unwrap());
        self
    }

    pub fn use_utp(mut self, utp: bool) -> Self {
        self.use_utp = utp;
        self
    }

    pub fn log_level(mut self, level: u32) -> Self {
        self.log_level = level;
        self
    }
}

/// Interface into the major functions of Transmission
/// including adding, querying, and removing torrents.
pub struct Client {
    session: *mut transmission_sys::tr_session,
}

impl Client {
    /// Creates a new [`Client`] after initializing the session.
    /// Takes in a path to the configuration directory.
    pub fn new(config: ClientConfig) -> Self {
        // Change things into the types needed
        let c_dir = config.config_dir.expect("Configuration directory not set!");
        let c_dir = ffi::CString::new(c_dir.to_str().unwrap()).unwrap();

        let app_name = config.app_name.expect("Application name not set!");
        let app_name = ffi::CString::new(app_name).unwrap();

        let d_dir = config.download_dir.expect("Download directory not set!");
        let d_dir = ffi::CString::new(d_dir.to_str().unwrap()).unwrap();

        let ses;
        unsafe {
            // Set log level
            transmission_sys::tr_logSetLevel(config.log_level);

            let mut set: transmission_sys::tr_variant = mem::uninitialized();
            transmission_sys::tr_variantInitDict(&mut set, 0);
            transmission_sys::tr_sessionLoadSettings(&mut set, c_dir.as_ptr(), app_name.as_ptr());
            ses = transmission_sys::tr_sessionInit(c_dir.as_ptr(), false, &mut set);
            transmission_sys::tr_variantFree(&mut set);

            // Apply the other settings
            transmission_sys::tr_sessionSetDownloadDir(ses, d_dir.as_ptr());
            transmission_sys::tr_sessionSetUTPEnabled(ses, config.use_utp);
        }
        Self { session: ses }
    }

    /// Adds a torrent using either a path or URL to a torrent file.
    pub fn add_torrent_file(&self, path: &str) -> TrResult<Torrent> {
        let path = canonicalize(path).unwrap();
        let path = ffi::CString::new(path.to_str().unwrap()).unwrap();
        unsafe {
            let ctor = transmission_sys::tr_ctorNew(self.session);
            match transmission_sys::tr_ctorSetMetainfoFromFile(ctor, path.as_ptr()) {
                0 => Torrent::from_ctor(ctor),
                _ => Err(Error::Unknown),
            }
        }
    }

    /// Adds a torrent to download using a magnet link.
    pub fn add_torrent_magnet(&mut self, link: &str) -> TrResult<Torrent> {
        let link = ffi::CString::new(link).unwrap();
        unsafe {
            let ctor = transmission_sys::tr_ctorNew(self.session);
            match transmission_sys::tr_ctorSetMetainfoFromMagnetLink(ctor, link.as_ptr()) {
                0 => Torrent::from_ctor(ctor),
                _ => Err(Error::Unknown),
            }
        }
    }

    /// Returns a list of current torrents
    pub fn list_torrents(&self) -> TrResult<Vec<Torrent>> {
        unimplemented!()
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        unsafe {
            transmission_sys::tr_sessionClose(self.session);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // These both lead to the same torrent
    const MAGNET: &str = "magnet:?xt=urn:btih:85b530631740384bee5eeceb181ee5abebde856f&dn=hello.txt&tr=udp%3a%2f%2ftracker.uw0.xyz%3a6969";
    const FILE_PATH: &str = "./hello.torrent";

    #[test]
    fn add_torrent_magnet() {
        std::fs::remove_dir_all("/tmp/tr-test-magnet").unwrap_or(());
        std::fs::create_dir("/tmp/tr-test-magnet").unwrap();
        let c = ClientConfig::new()
            .app_name("testing")
            .config_dir("/tmp/tr-test-magnet/")
            .download_dir("/tmp/tr-test-magnet/");
        Client::new(c).add_torrent_magnet(MAGNET).unwrap().stats();
    }

    #[test]
    fn add_torrent_file() {
        std::fs::remove_dir_all("/tmp/tr-test-file").unwrap_or(());
        std::fs::create_dir("/tmp/tr-test-file").unwrap();
        let c = ClientConfig::new()
            .app_name("testing")
            .config_dir("/tmp/tr-test-file/")
            .download_dir("/tmp/tr-test-file/");
        Client::new(c).add_torrent_file(FILE_PATH).unwrap().stats();
    }
}
