//! The config file: where it lives, what it holds, and the rules for each
//! value.
//!
//! The engine reads the file with [`parse`]. The settings window edits it with
//! [`ConfigDocument`], which keeps comments and keys it does not know, and
//! checks the whole result with the same [`parse`] before it may save.
//!
//! ```toml
//! trigger = "::"
//!
//! [aliases]
//! lgtm = "👍"
//! shrug = '¯\_(ツ)_/¯'
//! ```

use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use toml_edit::{DocumentMut, Item, Table, TableLike, value};

/// The trigger when the config file sets none.
pub const DEFAULT_TRIGGER: &str = "::";
/// The most characters a trigger may have.
pub const MAX_TRIGGER_CHARS: usize = 3;
/// The most Unicode code points an alias text may have.
pub const MAX_ALIAS_TEXT_CHARS: usize = 16;
/// The key of the trigger.
pub const TRIGGER_KEY: &str = "trigger";
/// The table of the aliases.
pub const ALIASES_KEY: &str = "aliases";

/// A character the query accepts: ASCII letters, digits, `_`, `+` and `-`.
pub fn is_query_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '+' | '-')
}

/// A name for an emoji or a short text, from the `[aliases]` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alias {
    /// The name as the file writes it. The search ignores its case.
    pub name: String,
    /// The text that a commit of the alias gives.
    pub text: String,
}

/// The values of a good config file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub trigger: String,
    /// Sorted by lower-case name, which is the tie order of the ranking.
    pub aliases: Vec<Alias>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            trigger: DEFAULT_TRIGGER.to_string(),
            aliases: Vec::new(),
        }
    }
}

/// One thing wrong with a config file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// The key, such as `trigger` or `aliases.lgtm`. Empty when the file does
    /// not parse.
    pub key: String,
    /// The line in the file, counted from 1, when it is known.
    pub line: Option<usize>,
    /// What is wrong, in one sentence.
    pub message: String,
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(line) = self.line {
            write!(f, "line {line}: ")?;
        }
        if !self.key.is_empty() {
            write!(f, "{}: ", self.key)?;
        }
        write!(f, "{}", self.message)
    }
}

/// The key of an alias in a problem, such as `aliases.lgtm`.
pub fn alias_key(name: &str) -> String {
    format!("{ALIASES_KEY}.{name}")
}

/// Check a trigger: 1 to 3 characters, none a query character, whitespace or
/// a control character.
pub fn check_trigger(trigger: &str) -> Result<(), String> {
    let n = trigger.chars().count();
    if n == 0 {
        return Err("the trigger is empty".into());
    }
    if n > MAX_TRIGGER_CHARS {
        return Err(format!(
            "the trigger has {n} characters, and the most is {MAX_TRIGGER_CHARS}"
        ));
    }
    for c in trigger.chars() {
        if is_query_char(c) {
            return Err(format!(
                "the trigger may not hold {c:?}, because letters, digits, _, + and - belong to the query"
            ));
        }
        if c.is_whitespace() {
            return Err("the trigger may not hold whitespace".into());
        }
        if c.is_control() {
            return Err("the trigger may not hold a control character".into());
        }
    }
    Ok(())
}

/// Check an alias name: one or more query characters.
pub fn check_alias_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("the alias name is empty".into());
    }
    if let Some(c) = name.chars().find(|c| !is_query_char(*c)) {
        return Err(format!(
            "the alias name may not hold {c:?}. Use letters, digits, _, + and -"
        ));
    }
    Ok(())
}

/// Check an alias text: 1 to 16 code points, no control character.
pub fn check_alias_text(text: &str) -> Result<(), String> {
    let n = text.chars().count();
    if n == 0 {
        return Err("the alias text is empty".into());
    }
    if n > MAX_ALIAS_TEXT_CHARS {
        return Err(format!(
            "the alias text has {n} characters, and the most is {MAX_ALIAS_TEXT_CHARS}"
        ));
    }
    if text.chars().any(char::is_control) {
        return Err("the alias text may not hold a line break, a tab or another control character".into());
    }
    Ok(())
}

/// The path of the config file: `$XDG_CONFIG_HOME/torpedo/config.toml` when
/// that is an absolute path, else `$HOME/.config/torpedo/config.toml`. None
/// when neither variable helps.
pub fn config_path(env: impl Fn(&str) -> Option<OsString>) -> Option<PathBuf> {
    let base = env("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| {
            env("HOME")
                .map(PathBuf::from)
                .filter(|p| p.is_absolute())
                .map(|home| home.join(".config"))
        })?;
    Some(base.join("torpedo").join("config.toml"))
}

