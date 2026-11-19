//! Find the IBus daemon's address the way libibus does.
//!
//! The order is `IBUS_ADDRESS`, then the file named by `IBUS_ADDRESS_FILE`,
//! then the default address file
//! `$XDG_CONFIG_HOME/ibus/bus/<machine-id>-<host>-<display>`
//! (`ibus_get_address` and `ibus_get_socket_path` in `src/ibusshare.c`).
//! The daemon does not pass `IBUS_ADDRESS` to the engines it starts, so an
//! engine usually reads a file.

use std::path::{Path, PathBuf};

/// What the lookup needs from the outside world. Tests pass a fake.
pub trait Host {
    /// An environment variable, or None when it is unset.
    fn var(&self, name: &str) -> Option<String>;
    /// The text of a file, or None when it cannot be read.
    fn read(&self, path: &Path) -> Option<String>;
    /// Whether a process with this PID is alive.
    fn alive(&self, pid: i32) -> bool;
}

/// The real environment, file system and process table.
pub struct RealHost;

impl Host for RealHost {
    fn var(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }
    fn read(&self, path: &Path) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }
    fn alive(&self, pid: i32) -> bool {
        Path::new(&format!("/proc/{pid}")).exists()
    }
}

/// The daemon's address, or a message that says why none was found.
pub fn ibus_address(host: &dyn Host) -> Result<String, String> {
    if let Some(address) = host.var("IBUS_ADDRESS").filter(|a| !a.is_empty()) {
        return Ok(address);
    }
    let path = address_file(host)?;
    let contents = host
        .read(&path)
        .ok_or_else(|| format!("cannot read the address file {}", path.display()))?;
    parse_address_file(&contents, host)
        .ok_or_else(|| format!("no live daemon in {}", path.display()))
}

/// The path of the address file: `IBUS_ADDRESS_FILE`, or the default.
pub fn address_file(host: &dyn Host) -> Result<PathBuf, String> {
    if let Some(file) = host.var("IBUS_ADDRESS_FILE").filter(|f| !f.is_empty()) {
        return Ok(PathBuf::from(file));
    }
    let config = match host.var("XDG_CONFIG_HOME").filter(|d| !d.is_empty()) {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(host.var("HOME").ok_or("HOME is not set")?).join(".config"),
    };
    let machine_id = ["/var/lib/dbus/machine-id", "/etc/machine-id"]
        .iter()
        .find_map(|p| host.read(Path::new(p)))
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty())
        .ok_or("no machine id")?;
    let (hostname, display) = display_parts(host);
    Ok(config
        .join("ibus")
        .join("bus")
        .join(format!("{machine_id}-{hostname}-{display}")))
}

/// The host and display parts of the file name, as `ibus_get_socket_path` makes them.
fn display_parts(host: &dyn Host) -> (String, String) {
    if let Some(wayland) = host.var("WAYLAND_DISPLAY") {
        return ("unix".to_string(), wayland);
    }
    let Some(display) = host.var("DISPLAY") else {
        return ("unix".to_string(), "0".to_string());
    };
    let (hostname, rest) = match display.split_once(':') {
        Some((h, r)) => (h.to_string(), r.to_string()),
        None => (display.clone(), "0".to_string()),
    };
    let number = rest.split('.').next().unwrap_or("").to_string();
    let hostname = if hostname.is_empty() {
        "unix".to_string()
    } else {
        hostname
    };
    (hostname, number)
}

