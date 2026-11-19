//! Usage: which emoji and alias texts the user committed, and how often and
//! how lately.
//!
//! Each entry has a usage score and the time of its last commit. The score
//! halves every 14 days. A commit adds 1 to the score as it has faded by
//! then. The usage file holds at most 200 entries.
//!
//! ```toml
//! # Torpedo writes this file after each commit. It ranks the emoji you use.
//! [[used]]
//! text = "🔥"
//! score = 1.5
//! last = 1763560000
//! ```

use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use toml_edit::{ArrayOfTables, DocumentMut, Item, Table, value};

use crate::config::{Problem, write_atomically};

/// The most entries the usage file keeps.
pub const MAX_ENTRIES: usize = 200;
/// The time in which a usage score halves: 14 days, in seconds.
pub const HALF_LIFE_SECS: f64 = 14.0 * 24.0 * 60.0 * 60.0;
/// The array of tables that holds the entries.
pub const USED_KEY: &str = "used";

const HEADER: &str = "# Torpedo writes this file after each commit. It ranks the emoji you use.\n";

/// One text the user committed.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// The emoji without its skin tone, or the alias text.
    pub text: String,
    /// The usage score at the time `last`.
    pub score: f64,
    /// The Unix time of the last commit, in seconds.
    pub last: u64,
}

impl Entry {
    /// The usage score at `now`. A `now` before `last` counts as `last`.
    pub fn score_at(&self, now: u64) -> f64 {
        let age = now.saturating_sub(self.last) as f64;
        self.score * (-age / HALF_LIFE_SECS).exp2()
    }

    /// A number that orders entries as their scores do at any one time.
    /// All scores fade by the same factor, so the order needs no clock.
    pub fn rank_key(&self) -> f64 {
        self.score.log2() + self.last as f64 / HALF_LIFE_SECS
    }
}

/// The usage: at most one entry per text.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Usage {
    entries: Vec<Entry>,
}

impl Usage {
    /// No usage.
    pub fn new() -> Usage {
        Usage::default()
    }

    /// The entries, in no particular order.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Whether there is no entry.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The entry of `text`.
    pub fn get(&self, text: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.text == text)
    }

    /// Record a commit of `text` at `now`, and keep at most 200 entries by
    /// dropping the lowest scores at `now`. A tie drops the older one.
    pub fn record(&mut self, text: &str, now: u64) {
        match self.entries.iter_mut().find(|e| e.text == text) {
            Some(e) => {
                e.score = e.score_at(now) + 1.0;
                e.last = e.last.max(now);
            }
            None => self.entries.push(Entry {
                text: text.to_string(),
                score: 1.0,
                last: now,
            }),
        }
        if self.entries.len() > MAX_ENTRIES {
            self.sort_best_first(now);
            self.entries.truncate(MAX_ENTRIES);
        }
    }

    fn sort_best_first(&mut self, now: u64) {
        self.entries.sort_by(|a, b| {
            b.score_at(now)
                .total_cmp(&a.score_at(now))
                .then(b.last.cmp(&a.last))
                .then_with(|| a.text.cmp(&b.text))
        });
    }

    /// The order key of each text: a higher key ranks first.
    pub fn ranking(&self) -> HashMap<String, f64> {
        self.entries
            .iter()
            .map(|e| (e.text.clone(), e.rank_key()))
            .collect()
    }

    /// The text of the usage file, best entries first.
    pub fn to_text(&self) -> String {
        let mut sorted = self.clone();
        let now = self.entries.iter().map(|e| e.last).max().unwrap_or(0);
        sorted.sort_best_first(now);
        let mut doc = DocumentMut::new();
        let mut used = ArrayOfTables::new();
        for e in &sorted.entries {
            let mut t = Table::new();
            t.insert("text", value(e.text.as_str()));
            t.insert("score", value(e.score));
            t.insert("last", value(e.last.min(i64::MAX as u64) as i64));
            used.push(t);
        }
        doc.insert(USED_KEY, Item::ArrayOfTables(used));
        format!("{HEADER}{doc}")
    }

    /// Write the usage file in one step: a temp file in the same folder,
    /// then a rename.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        write_atomically(path, &self.to_text())
    }
}

