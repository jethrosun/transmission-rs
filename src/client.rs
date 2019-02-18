use std::ffi;
use std::fs::canonicalize;
use std::mem;
use std::path::PathBuf;
use std::ptr::NonNull;
use std::sync::RwLock;
use transmission_sys;

use super::error::{Error, TrResult};
use super::torrent::Torrent;

// const MAGIC_NUMBER: u32 = 3245;

// TODO expand on this to have all the options Transmission exposes
/// Configuration for the torrent client made using a builder pattern.
pub struct ClientConfig {
    /// The name of the client application
    app_name: Option<String>,
    /// The path to the configuration directory
    config_dir: Option<PathBuf>,
    /// The path to the download directory
    download_dir: Option<PathBuf>,
    /// Whether or not to use UTP
    use_utp: bool,
    /// What level of logging to use.
    log_level: transmission_sys::tr_log_level,
}

impl ClientConfig {
    /// Create a new ClientConfig
    pub fn new() -> Self {
        Self {
            app_name: None,
            config_dir: None,
            download_dir: None,
            use_utp: false,
            log_level: transmission_sys::tr_log_level::TR_LOG_ERROR,
        }
    }

    /// Set the application's name.
    pub fn app_name(mut self, name: &str) -> Self {
        self.app_name = Some(String::from(name));
        self
    }

    /// Set the configuration directory path.
    pub fn config_dir(mut self, dir: &str) -> Self {
        self.config_dir = Some(canonicalize(dir).unwrap());
        self
    }

    /// Set the download directory path.
    pub fn download_dir(mut self, dir: &str) -> Self {
        self.download_dir = Some(canonicalize(dir).unwrap());
        self
    }

    /// Toggle using UTP.
    /// Defaults to `true`.
    pub fn use_utp(mut self, utp: bool) -> Self {
        self.use_utp = utp;
        self
    }

    /// Set the log level.
    pub fn log_level(mut self, level: transmission_sys::tr_log_level) -> Self {
        self.log_level = level;
        self
    }
}

/// Interface into the major functions of Transmission
/// including adding, querying, and removing torrents.
pub struct Client {
    // tr_session: RwLock<mem::ManuallyDrop<transmission_sys::tr_session>>,
    tr_session: RwLock<NonNull<transmission_sys::tr_session>>,
    closed: bool,
}

impl Client {
    /// Creates a new `Client` after initializing the session.
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
        Self {
            tr_session: RwLock::new(NonNull::new(ses).unwrap()),
            closed: false,
        }
    }

    /// Adds a torrent using either a path or URL to a torrent file.
    pub fn add_torrent_file(&mut self, path: &str) -> TrResult<Torrent> {
        let path = canonicalize(path).unwrap();
        let path = ffi::CString::new(path.to_str().unwrap()).unwrap();

        let mut ses = *self.tr_session.write().unwrap();
        let ctor;
        unsafe {
            ctor = transmission_sys::tr_ctorNew(ses.as_mut());
        }
        match unsafe { transmission_sys::tr_ctorSetMetainfoFromFile(ctor, path.as_ptr()) } {
            0 => Torrent::from_ctor(ctor),
            _ => Err(Error::Unknown),
        }
    }

    /// Adds a torrent to download using a magnet link.
    pub fn add_torrent_magnet(&mut self, link: &str) -> TrResult<Torrent> {
        let link = ffi::CString::new(link).unwrap();
        let mut ses = *self.tr_session.write().unwrap();
        let ctor;
        unsafe {
            ctor = transmission_sys::tr_ctorNew(ses.as_mut());
        }
        match unsafe { transmission_sys::tr_ctorSetMetainfoFromMagnetLink(ctor, link.as_ptr()) } {
            0 => Torrent::from_ctor(ctor),
            _ => Err(Error::Unknown),
        }
    }

    /// Returns a list of current torrents.
    pub fn list_torrents(&self) -> Vec<TrResult<&mut Torrent>> {
        /*
        let ses = &mut **self.tr_session.write().unwrap();
        let tors: *mut *mut transmission_sys::tr_torrent;
        let mut err = 0;
        let mut len = 0;
        unsafe {
            tors = transmission_sys::tr_sessionGetTorrents(ses, &mut err);
            len =
            Vec::from_raw_parts(tors, len, len)
                .iter()
                .map(|e| Torrent::from_tr_torrent(*e))
                .collect()
        }
        */
        unimplemented!()
    }

    /// Gracefully closes the client ending the session.
    /// Always call this otherwise the client will panic on drop.
    pub fn close(&mut self) {
        let ses = *self.tr_session.write().unwrap();
        self.closed = true;
        unsafe {
            transmission_sys::tr_sessionClose(ses.as_ptr());
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        if !self.closed {
            panic!("Dropped an unclosed Client session. Please call Client.close().")
        }
    }
}