/// The config path from the process environment.
pub fn default_config_path() -> Option<PathBuf> {
    config_path(|k| std::env::var_os(k))
}

fn line_of(text: &str, span: Option<std::ops::Range<usize>>) -> Option<usize> {
    span.map(|s| text[..s.start.min(text.len())].matches('\n').count() + 1)
}

fn table_like(item: &Item) -> Option<&dyn TableLike> {
    item.as_table_like()
}

/// Parse and check the text of a config file. Keys and tables it does not
/// know are skipped. Any problem makes the whole file bad.
pub fn parse(text: &str) -> Result<Config, Vec<Problem>> {
    let doc = match toml_edit::Document::parse(text) {
        Ok(doc) => doc,
        Err(e) => {
            return Err(vec![Problem {
                key: String::new(),
                line: line_of(text, e.span()),
                message: format!("the file is not valid TOML: {}", e.message().trim()),
            }]);
        }
    };
    let mut problems = Vec::new();
    let mut config = Config::default();

    if let Some(item) = doc.get(TRIGGER_KEY) {
        let line = line_of(text, item.span());
        match item.as_str() {
            Some(t) => match check_trigger(t) {
                Ok(()) => config.trigger = t.to_string(),
                Err(message) => problems.push(Problem {
                    key: TRIGGER_KEY.into(),
                    line,
                    message,
                }),
            },
            None => problems.push(Problem {
                key: TRIGGER_KEY.into(),
                line,
                message: "the trigger is not a string".into(),
            }),
        }
    }

    if let Some(item) = doc.get(ALIASES_KEY) {
        match table_like(item) {
            None => problems.push(Problem {
                key: ALIASES_KEY.into(),
                line: line_of(text, item.span()),
                message: "aliases is not a table".into(),
            }),
            Some(table) => {
                let mut seen: Vec<String> = Vec::new();
                for (name, item) in table.iter() {
                    let line = line_of(text, item.span());
                    let key = alias_key(name);
                    let mut bad = |message: String| {
                        problems.push(Problem {
                            key: key.clone(),
                            line,
                            message,
                        })
                    };
                    if let Err(m) = check_alias_name(name) {
                        bad(m);
                        continue;
                    }
                    let lower = name.to_lowercase();
                    if seen.contains(&lower) {
                        bad(format!(
                            "another alias has the name {name:?} in another case, and the search ignores case"
                        ));
                        continue;
                    }
                    seen.push(lower);
                    match item.as_str() {
                        None => bad("the alias text is not a string".into()),
                        Some(t) => match check_alias_text(t) {
                            Err(m) => bad(m),
                            Ok(()) => config.aliases.push(Alias {
                                name: name.to_string(),
                                text: t.to_string(),
                            }),
                        },
                    }
                }
            }
        }
    }

    if !problems.is_empty() {
        return Err(problems);
    }
    config.aliases.sort_by_key(|a| a.name.to_lowercase());
    Ok(config)
}

/// A config file to edit, with its comments and unknown keys kept.
#[derive(Debug, Clone)]
pub struct ConfigDocument {
    doc: DocumentMut,
}

/// Why an edit was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refused(pub Vec<Problem>);

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines: Vec<String> = self.0.iter().map(|p| p.to_string()).collect();
        write!(f, "{}", lines.join("\n"))
    }
}

impl std::error::Error for Refused {}

impl ConfigDocument {
    /// The document of this text. It refuses a text that does not parse,
    /// because editing it would drop what the user wrote.
    pub fn parse(text: &str) -> Result<ConfigDocument, Refused> {
        match text.parse::<DocumentMut>() {
            Ok(doc) => Ok(ConfigDocument { doc }),
            Err(_) => Err(Refused(parse(text).err().unwrap_or_default())),
        }
    }

    /// The document of the file at `path`. A missing file is an empty one.
    pub fn load(path: &Path) -> Result<ConfigDocument, LoadError> {
        match fs::read_to_string(path) {
            Ok(text) => ConfigDocument::parse(&text).map_err(LoadError::Refused),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(ConfigDocument {
                doc: DocumentMut::new(),
            }),
            Err(e) => Err(LoadError::Io(e.to_string())),
        }
    }

