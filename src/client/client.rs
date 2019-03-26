//! Client for download management.
use std::ffi;
use std::fs::canonicalize;
use std::ptr::NonNull;
use std::sync::{Arc, RwLock};
use transmission_sys;

use super::ClientConfig;
use crate::error::{Error, TrResult};
use crate::torrent::Torrent;

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
#[derive(Clone)]
pub struct Client {
    tr_session: Arc<RwLock<NonNull<transmission_sys::tr_session>>>,
}

impl Client {
    /// Creates a new `Client` and initializes the session.
    ///
    /// Takes a `ClientConfig` with the populated options.
    pub fn new(config: ClientConfig) -> Self {
        // Change things into the types needed
        let c_dir = config
            .config_dir
            .clone()
            .expect("Configuration directory not set!");
        let c_dir = ffi::CString::new(c_dir.to_str().unwrap()).unwrap();

        let app_name = config.app_name.clone().expect("Application name not set!");
        let app_name = ffi::CString::new(app_name).unwrap();

        let ses;
        unsafe {
            let mut set = config.to_variant();
            transmission_sys::tr_sessionLoadSettings(&mut set, c_dir.as_ptr(), app_name.as_ptr());

            ses = transmission_sys::tr_sessionInit(c_dir.as_ptr(), false, &mut set);
            transmission_sys::tr_variantFree(&mut set);
        }
        Self {
            tr_session: Arc::new(RwLock::new(NonNull::new(ses).unwrap())),
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
    pub fn add_torrent_file(&self, path: &str) -> TrResult<Torrent> {
        let path = canonicalize(path).unwrap();
        let path = ffi::CString::new(path.to_str().unwrap()).unwrap();

        let mut ses = self.tr_session.write().unwrap();
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
    pub fn add_torrent_magnet(&self, link: &str) -> TrResult<Torrent> {
        let link = ffi::CString::new(link).unwrap();
        let mut ses = self.tr_session.write().unwrap();
        let ctor;
        unsafe {
            ctor = transmission_sys::tr_ctorNew(ses.as_mut());
        }
        match unsafe { transmission_sys::tr_ctorSetMetainfoFromMagnetLink(ctor, link.as_ptr()) } {
            0 => Torrent::from_ctor(ctor),
            _ => Err(Error::Unknown),
        }
    }

    /// Consumes the Client and gracefully closes the session.
    ///
    /// This should always be called to ensure that the Client lasts as long as you intend.
    pub fn close(self) {}
}

impl Drop for Client {
    fn drop(&mut self) {
        // If this is the last reference
        if Arc::strong_count(&self.tr_session) == 1 {
            // Close the session
            let ses = self.tr_session.write().unwrap();
            unsafe {
                transmission_sys::tr_sessionClose(ses.as_ptr());
            }
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

        let client = Client::new(c);

        thread::spawn(move || client.close());
        std::fs::remove_dir_all(test_dir).unwrap_or(());
    }
}
