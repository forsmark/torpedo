//! Reads the config file again when it changed, without a thread.
//!
//! The engine calls `check` on focus-in and when a search opens. One `stat`
//! gives a fingerprint: the modification time in nanoseconds, the size, the
//! device and the inode. Only a new fingerprint reads and parses the file. A
//! bad file keeps the last good config and is logged once.

use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use torpedo_core::config::{self, Config};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Fingerprint {
    Missing,
    Unreadable(String),
    File {
        mtime_ns: i128,
        size: u64,
        dev: u64,
        ino: u64,
    },
}

fn fingerprint(path: &Path) -> Fingerprint {
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

/// The config file and the last good config read from it.
pub struct ConfigWatch {
    path: Option<PathBuf>,
    last: Option<Fingerprint>,
    config: Config,
    log: fn(String),
}

impl ConfigWatch {
    /// A watch on `path`. With no path, the config is always the defaults.
    pub fn new(path: Option<PathBuf>, log: fn(String)) -> ConfigWatch {
        ConfigWatch {
            path,
            last: None,
            config: Config::default(),
            log,
        }
    }

    /// The config in use.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Read the file again when its fingerprint changed. Returns the new
    /// config when it changed what is in use.
    pub fn check(&mut self) -> Option<Config> {
        let path = self.path.clone()?;
        let now = fingerprint(&path);
        if self.last.as_ref() == Some(&now) {
            return None;
        }
        self.last = Some(now.clone());
        let next = match now {
            Fingerprint::Missing => Config::default(),
            Fingerprint::Unreadable(e) => {
                (self.log)(format!("config {}: cannot read it: {e}", path.display()));
                return None;
            }
            Fingerprint::File { .. } => match fs::read_to_string(&path) {
                Err(e) => {
                    (self.log)(format!("config {}: cannot read it: {e}", path.display()));
                    return None;
                }
                Ok(text) => match config::parse(&text) {
                    Ok(c) => c,
                    Err(problems) => {
                        for p in problems {
                            (self.log)(format!("config {}: {p}", path.display()));
                        }
                        (self.log)(format!(
                            "config {}: kept the last good config",
                            path.display()
                        ));
                        return None;
                    }
                },
            },
        };
        if next == self.config {
            return None;
        }
        (self.log)(format!(
            "config {}: trigger {:?}, {} aliases",
            path.display(),
            next.trigger,
            next.aliases.len()
        ));
        self.config = next.clone();
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());

    fn record(line: String) {
        LOG.lock().unwrap().push(line);
    }

    fn dir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("torpedo-watch-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn write(path: &Path, text: &str) {
        config::write_atomically(path, text).unwrap();
    }

    #[test]
    fn a_changed_file_is_read_again() {
        let d = dir("changed");
        let path = d.join("config.toml");
        let mut w = ConfigWatch::new(Some(path.clone()), record);
        assert_eq!(w.check(), None, "a missing file is the defaults");
        write(&path, "trigger = \";;\"");
        assert_eq!(w.check().unwrap().trigger, ";;");
        assert_eq!(w.check(), None, "no change, no read");
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn a_rewrite_in_the_same_tick_is_seen() {
        let d = dir("tick");
        let path = d.join("config.toml");
        fs::write(&path, "trigger = \";;\"").unwrap();
        let mut w = ConfigWatch::new(Some(path.clone()), record);
        w.check();
        let before = fs::metadata(&path).unwrap().modified().unwrap();
        // Same inode, another size, and the old modification time.
        fs::write(&path, "trigger = \":\"\n\n").unwrap();
        fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(before)
            .unwrap();
        assert_eq!(w.check().unwrap().trigger, ":");
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn a_broken_file_keeps_the_last_good_config() {
        let d = dir("broken");
        let path = d.join("config.toml");
        write(&path, "[aliases]\nlgtm = \"👍\"");
        let mut w = ConfigWatch::new(Some(path.clone()), record);
        assert_eq!(w.check().unwrap().aliases.len(), 1);
        write(&path, "trigger = \"");
        assert_eq!(w.check(), None);
        assert_eq!(w.config().aliases.len(), 1);
        let log = LOG.lock().unwrap().join("\n");
        assert!(
            log.contains(&format!("config {}: line 1:", path.display())),
            "{log}"
        );
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn a_bad_value_at_start_means_defaults() {
        let d = dir("bad");
        let path = d.join("config.toml");
        write(&path, "trigger = \"ab\"");
        let mut w = ConfigWatch::new(Some(path.clone()), record);
        assert_eq!(w.check(), None);
        assert_eq!(w.config(), &Config::default());
        let log = LOG.lock().unwrap().join("\n");
        assert!(log.contains("trigger: "), "{log}");
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn a_deleted_file_means_defaults() {
        let d = dir("deleted");
        let path = d.join("config.toml");
        write(&path, "trigger = \";;\"");
        let mut w = ConfigWatch::new(Some(path.clone()), record);
        w.check();
        fs::remove_file(&path).unwrap();
        assert_eq!(w.check(), Some(Config::default()));
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn no_path_is_the_defaults() {
        let mut w = ConfigWatch::new(None, record);
        assert_eq!(w.check(), None);
        assert_eq!(w.config(), &Config::default());
    }
}
