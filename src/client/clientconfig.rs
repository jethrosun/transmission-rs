use std::fs::canonicalize;
use std::path::PathBuf;

// TODO expand on this to have all the options Transmission exposes
/// Configuration for the torrent client made using a builder pattern.
#[derive(Default)]
pub struct ClientConfig {
    /// The name of the client application
    pub(crate) app_name: Option<String>,
    /// The path to the configuration directory
    pub(crate) config_dir: Option<PathBuf>,
    /// The path to the download directory
    pub(crate) download_dir: Option<PathBuf>,
    /// Whether or not to use UTP
    pub(crate) use_utp: bool,
    /// What level of logging to use.
    pub(crate) log_level: i64,
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
