//! Module containing the various types used in the library

use std::error;
use std::fmt;

use serde::Serialize;

use transmission_sys;

/// Different kinds of errors that can be produced by Transmission
#[derive(Debug, Serialize)]
pub enum Error {
    /// A general state of non-error.
    /// If this is is ever unwrapped from a Result please file a bug report.
    NoError,
    /// For all errors with unknown causes
    Unknown,
    // Error in parsing a torrent
    ParseErr,
    // When parsing a torrent if it is a duplicate
    ParseDuplicate,
    /// Local error when getting a torrent's stats
    StatLocal,
    /// Tracker error when getting a torrent's stats
    StatTracker,
    /// Tracker warning when getting a torrent's stats
    StatTrackerWarn,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // I'm lazy so this is the same as the debug output
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {}

// TODO more from declarations
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

/// Simple type for all results that use `Error`
pub type TrResult<T> = Result<T, Error>;