fn problem(index: Option<usize>, message: impl Into<String>) -> Problem {
    Problem {
        key: match index {
            Some(i) => format!("{USED_KEY}[{i}]"),
            None => USED_KEY.to_string(),
        },
        line: None,
        message: message.into(),
    }
}

/// Parse the text of a usage file. A text that does not parse, or any bad
/// entry, makes the whole file bad. Keys it does not know are skipped.
pub fn parse(text: &str) -> Result<Usage, Vec<Problem>> {
    let doc = text.parse::<toml_edit::Document<String>>().map_err(|e| {
        vec![Problem {
            key: String::new(),
            line: None,
            message: format!("the file is not valid TOML: {}", e.message().trim()),
        }]
    })?;
    let mut usage = Usage::new();
    let Some(item) = doc.get(USED_KEY) else {
        return Ok(usage);
    };
    let tables: Vec<&dyn toml_edit::TableLike> = if let Some(aot) = item.as_array_of_tables() {
        aot.iter().map(|t| t as &dyn toml_edit::TableLike).collect()
    } else if let Some(arr) = item.as_array() {
        let mut out = Vec::new();
        for (i, v) in arr.iter().enumerate() {
            match v.as_inline_table() {
                Some(t) => out.push(t as &dyn toml_edit::TableLike),
                None => return Err(vec![problem(Some(i), "the entry is not a table")]),
            }
        }
        out
    } else {
        return Err(vec![problem(None, "used is not an array of tables")]);
    };
    let mut problems = Vec::new();
    for (i, t) in tables.into_iter().enumerate() {
        let text = t.get("text").and_then(Item::as_str);
        let score = t
            .get("score")
            .and_then(|s| s.as_float().or_else(|| s.as_integer().map(|n| n as f64)));
        let last = t.get("last").and_then(Item::as_integer);
        match (text, score, last) {
            (None, _, _) => problems.push(problem(Some(i), "the entry has no text")),
            (Some(""), _, _) => problems.push(problem(Some(i), "the text is empty")),
            (Some(t), _, _) if t.chars().any(char::is_control) => {
                problems.push(problem(Some(i), "the text holds a control character"))
            }
            (Some(_), s, _) if !s.is_some_and(|s| s.is_finite() && s >= 0.0) => {
                problems.push(problem(Some(i), "the score is not a number of 0 or more"))
            }
            (Some(_), _, l) if !l.is_some_and(|l| l >= 0) => problems.push(problem(
                Some(i),
                "last is not a whole number of seconds of 0 or more",
            )),
            (Some(text), Some(score), Some(last)) => {
                if usage.get(text).is_some() {
                    problems.push(problem(Some(i), format!("the text {text:?} repeats")));
                } else {
                    usage.entries.push(Entry {
                        text: text.to_string(),
                        score,
                        last: last as u64,
                    });
                }
            }
            _ => unreachable!("each missing value is handled above"),
        }
    }
    if problems.is_empty() {
        Ok(usage)
    } else {
        Err(problems)
    }
}

/// The path of the usage file: `$XDG_STATE_HOME/torpedo/usage.toml` when
/// that is an absolute path, else `$HOME/.local/state/torpedo/usage.toml`.
/// None when neither variable helps.
pub fn usage_path(env: impl Fn(&str) -> Option<OsString>) -> Option<PathBuf> {
    let base = env("XDG_STATE_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| {
            env("HOME")
                .map(PathBuf::from)
                .filter(|p| p.is_absolute())
                .map(|home| home.join(".local").join("state"))
        })?;
    Some(base.join("torpedo").join("usage.toml"))
}

/// The usage path from the process environment.
pub fn default_usage_path() -> Option<PathBuf> {
    usage_path(|k| std::env::var_os(k))
}

