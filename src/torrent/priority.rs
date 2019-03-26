use transmission_sys;

/// The priority of a torrent as either:
///
/// - High
/// - Normal
/// - Low
///
/// Priority does not directly affect download speed but
/// instead changes how the torrent will be queued compared to other torrents
pub enum Priority {
    Low = transmission_sys::TR_PRI_LOW as isize,
    Normal = transmission_sys::TR_PRI_NORMAL as isize,
    High = transmission_sys::TR_PRI_HIGH as isize,
}
