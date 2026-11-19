//! The fingerprint of a file: one `stat` that tells whether it changed.
//!
//! The modification time in nanoseconds, the size, the device and the inode.
//! Size and inode catch two writes within one timestamp tick, and a rename
//! always gives a new inode.

use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// What one `stat` of a file found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fingerprint {
    Missing,
    Unreadable(String),
    File {
        mtime_ns: i128,
        size: u64,
        dev: u64,
        ino: u64,
    },
}

pub fn fingerprint(path: &Path) -> Fingerprint {
    match fs::metadata(path) {
        Ok(m) => Fingerprint::File {
            mtime_ns: m.mtime() as i128 * 1_000_000_000 + m.mtime_nsec() as i128,
            size: m.len(),
            dev: m.dev(),
            ino: m.ino(),
        },
        Err(e) if e.kind() == ErrorKind::NotFound => Fingerprint::Missing,
        Err(e) => Fingerprint::Unreadable(e.to_string()),
    }
}