unsafe impl std::marker::Send for Client {}
unsafe impl std::marker::Sync for Client {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    // These both lead to the same torrent of Alpine Linux extended
    const MAGNET: &str = "magnet:?xt=urn:btih:f04905751c91af11a3745b1ce4500f4bf0de0d18&dn=alpine-extended-3.8.2-x86_64.iso&tr=http%3a%2f%2ftorrent.resonatingmedia.com%3a6969%2fannounce";
    const FILE_PATH: &str = "./alpine.torrent";

    // Try to add by magnet link
    #[test]
    fn add_torrent_magnet() {
        std::fs::remove_dir_all("/tmp/tr-test-magnet").unwrap_or(());
        std::fs::create_dir("/tmp/tr-test-magnet").unwrap();
        let c = ClientConfig::new()
            .app_name("testing")
            .config_dir("/tmp/tr-test-magnet/")
            .download_dir("/tmp/tr-test-magnet/");
        let mut c = Client::new(c);
        let t = c.add_torrent_magnet(MAGNET).unwrap();
        dbg!(t.info());
        dbg!(t.stats());
        c.close();
    }

    // Try to add by torrent file
    #[test]
    fn add_torrent_file() {
        std::fs::remove_dir_all("/tmp/tr-test-file").unwrap_or(());
        std::fs::create_dir("/tmp/tr-test-file").unwrap();
        let c = ClientConfig::new()
            .app_name("testing")
            .config_dir("/tmp/tr-test-file/")
            .download_dir("/tmp/tr-test-file/");
        let mut c = Client::new(c);
        let t = c.add_torrent_file(FILE_PATH).unwrap();
        dbg!(t.info());
        dbg!(t.stats());
        c.close();
    }

    #[test]
    fn thread_safe() {
        std::fs::remove_dir_all("/tmp/tr-test-threadsafe").unwrap_or(());
        std::fs::create_dir("/tmp/tr-test-threadsafe").unwrap();
        let c = ClientConfig::new()
            .app_name("testing")
            .config_dir("/tmp/tr-test-threadsafe/")
            .download_dir("/tmp/tr-test-threadsafe/");
        let mut client = Client::new(c);
        thread::spawn(move || client.close());
    }

    // Wait for download to finish
    #[test]
    #[ignore]
    fn download_torrent() {
        std::fs::remove_dir_all("/tmp/tr-test-dl").unwrap_or(());
        std::fs::create_dir("/tmp/tr-test-dl").unwrap();
        let c = ClientConfig::new()
            .app_name("testing")
            .config_dir("/tmp/tr-test-dl/")
            .download_dir("/tmp/tr-test-dl/");
        let mut c = Client::new(c);
        let t = c.add_torrent_file(FILE_PATH).unwrap();
        t.start();
        dbg!(t.stats());
        // Run until done
        while t.stats().percent_complete < 1.0 {
            print!("{:#?}\r", t.stats().percent_complete);
        }
        c.close();
    }
}
