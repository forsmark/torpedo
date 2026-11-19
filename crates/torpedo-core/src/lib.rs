//! The pure logic of torpedo: trigger detection, search and config.
//!
//! This crate uses no D-Bus and no GTK, so its tests run without a desktop.

pub mod boundary;
pub mod config;
pub mod emoji;
pub mod keys;
pub mod picker;

/// The version of torpedo, from the crate's manifest.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert!(!version().is_empty());
    }
}