    /// The text of the document.
    pub fn text(&self) -> String {
        self.doc.to_string()
    }

    /// The config of the document, or its problems.
    pub fn config(&self) -> Result<Config, Vec<Problem>> {
        parse(&self.text())
    }

    /// The trigger as written, good or bad, when it is a string.
    pub fn raw_trigger(&self) -> Option<String> {
        self.doc.get(TRIGGER_KEY)?.as_str().map(str::to_string)
    }

    /// The aliases as written, good or bad, in file order. A text that is not
    /// a string shows as empty.
    pub fn raw_aliases(&self) -> Vec<Alias> {
        let Some(table) = self.doc.get(ALIASES_KEY).and_then(Item::as_table_like) else {
            return Vec::new();
        };
        table
            .iter()
            .map(|(name, item)| Alias {
                name: name.to_string(),
                text: item.as_str().unwrap_or_default().to_string(),
            })
            .collect()
    }

    /// Apply an edit only when the result is a good config file.
    fn try_edit(&mut self, edit: impl FnOnce(&mut DocumentMut)) -> Result<(), Refused> {
        let mut next = self.doc.clone();
        edit(&mut next);
        parse(&next.to_string()).map_err(Refused)?;
        self.doc = next;
        Ok(())
    }

    fn aliases_mut(doc: &mut DocumentMut) -> &mut dyn TableLike {
        if doc.get(ALIASES_KEY).and_then(Item::as_table_like).is_none() {
            doc.insert(ALIASES_KEY, Item::Table(Table::new()));
        }
        doc[ALIASES_KEY]
            .as_table_like_mut()
            .expect("aliases is a table")
    }

    /// The name of the alias that equals `name` without regard to case.
    fn find_alias(&self, name: &str) -> Option<String> {
        let lower = name.to_lowercase();
        self.raw_aliases()
            .into_iter()
            .map(|a| a.name)
            .find(|n| n.to_lowercase() == lower)
    }

    /// Set the trigger.
    pub fn set_trigger(&mut self, trigger: &str) -> Result<(), Refused> {
        check_trigger(trigger).map_err(|message| {
            Refused(vec![Problem {
                key: TRIGGER_KEY.into(),
                line: None,
                message,
            }])
        })?;
        self.try_edit(|doc| match doc.get_mut(TRIGGER_KEY) {
            Some(item) if item.is_value() => {
                let decor = item.as_value().map(|v| v.decor().clone());
                *item = value(trigger);
                if let (Some(decor), Some(v)) = (decor, item.as_value_mut()) {
                    *v.decor_mut() = decor;
                }
            }
            _ => {
                doc.insert(TRIGGER_KEY, value(trigger));
            }
        })
    }

    /// Add an alias, or change the text of the alias with this name in any
    /// case.
    pub fn set_alias(&mut self, name: &str, text: &str) -> Result<(), Refused> {
        check_one_alias(name, text)?;
        let existing = self.find_alias(name);
        self.try_edit(|doc| {
            let table = ConfigDocument::aliases_mut(doc);
            match existing {
                Some(old) if old == name => {
                    let item = table.get_mut(name).expect("the alias exists");
                    let decor = item.as_value().map(|v| v.decor().clone());
                    *item = value(text);
                    if let (Some(decor), Some(v)) = (decor, item.as_value_mut()) {
                        *v.decor_mut() = decor;
                    }
                }
                Some(old) => {
                    table.remove(&old);
                    table.insert(name, value(text));
                }
                None => {
                    table.insert(name, value(text));
                }
            }
        })
    }

    /// Rename the alias `old` to `new`, keeping its text.
    pub fn rename_alias(&mut self, old: &str, new: &str) -> Result<(), Refused> {
        let Some(found) = self.find_alias(old) else {
            return Err(Refused(vec![Problem {
                key: alias_key(old),
                line: None,
                message: "no alias has this name".into(),
            }]));
        };
        let text = self
            .raw_aliases()
            .into_iter()
            .find(|a| a.name == found)
            .map(|a| a.text)
            .unwrap_or_default();
        check_one_alias(new, &text)?;
        if found.to_lowercase() != new.to_lowercase() && self.find_alias(new).is_some() {
            return Err(Refused(vec![Problem {
                key: alias_key(new),
                line: None,
                message: "another alias has this name".into(),
            }]));
        }
        self.try_edit(|doc| {
            let table = ConfigDocument::aliases_mut(doc);
            table.remove(&found);
            table.insert(new, value(text.as_str()));
        })
    }

