//! Module containing the various Error types used in the library.
use std::error;
use std::fmt;

use serde::{Deserialize, Serialize};

use transmission_sys;

/// Different kinds of errors that can be produced by Transmission
///
/// This enum acts as a general wrapper for errors. Most errors produced by
/// `transmission-sys` can be converted to this using `Error::From`.
#[derive(Debug, Serialize, Deserialize)]
pub enum Error {
    /// A general state of non-error.
    /// If this is is ever the `Err` of a `Result` please file a bug report.
    NoError,
    /// For all errors with unknown causes.
    Unknown,
    /// An error occured in file I/O.
    IOError,
    /// Error in parsing a torrent.
    ParseErr,
    /// When parsing a torrent if it is a duplicate.
    ParseDuplicate,
    /// Local error when getting a torrent's stats.
    StatLocal,
    /// Tracker error when getting a torrent's stats.
    StatTracker,
    /// Tracker warning when getting a torrent's stats.
    StatTrackerWarn,
    /// An error with the URL when getting metainfo.
    MakeMetaUrl,
    /// Getting metainfo was cancelled.
    MakeMetaCancelled,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // I'm lazy so this is the same as the debug output
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {}

impl From<transmission_sys::tr_stat_errtype> for Error {
    fn from(staterr: transmission_sys::tr_stat_errtype) -> Self {
        match staterr {
            transmission_sys::tr_stat_errtype::TR_STAT_OK => Error::NoError,
            transmission_sys::tr_stat_errtype::TR_STAT_LOCAL_ERROR => Error::StatLocal,
            transmission_sys::tr_stat_errtype::TR_STAT_TRACKER_ERROR => Error::StatTracker,
            transmission_sys::tr_stat_errtype::TR_STAT_TRACKER_WARNING => Error::StatTrackerWarn,
        }
    }
}

impl From<transmission_sys::tr_metainfo_builder_err> for Error {
    fn from(builderr: transmission_sys::tr_metainfo_builder_err) -> Self {
        match builderr {
            transmission_sys::tr_metainfo_builder_err::TR_MAKEMETA_OK => Error::NoError,
            transmission_sys::tr_metainfo_builder_err::TR_MAKEMETA_URL => Error::MakeMetaUrl,
            transmission_sys::tr_metainfo_builder_err::TR_MAKEMETA_CANCELLED => {
                Error::MakeMetaCancelled
            }
            transmission_sys::tr_metainfo_builder_err::TR_MAKEMETA_IO_READ => Error::IOError,
            transmission_sys::tr_metainfo_builder_err::TR_MAKEMETA_IO_WRITE => Error::IOError,
        }
    }
}

impl From<transmission_sys::tr_parse_result> for Error {
    fn from(parseerr: transmission_sys::tr_parse_result) -> Self {
        match parseerr {
            transmission_sys::tr_parse_result::TR_PARSE_OK => Error::NoError,
            transmission_sys::tr_parse_result::TR_PARSE_ERR => Error::ParseErr,
            transmission_sys::tr_parse_result::TR_PARSE_DUPLICATE => Error::ParseDuplicate,
        }
    }
}

// Let's us handle the way parse errors can be returned more specifically
pub(crate) type ParseInt = i32;

impl From<ParseInt> for Error {
    fn from(int: ParseInt) -> Self {
        match int {
            0 => Error::NoError,
            1 => Error::ParseErr,
            2 => Error::ParseDuplicate,
            _ => Error::Unknown,
        }
    }
}

impl Error {
    /// Converts the `Error` to a `TrResult` where `NoError` causes `Ok`.
    pub fn to_result(self) -> TrResult<()> {
        match self {
            Error::NoError => Ok(()),
            x => Err(x),
        }
    }
}

/// Simple type for all results that use `Error`.
pub type TrResult<T> = Result<T, Error>;