/// The path that a broken usage file is renamed to: `usage.toml.broken`
/// beside it.
pub fn broken_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".broken");
    path.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const DAY: u64 = 24 * 60 * 60;
    const NOW: u64 = 1_763_560_000;

    fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<OsString> + 'a {
        move |k| {
            pairs
                .iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| OsString::from(v))
        }
    }

    #[test]
    fn the_path_under_xdg_state_home() {
        assert_eq!(
            usage_path(env(&[("XDG_STATE_HOME", "/s"), ("HOME", "/h")])),
            Some(PathBuf::from("/s/torpedo/usage.toml"))
        );
    }

    #[test]
    fn the_default_path() {
        let want = Some(PathBuf::from("/h/.local/state/torpedo/usage.toml"));
        assert_eq!(usage_path(env(&[("HOME", "/h")])), want);
        assert_eq!(
            usage_path(env(&[("XDG_STATE_HOME", ""), ("HOME", "/h")])),
            want
        );
        assert_eq!(
            usage_path(env(&[("XDG_STATE_HOME", "rel"), ("HOME", "/h")])),
            want
        );
        assert_eq!(usage_path(env(&[])), None);
        assert_eq!(usage_path(env(&[("HOME", "rel")])), None);
    }

    #[test]
    fn the_broken_path() {
        assert_eq!(
            broken_path(Path::new("/s/torpedo/usage.toml")),
            PathBuf::from("/s/torpedo/usage.toml.broken")
        );
    }

    fn entry(text: &str, score: f64, last: u64) -> Entry {
        Entry {
            text: text.into(),
            score,
            last,
        }
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn a_score_halves_every_14_days() {
        let e = entry("🔥", 4.0, NOW);
        assert!(close(e.score_at(NOW), 4.0));
        assert!(close(e.score_at(NOW + 14 * DAY), 2.0));
        assert!(close(e.score_at(NOW + 28 * DAY), 1.0));
    }

    #[test]
    fn a_clock_that_went_back_counts_as_no_age() {
        assert!(close(entry("🔥", 3.0, NOW).score_at(NOW - DAY), 3.0));
    }

    #[test]
    fn a_new_entry_starts_at_one() {
        let mut u = Usage::new();
        u.record("🔥", NOW);
        assert_eq!(u.get("🔥"), Some(&entry("🔥", 1.0, NOW)));
    }

    #[test]
    fn a_commit_adds_to_the_faded_score() {
        let mut u = Usage {
            entries: vec![entry("🔥", 2.0, NOW - 14 * DAY)],
        };
        u.record("🔥", NOW);
        let e = u.get("🔥").unwrap();
        assert!(close(e.score, 2.0));
        assert_eq!(e.last, NOW);
    }

    #[test]
    fn the_limit_drops_the_lowest_score() {
        let mut u = Usage::new();
        for i in 0..MAX_ENTRIES - 1 {
            u.entries.push(entry(&format!("t{i}"), 2.0, NOW));
        }
        u.entries.push(entry("😀", 0.5, NOW));
        u.record("🔥", NOW);
        assert_eq!(u.entries().len(), MAX_ENTRIES);
        assert!(u.get("😀").is_none());
        assert!(u.get("🔥").is_some());
    }

    #[test]
    fn a_tie_at_the_limit_drops_the_older_entry() {
        let mut u = Usage::new();
        for i in 0..MAX_ENTRIES - 1 {
            u.entries.push(entry(&format!("t{i}"), 5.0, NOW));
        }
        // It fades to exactly 1 at NOW, the score of a new entry.
        u.entries.push(entry("old", 2.0, NOW - 14 * DAY));
        u.record("🔥", NOW);
        assert_eq!(u.entries().len(), MAX_ENTRIES);
        assert!(u.get("old").is_none());
        assert!(u.get("🔥").is_some());
    }

    #[test]
    fn the_ranking_orders_as_the_scores_do_at_any_time() {
        let u = Usage {
            entries: vec![
                entry("a", 4.0, NOW - 60 * DAY),
                entry("b", 1.0, NOW),
                entry("c", 3.0, NOW - 5 * DAY),
                entry("d", 0.4, NOW - DAY),
            ],
        };
        let keys = u.ranking();
        for now in [NOW, NOW + DAY, NOW + 100 * DAY, NOW + 1000 * DAY] {
            for x in u.entries() {
                for y in u.entries() {
                    if x.score_at(now) > y.score_at(now) * (1.0 + 1e-9) {
                        assert!(
                            keys[&x.text] > keys[&y.text],
                            "{} {} at {now}",
                            x.text,
                            y.text
                        );
                    }
                }
            }
        }
        assert!(keys["b"] > keys["a"]);
    }

    #[test]
    fn a_round_trip() {
        let mut u = Usage::new();
        u.record("🔥", NOW);
        u.record("¯\\_(ツ)_/¯", NOW - DAY);
        u.record("\"quoted\"", NOW);
        let text = u.to_text();
        assert!(text.starts_with("# Torpedo writes this file"));
        let back = parse(&text).unwrap();
        assert_eq!(back.entries().len(), 3);
        for e in u.entries() {
            assert_eq!(back.get(&e.text), Some(e));
        }
    }

    #[test]
    fn an_empty_or_missing_list_is_no_usage() {
        assert!(parse("").unwrap().is_empty());
        assert!(parse("other = 1\n").unwrap().is_empty());
        assert!(parse("used = []\n").unwrap().is_empty());
    }

    #[test]
    fn inline_tables_and_integer_scores_parse() {
        let u = parse("used = [{ text = \"🔥\", score = 2, last = 5, extra = true }]\n").unwrap();
        assert_eq!(u.get("🔥"), Some(&entry("🔥", 2.0, 5)));
    }

    fn bad(text: &str) -> String {
        let p = parse(text).unwrap_err();
        p.iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn bad_files() {
        assert!(bad("[[used]").contains("not valid TOML"));
        assert!(bad("used = 3\n").contains("not an array"));
        assert!(bad("used = [1]\n").contains("not a table"));
        assert!(bad("[[used]]\nscore = 1.0\nlast = 1\n").contains("no text"));
        assert!(bad("[[used]]\ntext = \"\"\nscore = 1.0\nlast = 1\n").contains("empty"));
        assert!(bad("[[used]]\ntext = \"a\\nb\"\nscore = 1.0\nlast = 1\n").contains("control"));
        assert!(bad("[[used]]\ntext = \"a\"\nlast = 1\n").contains("score"));
        assert!(bad("[[used]]\ntext = \"a\"\nscore = -1.0\nlast = 1\n").contains("score"));
        assert!(bad("[[used]]\ntext = \"a\"\nscore = nan\nlast = 1\n").contains("score"));
        assert!(bad("[[used]]\ntext = \"a\"\nscore = \"1\"\nlast = 1\n").contains("score"));
        assert!(bad("[[used]]\ntext = \"a\"\nscore = 1.0\n").contains("last"));
        assert!(bad("[[used]]\ntext = \"a\"\nscore = 1.0\nlast = -5\n").contains("last"));
        assert!(bad("[[used]]\ntext = \"a\"\nscore = 1.0\nlast = 1.5\n").contains("last"));
        assert!(
            bad("[[used]]\ntext = \"a\"\nscore = 1.0\nlast = 1\n[[used]]\ntext = \"a\"\nscore = 2.0\nlast = 1\n")
                .contains("repeats")
        );
    }

    #[test]
    fn a_file_of_more_than_200_entries_is_read_whole_and_cut_at_the_next_write() {
        let mut u = Usage::new();
        for i in 0..250 {
            u.entries.push(entry(&format!("t{i}"), 1.0 + i as f64, NOW));
        }
        let mut back = parse(&u.to_text()).unwrap();
        assert_eq!(back.entries().len(), 250);
        back.record("🔥", NOW);
        assert_eq!(back.entries().len(), MAX_ENTRIES);
        assert!(back.get("t249").is_some());
        assert!(back.get("t0").is_none());
    }

    #[test]
    fn save_replaces_the_file() {
        let dir = std::env::temp_dir().join(format!("tpd-usage-{}", std::process::id()));
        let path = dir.join("torpedo").join("usage.toml");
        let mut u = Usage::new();
        u.record("🔥", NOW);
        u.save(&path).unwrap();
        assert_eq!(parse(&fs::read_to_string(&path).unwrap()).unwrap(), u);
        let left: Vec<_> = fs::read_dir(path.parent().unwrap())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(left, vec![std::ffi::OsString::from("usage.toml")]);
        fs::remove_dir_all(&dir).unwrap();
    }
}