    /// Delete the alias with this name in any case. Deleting a name that no
    /// alias has changes nothing.
    pub fn remove_alias(&mut self, name: &str) -> Result<(), Refused> {
        let Some(found) = self.find_alias(name) else {
            return Ok(());
        };
        self.try_edit(|doc| {
            ConfigDocument::aliases_mut(doc).remove(&found);
        })
    }

    /// Write the document to `path` in one step: a temp file in the same
    /// folder, synced, then renamed over the file. A reader sees the old file
    /// or the new one, never part of one. It refuses a bad document.
    pub fn save(&self, path: &Path) -> Result<(), SaveError> {
        let text = self.text();
        parse(&text).map_err(|p| SaveError::Refused(Refused(p)))?;
        write_atomically(path, &text).map_err(|e| SaveError::Io(e.to_string()))
    }
}

fn check_one_alias(name: &str, text: &str) -> Result<(), Refused> {
    let key = alias_key(name);
    let problem = |message| {
        Refused(vec![Problem {
            key: key.clone(),
            line: None,
            message,
        }])
    };
    check_alias_name(name).map_err(problem)?;
    check_alias_text(text).map_err(problem)
}

/// Why a file could not be loaded for editing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    Refused(Refused),
    Io(String),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Refused(r) => write!(f, "{r}"),
            LoadError::Io(e) => write!(f, "{e}"),
        }
    }
}

/// Why a save failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveError {
    Refused(Refused),
    Io(String),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::Refused(r) => write!(f, "{r}"),
            SaveError::Io(e) => write!(f, "cannot write the config file: {e}"),
        }
    }
}

