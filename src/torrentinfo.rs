use std::ffi;

use chrono::prelude::NaiveDateTime;
use serde::Serialize;
use transmission_sys;

// TODO fix hash fields they tend to be broken for unknown reasons

#[derive(Debug, Serialize)]
pub struct TorrentFile {
    length: u64,
    name: String,
    priority: i8,
    dnd: i8,
    is_renamed: bool,
    first_piece: u32,
    last_piece: u32,
    offset: u64,
}

impl From<transmission_sys::tr_file> for TorrentFile {
    fn from(file: transmission_sys::tr_file) -> Self {
        Self {
            length: file.length,
            name: unsafe { ffi::CStr::from_ptr(file.name).to_str().unwrap().to_owned() },
            priority: file.priority,
            dnd: file.dnd,
            is_renamed: file.is_renamed != 0,
            first_piece: file.firstPiece,
            last_piece: file.lastPiece,
            offset: file.offset,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TorrentPiece {
    time_checked: NaiveDateTime,
    hash: [u8; 20],
    priority: i8,
    dnd: i8,
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

#[derive(Debug, Serialize)]
pub struct TrackerInfo {
    tier: i32,
    announce: String,
    scrape: String,
    id: u32,
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

#[derive(Debug, Serialize)]
pub struct TorrentInfo {
    total_size: u64,
    original_name: String,
    name: String,
    torrent: String,
    webseeds: Vec<String>,
    comment: String,
    creator: String,
    files: Vec<TorrentFile>,
    pieces: Vec<TorrentPiece>,
    trackers: Vec<TrackerInfo>,
    date_created: NaiveDateTime,
    tracker_count: u32,
    webseed_count: u32,
    file_count: u32,
    piece_size: u32,
    piece_count: u32,
    hash: [u8; 20],
    hash_string: String,
    is_private: bool,
    is_folder: bool,
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
