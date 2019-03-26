//! Various structs containing Torrent information.
use std::ffi;

use chrono::prelude::NaiveDateTime;
use serde::{Deserialize, Serialize};
use transmission_sys;

use super::Priority;

/// A file that is part of a torrent.
#[derive(Debug, Serialize, Deserialize)]
pub struct TorrentFile {
    /// The length of the file in bytes
    pub length: u64,
    /// Name of the file
    pub name: String,
    /// Download priority of the file
    pub priority: Priority,
    pub dnd: i8,
    /// Was the file renamed?
    pub is_renamed: bool,
    pub first_piece: u32,
    pub last_piece: u32,
    pub offset: u64,
}

impl From<transmission_sys::tr_file> for TorrentFile {
    fn from(file: transmission_sys::tr_file) -> Self {
        Self {
            length: file.length,
            name: unsafe { ffi::CStr::from_ptr(file.name).to_str().unwrap().to_owned() },
            priority: Priority::from(file.priority),
            dnd: file.dnd,
            is_renamed: file.is_renamed != 0,
            first_piece: file.firstPiece,
            last_piece: file.lastPiece,
            offset: file.offset,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TorrentPiece {
    /// Last time the piece was checked
    pub time_checked: NaiveDateTime,
    pub hash: [u8; 20],
    /// Priority of the piece
    pub priority: i8,
    pub dnd: i8,
}

impl From<transmission_sys::tr_piece> for TorrentPiece {
    fn from(piece: transmission_sys::tr_piece) -> Self {
        Self {
            time_checked: NaiveDateTime::from_timestamp(piece.timeChecked, 0),
            hash: piece.hash,
            priority: piece.priority,
            dnd: piece.dnd,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackerInfo {
    pub tier: i32,
    pub announce: String,
    pub scrape: String,
    pub id: u32,
}

impl From<transmission_sys::tr_tracker_info> for TrackerInfo {
    fn from(tracker: transmission_sys::tr_tracker_info) -> Self {
        Self {
            tier: tracker.tier,
            announce: unsafe { ffi::CStr::from_ptr(tracker.announce) }
                .to_str()
                .unwrap()
                .to_owned(),
            scrape: unsafe { ffi::CStr::from_ptr(tracker.scrape) }
                .to_str()
                .unwrap()
                .to_owned(),
            id: tracker.id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TorrentInfo {
    /// Total download size in bytes
    pub total_size: u64,
    /// Original name of the torrent
    pub original_name: String,
    /// Name of the torrent
    pub name: String,
    pub torrent: String,
    /// Webseeds of the torrent
    pub webseeds: Vec<String>,
    /// Comment on the torrent
    pub comment: String,
    /// The torrent's creator
    pub creator: String,
    /// Files of the torrent
    pub files: Vec<TorrentFile>,
    /// Pieces of the torrent
    ///
    /// This is skipped in Serialization due to it's size.
    /// If you want it serialized you will have to do it manually.
    #[serde(skip)]
    pub pieces: Vec<TorrentPiece>,
    /// Trackers of the torrent
    pub trackers: Vec<TrackerInfo>,
    /// Date the torrent was created
    pub date_created: NaiveDateTime,
    /// Number of trackers
    pub tracker_count: u32,
    /// Number of webseeds
    pub webseed_count: u32,
    /// Number of files
    pub file_count: u32,
    /// Sice of pieces in bytes
    pub piece_size: u32,
    /// Number of pieces
    pub piece_count: u32,
    pub hash: [u8; 20],
    /// String hash of the torrent
    pub hash_string: String,
    pub is_private: bool,
    /// Is it a torrent of a folder?
    pub is_folder: bool,
}

impl From<transmission_sys::tr_info> for TorrentInfo {
    fn from(info: transmission_sys::tr_info) -> Self {
        Self {
            total_size: info.totalSize,
            original_name: unsafe { ffi::CStr::from_ptr(info.originalName) }
                .to_str()
                .unwrap()
                .to_owned(),
            name: unsafe { ffi::CStr::from_ptr(info.name) }
                .to_str()
                .unwrap()
                .to_owned(),
            torrent: {
                if info.torrent.is_null() {
                    String::new()
                } else {
                    unsafe { ffi::CStr::from_ptr(info.torrent) }
                        .to_str()
                        .unwrap_or("")
                        .to_owned()
                }
            },
            webseeds: unsafe {
                std::slice::from_raw_parts_mut(info.webseeds, info.webseedCount as usize)
            }
            .iter()
            .map(|p| {
                unsafe { ffi::CStr::from_ptr(*p) }
                    .to_str()
                    .unwrap_or("")
                    .to_owned()
            })
            .collect(),
            comment: unsafe { ffi::CStr::from_ptr(info.comment) }
                .to_str()
                .unwrap()
                .to_owned(),
            creator: unsafe { ffi::CStr::from_ptr(info.creator) }
                .to_str()
                .unwrap()
                .to_owned(),
            files: unsafe { std::slice::from_raw_parts_mut(info.files, info.fileCount as usize) }
                .iter()
                .map(|e| TorrentFile::from(*e))
                .collect(),
            pieces: unsafe {
                std::slice::from_raw_parts_mut(info.pieces, info.pieceCount as usize)
            }
            .iter()
            .map(|e| TorrentPiece::from(*e))
            .collect(),
            trackers: unsafe {
                std::slice::from_raw_parts_mut(info.trackers, info.trackerCount as usize)
            }
            .iter()
            .map(|e| TrackerInfo::from(*e))
            .collect(),
            date_created: NaiveDateTime::from_timestamp(info.dateCreated, 0),
            tracker_count: info.trackerCount,
            webseed_count: info.webseedCount,
            file_count: info.fileCount,
            piece_size: info.pieceSize,
            piece_count: info.pieceCount,
            hash: info.hash,
            hash_string: {
                let slice = unsafe { &*(&info.hashString[0..] as *const _ as *const [u8]) };
                if slice[0] == 0 {
                    String::new()
                } else {
                    ffi::CStr::from_bytes_with_nul(slice)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_owned()
                }
            },
            is_private: info.isPrivate,
            is_folder: info.isFolder,
        }
    }
}
