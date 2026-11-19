//! The mode without a window: checks and scripted edits of the config file.
//!
//! `torpedo-settings` parses its arguments before it touches GTK. With any of
//! `--check`, `--set-trigger`, `--set-alias` or `--remove-alias` it runs here
//! and exits, so it needs no display.

use std::fmt::Write as _;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use torpedo_core::config::{self, ConfigDocument, Problem};

pub const USAGE: &str = "\
Usage: torpedo-settings [--config <file>] [<action>...]

Without an action, open the settings window.

Actions, which open no window:
  --check                    check the config file. Exit 1 when it is bad.
  --set-trigger <trigger>    set the trigger
  --set-alias <name>=<text>  add an alias, or change its text
  --remove-alias <name>      delete an alias

Options:
  --config <file>            another config file than
                             $XDG_CONFIG_HOME/torpedo/config.toml
  -h, --help                 print this help
  --version                  print the version
";

/// One edit of the config file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    SetTrigger(String),
    SetAlias(String, String),
    RemoveAlias(String),
}

/// What the arguments ask for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Window {
        config: Option<PathBuf>,
    },
    Check {
        config: Option<PathBuf>,
    },
    Edit {
        config: Option<PathBuf>,
        edits: Vec<Edit>,
    },
    Help,
    Version,
}

/// Parse the arguments after the program name.
pub fn parse_args(args: &[String]) -> Result<Mode, String> {
    let mut config = None;
    let mut check = false;
    let mut edits = Vec::new();
    let mut it = args.iter();
    let value = |it: &mut std::slice::Iter<String>, flag: &str| {
        it.next()
            .cloned()
            .ok_or_else(|| format!("{flag} needs a value"))
    };
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Mode::Help),
            "--version" => return Ok(Mode::Version),
            "--config" => config = Some(PathBuf::from(value(&mut it, "--config")?)),
            "--check" => check = true,
            "--set-trigger" => edits.push(Edit::SetTrigger(value(&mut it, "--set-trigger")?)),
            "--set-alias" => {
                let v = value(&mut it, "--set-alias")?;
                let (name, text) = v
                    .split_once('=')
                    .ok_or_else(|| format!("--set-alias needs <name>=<text>, not {v:?}"))?;
                edits.push(Edit::SetAlias(name.to_string(), text.to_string()));
            }
            "--remove-alias" => edits.push(Edit::RemoveAlias(value(&mut it, "--remove-alias")?)),
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    match (check, edits.is_empty()) {
        (true, false) => Err("--check does not go with an edit".into()),
        (true, true) => Ok(Mode::Check { config }),
        (false, false) => Ok(Mode::Edit { config, edits }),
        (false, true) => Ok(Mode::Window { config }),
    }
}

/// The config file: the one `--config` names, or the default.
pub fn resolve(config: Option<PathBuf>) -> Result<PathBuf, String> {
    config.or_else(config::default_config_path).ok_or_else(|| {
        "cannot find the config file: neither XDG_CONFIG_HOME nor HOME is set".into()
    })
}

/// A problem as one line: `<path>:<line>: <key>: <problem>`.
pub fn problem_line(path: &Path, p: &Problem) -> String {
    let mut s = path.display().to_string();
    if let Some(line) = p.line {
        let _ = write!(s, ":{line}");
    }
    if !p.key.is_empty() {
        let _ = write!(s, ": {}", p.key);
    }
    let _ = write!(s, ": {}", p.message);
    s
}

/// The output and exit status of a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub status: i32,
    pub out: String,
    pub err: String,
}

impl Report {
    fn ok(out: String) -> Report {
        Report {
            status: 0,
            out,
            err: String::new(),
        }
    }

    fn fail(err: String) -> Report {
        Report {
            status: 1,
            out: String::new(),
            err,
        }
    }
}

/// Check the config file at `path`.
pub fn check(path: &Path) -> Report {
    match fs::read_to_string(path) {
        Err(e) if e.kind() == ErrorKind::NotFound => Report::ok(format!(
            "{}: ok (no file, so the defaults)\n",
            path.display()
        )),
        Err(e) => Report::fail(format!("{}: cannot read it: {e}\n", path.display())),
        Ok(text) => match config::parse(&text) {
            Ok(_) => Report::ok(format!("{}: ok\n", path.display())),
            Err(problems) => Report::fail(
                problems
                    .iter()
                    .map(|p| problem_line(path, p) + "\n")
                    .collect(),
            ),
        },
    }
}

