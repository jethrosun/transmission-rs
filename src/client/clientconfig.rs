use std::ffi::CString;
use std::fs::canonicalize;
use std::mem;
use std::path::PathBuf;

use transmission_sys;

// TODO expand on this to have all the options Transmission exposes
/// Configuration for the torrent client made using a builder pattern.
#[derive(Default)]
pub struct ClientConfig {
    /// The name of the client application
    pub(crate) app_name: Option<String>,
    /// The path to the configuration directory
    pub(crate) config_dir: Option<PathBuf>,
    /// The path to the download directory
    download_dir: Option<PathBuf>,
    /// Whether or not to use UTP
    use_utp: bool,
    /// What level of logging to use.
    log_level: i64,
    /// Is RPC enabled?
    rpc_enabled: bool,
    /// The URL the RPC will serve on
    rpc_url: Option<String>,
    /// The port the RPC will serve on
    rpc_port: Option<String>,
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
            rpc_enabled: false,
            rpc_url: None,
            rpc_port: None,
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

    pub fn rpc_enabled(mut self, rpc: bool) -> Self {
        self.rpc_enabled = rpc;
        self
    }

    pub fn rpc_url(mut self, rpc_url: String) -> Self {
        self.rpc_url = Some(rpc_url);
        self
    }

    pub fn rpc_port(mut self, rpc_port: String) -> Self {
        self.rpc_port = Some(rpc_port);
        self
    }

    pub(crate) unsafe fn to_variant(self) -> transmission_sys::tr_variant {
        let mut variant: transmission_sys::tr_variant = mem::uninitialized();
        transmission_sys::tr_variantInitDict(&mut variant, 0);

        // Set the download directory
        if let Some(download_dir) = self.download_dir {
            let d_dir = CString::new(download_dir.to_str().unwrap()).unwrap();
            transmission_sys::tr_variantDictAddStr(
                &mut variant,
                transmission_sys::TR_KEY_download_dir as usize,
                d_dir.as_ptr(),
            );
        }

        // Set the UTP
        transmission_sys::tr_variantDictAddBool(
            &mut variant,
            transmission_sys::TR_KEY_utp_enabled as usize,
            self.use_utp,
        );

        // Set the log level
        transmission_sys::tr_variantDictAddInt(
            &mut variant,
            transmission_sys::TR_KEY_message_level as usize,
            self.log_level,
        );

        // Set RPC
        transmission_sys::tr_variantDictAddBool(
            &mut variant,
            transmission_sys::TR_KEY_rpc_enabled as usize,
            self.rpc_enabled,
        );

        if self.rpc_enabled {
            // Set RPC URL
            if let Some(rpc_url) = self.rpc_url {
                let r_url = CString::new(rpc_url).unwrap();
                transmission_sys::tr_variantDictAddStr(
                    &mut variant,
                    transmission_sys::TR_KEY_rpc_url as usize,
                    r_url.as_ptr(),
                );
            }

            // Set RPC port
            if let Some(rpc_port) = self.rpc_port {
                let r_port = CString::new(rpc_port).unwrap();
                transmission_sys::tr_variantDictAddStr(
                    &mut variant,
                    transmission_sys::TR_KEY_rpc_port as usize,
                    r_port.as_ptr(),
                );
            }
        }

        variant
    }
}