/// The address in an address file, unless its daemon PID is dead.
fn parse_address_file(contents: &str, host: &dyn Host) -> Option<String> {
    let mut address = None;
    let mut pid = None;
    for line in contents.lines().filter(|l| !l.starts_with('#')) {
        if let Some(a) = line.strip_prefix("IBUS_ADDRESS=") {
            address = Some(a.trim().to_string());
        } else if let Some(p) = line.strip_prefix("IBUS_DAEMON_PID=") {
            pid = p.trim().parse::<i32>().ok();
        }
    }
    if let Some(pid) = pid
        && !host.alive(pid)
    {
        return None;
    }
    address.filter(|a| !a.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    struct Fake {
        vars: HashMap<&'static str, &'static str>,
        files: HashMap<PathBuf, &'static str>,
        dead: Vec<i32>,
    }

    impl Host for Fake {
        fn var(&self, name: &str) -> Option<String> {
            self.vars.get(name).map(|v| v.to_string())
        }
        fn read(&self, path: &Path) -> Option<String> {
            self.files.get(path).map(|v| v.to_string())
        }
        fn alive(&self, pid: i32) -> bool {
            !self.dead.contains(&pid)
        }
    }

    fn fake(vars: &[(&'static str, &'static str)]) -> Fake {
        let mut f = Fake {
            vars: vars.iter().copied().collect(),
            ..Fake::default()
        };
        f.files.insert(PathBuf::from("/etc/machine-id"), "abc123\n");
        f
    }

    const FILE: &str = "# comment\nIBUS_ADDRESS=unix:path=/tmp/x,guid=1\nIBUS_DAEMON_PID=42\n";

    #[test]
    fn ibus_address_wins() {
        let mut f = fake(&[
            ("IBUS_ADDRESS", "unix:path=/a"),
            ("IBUS_ADDRESS_FILE", "/f"),
        ]);
        f.files.insert(PathBuf::from("/f"), FILE);
        assert_eq!(ibus_address(&f).unwrap(), "unix:path=/a");
    }

    #[test]
    fn address_file_variable_comes_next() {
        let mut f = fake(&[("IBUS_ADDRESS_FILE", "/f")]);
        f.files.insert(PathBuf::from("/f"), FILE);
        assert_eq!(ibus_address(&f).unwrap(), "unix:path=/tmp/x,guid=1");
    }

    #[test]
    fn an_empty_ibus_address_does_not_count() {
        let mut f = fake(&[("IBUS_ADDRESS", ""), ("IBUS_ADDRESS_FILE", "/f")]);
        f.files.insert(PathBuf::from("/f"), FILE);
        assert_eq!(ibus_address(&f).unwrap(), "unix:path=/tmp/x,guid=1");
    }

    #[test]
    fn default_file_under_wayland() {
        let f = fake(&[("XDG_CONFIG_HOME", "/c"), ("WAYLAND_DISPLAY", "wayland-0")]);
        assert_eq!(
            address_file(&f).unwrap(),
            PathBuf::from("/c/ibus/bus/abc123-unix-wayland-0")
        );
    }

    #[test]
    fn default_file_reads_the_dbus_machine_id_first() {
        let mut f = fake(&[("XDG_CONFIG_HOME", "/c"), ("WAYLAND_DISPLAY", "w")]);
        f.files
            .insert(PathBuf::from("/var/lib/dbus/machine-id"), "dbusid\n");
        assert_eq!(
            address_file(&f).unwrap(),
            PathBuf::from("/c/ibus/bus/dbusid-unix-w")
        );
    }

    #[test]
    fn default_file_under_x11_local_display() {
        let f = fake(&[("XDG_CONFIG_HOME", "/c"), ("DISPLAY", ":0")]);
        assert_eq!(
            address_file(&f).unwrap(),
            PathBuf::from("/c/ibus/bus/abc123-unix-0")
        );
    }

    #[test]
    fn default_file_under_x11_remote_display_drops_the_screen() {
        let f = fake(&[("XDG_CONFIG_HOME", "/c"), ("DISPLAY", "host:1.0")]);
        assert_eq!(
            address_file(&f).unwrap(),
            PathBuf::from("/c/ibus/bus/abc123-host-1")
        );
    }

    #[test]
    fn default_config_folder_is_under_home() {
        let f = fake(&[("HOME", "/h"), ("WAYLAND_DISPLAY", "w")]);
        assert_eq!(
            address_file(&f).unwrap(),
            PathBuf::from("/h/.config/ibus/bus/abc123-unix-w")
        );
    }

    #[test]
    fn default_file_gives_the_address() {
        let mut f = fake(&[("XDG_CONFIG_HOME", "/c"), ("WAYLAND_DISPLAY", "w")]);
        f.files
            .insert(PathBuf::from("/c/ibus/bus/abc123-unix-w"), FILE);
        assert_eq!(ibus_address(&f).unwrap(), "unix:path=/tmp/x,guid=1");
    }

    #[test]
    fn a_dead_daemon_pid_makes_the_file_not_count() {
        let mut f = fake(&[("IBUS_ADDRESS_FILE", "/f")]);
        f.files.insert(PathBuf::from("/f"), FILE);
        f.dead.push(42);
        assert!(ibus_address(&f).is_err());
    }

    #[test]
    fn a_missing_file_is_an_error() {
        let f = fake(&[("IBUS_ADDRESS_FILE", "/missing")]);
        assert!(ibus_address(&f).unwrap_err().contains("/missing"));
    }
}
