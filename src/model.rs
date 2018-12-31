//! Module containing the various types used in the library

/// Different kinds of errors that can be produced by Transmission
pub enum Error {}

/// Simple type for all results that use [model::Error]
pub type TrResult<T> = Result<T, Error>;
