//! Module containing the various types used in the library

/// Different kinds of errors that can be produced by Transmission
pub enum Error {
    /// For all errors with unknown causes
    Unknown,
    // Error in parsing a torrent
    ParseErr,
    // When parsing a torrent if it is a duplicate
    ParseDuplicate(i32),
}

/// Simple type for all results that use [model::Error]
pub type TrResult<T> = Result<T, Error>;