/// Write `text` to `path` with a temp file and a rename in the same folder.
pub fn write_atomically(path: &Path, text: &str) -> std::io::Result<()> {
    let dir = match path.parent() {
        Some(d) if !d.as_os_str().is_empty() => d.to_path_buf(),
        _ => PathBuf::from("."),
    };
    fs::create_dir_all(&dir)?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "config.toml".into());
    let tmp = dir.join(format!(".{name}.{}.tmp", std::process::id()));
    let result = (|| {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(text.as_bytes())?;
        f.sync_all()?;
        fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bad_keys(text: &str) -> Vec<String> {
        parse(text).unwrap_err().into_iter().map(|p| p.key).collect()
    }

    fn temp_dir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "torpedo-core-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn an_empty_file_is_the_defaults() {
        assert_eq!(parse("").unwrap(), Config::default());
        assert_eq!(Config::default().trigger, "::");
    }

    #[test]
    fn good_triggers() {
        for t in ["::", ":", ";;", ":;:", "§", "!!"] {
            assert_eq!(check_trigger(t), Ok(()), "{t}");
            assert_eq!(parse(&format!("trigger = {t:?}")).unwrap().trigger, t);
        }
    }

    #[test]
    fn bad_triggers() {
        for t in [
            "", ":a", "+", "-", "_", "1", ": ", " ", "::::", "\t", ":\u{7}",
        ] {
            assert!(check_trigger(t).is_err(), "{t:?}");
            let text = format!("trigger = {:?}", t).replace("\\u{7}", "\\u0007");
            assert_eq!(bad_keys(&text), ["trigger"], "{t:?}");
        }
    }

    #[test]
    fn a_trigger_that_is_not_a_string() {
        assert_eq!(bad_keys("trigger = 2"), ["trigger"]);
        assert_eq!(bad_keys("[trigger]\na = 1"), ["trigger"]);
    }

    #[test]
    fn aliases_are_read_and_sorted_by_lower_case_name() {
        let c = parse("[aliases]\nzz = \"🤖\"\nLGTM = \"👍\"\nshrug = '¯\\_(ツ)_/¯'\n\"+2\" = \"👍👍\"\n")
            .unwrap();
        let names: Vec<&str> = c.aliases.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["+2", "LGTM", "shrug", "zz"]);
        assert_eq!(c.aliases[2].text, "¯\\_(ツ)_/¯");
        assert_eq!(c.aliases[2].text.chars().count(), 9);
    }

    #[test]
    fn an_inline_aliases_table_works() {
        let c = parse("aliases = { lgtm = \"👍\" }").unwrap();
        assert_eq!(c.aliases.len(), 1);
    }

    #[test]
    fn bad_alias_names() {
        assert_eq!(
            bad_keys("[aliases]\n\"looks good\" = \"👍\""),
            ["aliases.looks good"]
        );
        assert_eq!(bad_keys("[aliases]\n\"\" = \"👍\""), ["aliases."]);
        assert_eq!(bad_keys("[aliases]\n\"é\" = \"👍\""), ["aliases.é"]);
        assert_eq!(
            bad_keys("[aliases]\nlgtm = \"👍\"\nLGTM = \"✅\""),
            ["aliases.LGTM"]
        );
        assert!(parse("[aliases]\n\"lgtm+\" = \"👍\"").is_ok());
    }

    #[test]
    fn bad_alias_texts() {
        assert_eq!(bad_keys("[aliases]\nlgtm = \"\""), ["aliases.lgtm"]);
        assert_eq!(
            bad_keys("[aliases]\nlong = \"abcdefghijklmnopq\""),
            ["aliases.long"]
        );
        assert!(parse("[aliases]\nsixteen = \"abcdefghijklmnop\"").is_ok());
        assert_eq!(bad_keys("[aliases]\ntwo = \"a\\nb\""), ["aliases.two"]);
        assert_eq!(bad_keys("[aliases]\ntab = \"a\\tb\""), ["aliases.tab"]);
        assert_eq!(bad_keys("[aliases]\none = 1"), ["aliases.one"]);
        assert_eq!(bad_keys("aliases = 1"), ["aliases"]);
    }

    #[test]
    fn the_longest_gemoji_emoji_fits_an_alias() {
        let longest = crate::emoji::EmojiIndex::gemoji()
            .entries()
            .iter()
            .map(|e| e.emoji.chars().count())
            .max()
            .unwrap();
        assert!(longest <= MAX_ALIAS_TEXT_CHARS);
    }

    #[test]
    fn one_bad_value_makes_the_whole_file_bad() {
        assert!(parse("trigger = \";;\"\n[aliases]\nlgtm = \"\"").is_err());
    }

    #[test]
    fn unknown_keys_and_tables_are_skipped() {
        let c = parse("skin_tone = \"medium\"\ntrigger = \";;\"\n[recents]\nlast = 1\n").unwrap();
        assert_eq!(c.trigger, ";;");
    }

    #[test]
    fn a_file_that_does_not_parse_names_its_line() {
        let p = parse("# one\ntrigger = \";;\n").unwrap_err();
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].key, "");
        assert_eq!(p[0].line, Some(2));
        assert!(p[0].to_string().starts_with("line 2: "));
    }

    #[test]
    fn a_bad_value_names_its_line() {
        let p = parse("# one\n\ntrigger = \"ab\"\n").unwrap_err();
        assert_eq!(p[0].line, Some(3));
        assert!(p[0].to_string().starts_with("line 3: trigger: "));
    }

    #[test]
    fn config_path_branches() {
        let env = |pairs: &'static [(&'static str, &'static str)]| {
            move |k: &str| {
                pairs
                    .iter()
                    .find(|(n, _)| *n == k)
                    .map(|(_, v)| OsString::from(*v))
            }
        };
        assert_eq!(
            config_path(env(&[("XDG_CONFIG_HOME", "/x"), ("HOME", "/h")])),
            Some(PathBuf::from("/x/torpedo/config.toml"))
        );
        assert_eq!(
            config_path(env(&[("HOME", "/h")])),
            Some(PathBuf::from("/h/.config/torpedo/config.toml"))
        );
        assert_eq!(
            config_path(env(&[("XDG_CONFIG_HOME", ""), ("HOME", "/h")])),
            Some(PathBuf::from("/h/.config/torpedo/config.toml"))
        );
        assert_eq!(
            config_path(env(&[("XDG_CONFIG_HOME", "rel"), ("HOME", "/h")])),
            Some(PathBuf::from("/h/.config/torpedo/config.toml"))
        );
        assert_eq!(config_path(env(&[])), None);
    }

    const KEPT: &str = "# my settings\nskin_tone = \"medium\" # later\ntrigger = \"::\"\n\n[aliases]\n# work\nlgtm = \"👍\"\n\n[recents]\nx = 1\n";

    fn kept(text: &str) {
        assert!(text.contains("# my settings\n"), "{text}");
        assert!(text.contains("skin_tone = \"medium\" # later"), "{text}");
        assert!(text.contains("# work\n"), "{text}");
        assert!(text.contains("[recents]\nx = 1"), "{text}");
    }

    #[test]
    fn edits_keep_comments_and_unknown_keys() {
        let mut d = ConfigDocument::parse(KEPT).unwrap();
        d.set_trigger(";;").unwrap();
        kept(&d.text());
        d.set_alias("ship", "🚀").unwrap();
        kept(&d.text());
        d.set_alias("lgtm", "✅").unwrap();
        kept(&d.text());
        d.rename_alias("ship", "boat").unwrap();
        d.remove_alias("boat").unwrap();
        kept(&d.text());
        let c = d.config().unwrap();
        assert_eq!(c.trigger, ";;");
        assert_eq!(
            c.aliases,
            [Alias {
                name: "lgtm".into(),
                text: "✅".into()
            }]
        );
    }

    #[test]
    fn refused_edits_change_nothing() {
        let mut d = ConfigDocument::parse(KEPT).unwrap();
        let before = d.text();
        assert_eq!(d.set_trigger("ab").unwrap_err().0[0].key, "trigger");
        assert_eq!(
            d.set_alias("looks good", "👍").unwrap_err().0[0].key,
            "aliases.looks good"
        );
        assert!(d.set_alias("x", "").is_err());
        assert!(d.rename_alias("nope", "x").is_err());
        d.set_alias("ship", "🚀").unwrap();
        assert!(d.rename_alias("ship", "LGTM").is_err());
        d.remove_alias("ship").unwrap();
        assert_eq!(d.text(), before);
    }

    #[test]
    fn set_alias_replaces_a_name_in_another_case() {
        let mut d = ConfigDocument::parse("[aliases]\nlgtm = \"👍\"\n").unwrap();
        d.set_alias("LGTM", "✅").unwrap();
        assert_eq!(d.raw_aliases().len(), 1);
        assert_eq!(d.config().unwrap().aliases[0].name, "LGTM");
    }

    #[test]
    fn an_edit_of_a_bad_file_waits_for_every_fix() {
        let mut d = ConfigDocument::parse("trigger = \"ab\"\n[aliases]\nx = \"\"\n").unwrap();
        assert_eq!(d.raw_trigger().as_deref(), Some("ab"));
        assert!(d.set_trigger(";;").is_err());
        assert!(d.remove_alias("x").is_err());
    }

    #[test]
    fn a_document_that_does_not_parse_is_refused() {
        assert_eq!(ConfigDocument::parse("trigger = \"").unwrap_err().0[0].line, Some(1));
    }

    #[test]
    fn text_emoji_and_quoted_names_round_trip() {
        let mut d = ConfigDocument::parse("").unwrap();
        d.set_alias("shrug", "¯\\_(ツ)_/¯").unwrap();
        d.set_alias("+2", "👍👍").unwrap();
        let c = parse(&d.text()).unwrap();
        assert_eq!(c.aliases[0].name, "+2");
        assert_eq!(c.aliases[1].text, "¯\\_(ツ)_/¯");
    }

    #[test]
    fn save_creates_the_folder_and_replaces_the_file() {
        use std::os::unix::fs::MetadataExt;
        let dir = temp_dir("save");
        let path = dir.join("torpedo").join("config.toml");
        let mut d = ConfigDocument::load(&path).unwrap();
        d.set_alias("lgtm", "👍").unwrap();
        d.save(&path).unwrap();
        let first = fs::metadata(&path).unwrap().ino();
        let mut d = ConfigDocument::load(&path).unwrap();
        d.set_trigger(";;").unwrap();
        d.save(&path).unwrap();
        assert_ne!(fs::metadata(&path).unwrap().ino(), first);
        let names: Vec<_> = fs::read_dir(path.parent().unwrap())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(names, [OsString::from("config.toml")]);
        let c = parse(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(c.trigger, ";;");
        assert_eq!(c.aliases.len(), 1);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn load_refuses_a_file_that_does_not_parse() {
        let dir = temp_dir("load");
        let path = dir.join("config.toml");
        fs::write(&path, "trigger = \"").unwrap();
        assert!(matches!(
            ConfigDocument::load(&path),
            Err(LoadError::Refused(_))
        ));
        fs::remove_dir_all(dir).unwrap();
    }
}
