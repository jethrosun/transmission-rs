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
#[derive(Default)]
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
    log_level: i64,
}

impl ClientConfig {
    /// Create a new ClientConfig
    pub fn new() -> Self {
        Self {
            app_name: None,
            config_dir: None,
            download_dir: None,
            use_utp: true,
            log_level: 1,
        }
    }

    /// Set the application's name. Must be set.
    pub fn app_name(mut self, name: &str) -> Self {
        self.app_name = Some(String::from(name));
        self
    }

    /// Set the configuration directory path. Must be set.
    pub fn config_dir(mut self, dir: &str) -> Self {
        self.config_dir = Some(canonicalize(dir).unwrap());
        self
    }

    /// Set the download directory path. Must be set.
    pub fn download_dir(mut self, dir: &str) -> Self {
        self.download_dir = Some(canonicalize(dir).unwrap());
        self
    }

    /// Toggle using UTP. Defaults to `true`.
    pub fn use_utp(mut self, utp: bool) -> Self {
        self.use_utp = utp;
        self
    }

    /// Set the log level.
    ///
    /// - 0: No logging
    /// - 1: Errors
    /// - 2: Info
    /// - 3: Debug
    /// - 4: Everything
    /// Defaults to 1
    pub fn log_level(mut self, level: i64) -> Self {
        self.log_level = level;
        self
    }
}

/// Interface into the major functions of Transmission
/// including adding, and removing torrents.
///
/// The `Client` does not keep track of the created torrents itself.
///
/// Example of creating a session and adding a torrent and waiting for it to complete.
/// ```no_run
/// use transmission::{ ClientConfig, Client};
///
/// # let test_dir = "/tmp/tr-test-long";
/// # let config_dir = test_dir;
/// # let download_dir = test_dir;
/// let file_path = "./alpine.torrent";
///
/// # std::fs::create_dir(test_dir).unwrap();
///
/// let c = ClientConfig::new()
///    .app_name("testing")
///    .config_dir(config_dir)
///    .download_dir(download_dir);
/// let mut c = Client::new(c);
///
/// let t = c.add_torrent_file(file_path).unwrap();
/// t.start();
///
/// // Run until done
/// while t.stats().percent_complete < 1.0 {
///     print!("{:#?}\r", t.stats().percent_complete);
/// }
/// c.close();
///
/// # std::fs::remove_dir_all(test_dir).unwrap();
/// ```
pub struct Client {
    // tr_session: RwLock<mem::ManuallyDrop<transmission_sys::tr_session>>,
    tr_session: RwLock<NonNull<transmission_sys::tr_session>>,
    closed: bool,
}

impl Client {
    /// Creates a new `Client` and initializes the session.
    ///
    /// Takes a `ClientConfig` with the populated options.
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
            let mut set: transmission_sys::tr_variant = mem::uninitialized();
            transmission_sys::tr_variantInitDict(&mut set, 0);
            transmission_sys::tr_sessionLoadSettings(&mut set, c_dir.as_ptr(), app_name.as_ptr());

            // Set the log level
            transmission_sys::tr_variantDictAddInt(
                &mut set,
                transmission_sys::TR_KEY_message_level as usize,
                config.log_level,
            );

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

    /// Adds a torrent using a torrent file.
    ///
    /// Takes the path to the torrent file on the disk.
    ///
    /// ```
    /// use transmission::{ ClientConfig, Client};
    ///
    /// # let test_dir = "/tmp/tr-test-file";
    /// # let config_dir = test_dir;
    /// # let download_dir = test_dir;
    /// let file_path = "./alpine.torrent";
    ///
    /// # std::fs::create_dir(test_dir).unwrap();
    ///
    /// let c = ClientConfig::new()
    ///    .app_name("testing")
    ///    .config_dir(config_dir)
    ///    .download_dir(download_dir);
    /// let mut c = Client::new(c);
    ///
    /// let t = c.add_torrent_file(file_path).unwrap();
    ///
    /// c.close();
    ///
    /// # std::fs::remove_dir_all(test_dir).unwrap();
    /// ```
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

    /// Adds a torrent using a magnet link.
    ///
    /// Takes the magnet URI of the torrent.
    ///
    /// ```
    /// use transmission::{ ClientConfig, Client};
    ///
    /// # let test_dir = "/tmp/tr-test-magnet";
    /// # let config_dir = test_dir;
    /// # let download_dir = test_dir;
    ///
    /// # let magnet_uri = "magnet:?xt=urn:btih:f04905751c91af11a3745b1ce4500f4bf0de0d18&dn=alpine-extended-3.8.2-x86_64.iso&tr=http%3a%2f%2ftorrent.resonatingmedia.com%3a6969%2fannounce";
    /// # std::fs::create_dir(test_dir).unwrap();
    ///
    /// let c = ClientConfig::new()
    ///    .app_name("testing")
    ///    .config_dir(config_dir)
    ///    .download_dir(download_dir);
    /// let mut c = Client::new(c);
    ///
    /// let t = c.add_torrent_magnet(magnet_uri).unwrap();
    ///
    /// c.close();
    ///
    /// # std::fs::remove_dir_all(test_dir).unwrap();
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

    /// Gracefully closes the client ending the session.
    ///
    /// Always call this otherwise the client will `panic!` on drop in order to prevent issues.
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

    #[test]
    fn thread_safe() {
        let test_dir = "/tmp/tr-test-thread";

        std::fs::create_dir(test_dir).unwrap();

        let c = ClientConfig::new()
            .app_name("testing")
            .config_dir(test_dir)
            .download_dir(test_dir);

        let mut client = Client::new(c);

        thread::spawn(move || client.close());
        std::fs::remove_dir_all(test_dir).unwrap_or(());
    }
}