/// Apply the edits in order to the config file at `path`, and save once. Any
/// refused edit leaves the file unchanged.
pub fn edit(path: &Path, edits: &[Edit]) -> Report {
    let mut doc = match ConfigDocument::load(path) {
        Ok(d) => d,
        Err(config::LoadError::Refused(r)) => {
            return Report::fail(r.0.iter().map(|p| problem_line(path, p) + "\n").collect());
        }
        Err(config::LoadError::Io(e)) => {
            return Report::fail(format!("{}: cannot read it: {e}\n", path.display()));
        }
    };
    for e in edits {
        let result = match e {
            Edit::SetTrigger(t) => doc.set_trigger(t),
            Edit::SetAlias(n, t) => doc.set_alias(n, t),
            Edit::RemoveAlias(n) => doc.remove_alias(n),
        };
        if let Err(r) = result {
            return Report::fail(r.0.iter().map(|p| problem_line(path, p) + "\n").collect());
        }
    }
    match doc.save(path) {
        Ok(()) => Report::ok(format!("{}: saved\n", path.display())),
        Err(e) => Report::fail(format!("{}: {e}\n", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    fn dir(name: &str) -> PathBuf {
        let d =
            std::env::temp_dir().join(format!("torpedo-settings-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn no_action_opens_the_window() {
        assert_eq!(parse_args(&[]), Ok(Mode::Window { config: None }));
        assert_eq!(
            parse_args(&args(&["--config", "a.toml"])),
            Ok(Mode::Window {
                config: Some("a.toml".into())
            })
        );
    }

    #[test]
    fn actions_parse_in_order() {
        assert_eq!(
            parse_args(&args(&[
                "--set-trigger",
                ";;",
                "--set-alias",
                "eq==",
                "--remove-alias",
                "x"
            ])),
            Ok(Mode::Edit {
                config: None,
                edits: vec![
                    Edit::SetTrigger(";;".into()),
                    Edit::SetAlias("eq".into(), "=".into()),
                    Edit::RemoveAlias("x".into())
                ]
            })
        );
        assert_eq!(
            parse_args(&args(&["--check"])),
            Ok(Mode::Check { config: None })
        );
    }

    #[test]
    fn bad_arguments_are_refused() {
        assert!(parse_args(&args(&["--set-alias", "noequals"])).is_err());
        assert!(parse_args(&args(&["--set-trigger"])).is_err());
        assert!(parse_args(&args(&["--check", "--remove-alias", "x"])).is_err());
        assert!(parse_args(&args(&["--frobnicate"])).is_err());
        assert_eq!(parse_args(&args(&["--help"])), Ok(Mode::Help));
    }

    #[test]
    fn check_reports_each_problem_with_its_line() {
        let d = dir("check");
        let path = d.join("config.toml");
        assert_eq!(check(&path).status, 0);
        fs::write(&path, "# x\ntrigger = \"ab\"\n").unwrap();
        let r = check(&path);
        assert_eq!(r.status, 1);
        assert_eq!(
            r.err.lines().next().unwrap(),
            format!(
                "{}:2: trigger: {}",
                path.display(),
                config::check_trigger("ab").unwrap_err()
            )
        );
        fs::write(&path, "trigger = \";;\"\n").unwrap();
        assert_eq!(check(&path).out, format!("{}: ok\n", path.display()));
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn a_refused_edit_writes_nothing() {
        let d = dir("edit");
        let path = d.join("config.toml");
        fs::write(&path, "trigger = \";;\"\n").unwrap();
        let r = edit(
            &path,
            &[
                Edit::SetAlias("ok".into(), "👍".into()),
                Edit::SetTrigger("ab".into()),
            ],
        );
        assert_eq!(r.status, 1);
        assert!(r.err.contains(": trigger: "));
        assert_eq!(fs::read_to_string(&path).unwrap(), "trigger = \";;\"\n");
        let r = edit(&path, &[Edit::SetAlias("ok".into(), "👍".into())]);
        assert_eq!(r.status, 0, "{}", r.err);
        assert!(fs::read_to_string(&path).unwrap().contains("ok = \"👍\""));
        fs::remove_dir_all(d).unwrap();
    }
}
