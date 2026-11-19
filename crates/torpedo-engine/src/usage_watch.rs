//! Reads the usage file again when it changed, and writes it after a commit.
//!
//! The engine calls `check` where it checks the config file: at start, on
//! focus-in and when a search opens. A missing or bad file is no usage. A bad
//! file is logged once, and the next `record` first renames it to
//! `usage.toml.broken`, so a hand edit that went wrong is not lost silently.
//! After its own write the watch keeps the new fingerprint, so only a change
//! from outside, such as "Clear usage", reads the file again.

use std::fs;
use std::path::PathBuf;

use torpedo_core::usage::{self, Usage, broken_path};

use crate::fingerprint::{Fingerprint, fingerprint};

/// The usage file and the usage read from it.
pub struct UsageWatch {
    path: Option<PathBuf>,
    last: Option<Fingerprint>,
    usage: Usage,
    broken: bool,
    log: fn(String),
}

impl UsageWatch {
    /// A watch on `path`. With no path there is no usage, and nothing is
    /// written.
    pub fn new(path: Option<PathBuf>, log: fn(String)) -> UsageWatch {
        UsageWatch {
            path,
            last: None,
            usage: Usage::new(),
            broken: false,
            log,
        }
    }

    /// The usage in use.
    pub fn usage(&self) -> &Usage {
        &self.usage
    }

    /// Read the file again when its fingerprint changed. Returns whether the
    /// usage may have changed.
    pub fn check(&mut self) -> bool {
        let Some(path) = self.path.clone() else {
            return false;
        };
        let now = fingerprint(&path);
        if self.last.as_ref() == Some(&now) {
            return false;
        }
        self.last = Some(now.clone());
        self.broken = false;
        self.usage = match now {
            Fingerprint::Missing => Usage::new(),
            Fingerprint::Unreadable(e) => {
                (self.log)(format!("usage {}: cannot read it: {e}", path.display()));
                Usage::new()
            }
            Fingerprint::File { .. } => match fs::read_to_string(&path) {
                Err(e) => {
                    (self.log)(format!("usage {}: cannot read it: {e}", path.display()));
                    Usage::new()
                }
                Ok(text) => match usage::parse(&text) {
                    Ok(u) => u,
                    Err(problems) => {
                        for p in problems {
                            (self.log)(format!("usage {}: {p}", path.display()));
                        }
                        (self.log)(format!(
                            "usage {}: no usage until the next commit replaces it",
                            path.display()
                        ));
                        self.broken = true;
                        Usage::new()
                    }
                },
            },
        };
        true
    }

    /// Record a commit of `text` at `now` (Unix seconds) and write the file.
    /// A broken file is first renamed to `usage.toml.broken`.
    pub fn record(&mut self, text: &str, now: u64) {
        self.usage.record(text, now);
        let Some(path) = self.path.clone() else {
            return;
        };
        if self.broken {
            let aside = broken_path(&path);
            match fs::rename(&path, &aside) {
                Ok(()) => (self.log)(format!(
                    "usage {}: kept the broken file as {}",
                    path.display(),
                    aside.display()
                )),
                Err(e) => (self.log)(format!(
                    "usage {}: cannot rename the broken file: {e}",
                    path.display()
                )),
            }
            self.broken = false;
        }
        match self.usage.save(&path) {
            Ok(()) => self.last = Some(fingerprint(&path)),
            Err(e) => (self.log)(format!("usage {}: cannot write it: {e}", path.display())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn quiet(_: String) {}

    fn dir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("torpedo-usage-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn read(path: &Path) -> Usage {
        usage::parse(&fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn a_new_file_is_read() {
        let d = dir("new");
        let path = d.join("usage.toml");
        let mut w = UsageWatch::new(Some(path.clone()), quiet);
        assert!(w.check(), "the first check reads");
        assert!(w.usage().is_empty());
        assert!(!w.check(), "no change, no read");
        let mut u = Usage::new();
        u.record("👎", 5);
        u.save(&path).unwrap();
        assert!(w.check());
        assert!(w.usage().get("👎").is_some());
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn a_deleted_file_is_no_usage() {
        let d = dir("deleted");
        let path = d.join("usage.toml");
        let mut w = UsageWatch::new(Some(path.clone()), quiet);
        w.check();
        w.record("👎", 5);
        fs::remove_file(&path).unwrap();
        assert!(w.check());
        assert!(w.usage().is_empty());
        w.record("🔥", 6);
        let u = read(&path);
        assert_eq!(u.entries().len(), 1);
        assert!(u.get("🔥").is_some());
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn its_own_write_is_not_a_change() {
        let d = dir("own");
        let path = d.join("usage.toml");
        let mut w = UsageWatch::new(Some(path.clone()), quiet);
        w.check();
        w.record("🔥", 5);
        assert!(!w.check());
        assert!(read(&path).get("🔥").is_some());
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn a_broken_file_is_kept_aside_by_the_next_commit() {
        let d = dir("broken");
        let path = d.join("usage.toml");
        let aside = d.join("usage.toml.broken");
        fs::write(&path, "[[used]").unwrap();
        fs::write(&aside, "older").unwrap();
        let mut w = UsageWatch::new(Some(path.clone()), quiet);
        w.check();
        assert!(w.usage().is_empty());
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "[[used]",
            "a check renames nothing"
        );
        w.record("🔥", 5);
        assert_eq!(fs::read_to_string(&aside).unwrap(), "[[used]");
        let u = read(&path);
        assert_eq!(u.entries().len(), 1);
        w.record("👍", 6);
        assert_eq!(fs::read_to_string(&aside).unwrap(), "[[used]", "only once");
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn a_missing_file_is_not_renamed() {
        let d = dir("missing");
        let path = d.join("usage.toml");
        let mut w = UsageWatch::new(Some(path.clone()), quiet);
        w.check();
        w.record("🔥", 5);
        assert!(!d.join("usage.toml.broken").exists());
        assert!(path.exists());
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn no_path_writes_nothing() {
        let mut w = UsageWatch::new(None, quiet);
        assert!(!w.check());
        w.record("🔥", 5);
        assert!(w.usage().get("🔥").is_some());
    }
}
