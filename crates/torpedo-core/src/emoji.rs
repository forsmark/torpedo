//! The emoji data and the search over shortcodes.
//!
//! The data is gemoji's `db/emoji.json`, embedded in the binary. Each alias in
//! it is a shortcode. See `data/gemoji/README.md` for its source.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

const GEMOJI: &str = include_str!("../data/gemoji/emoji.json");

/// One emoji with its shortcodes, in gemoji's order.
#[derive(Debug, Clone, Deserialize)]
pub struct Emoji {
    /// The emoji itself, such as "🔥".
    pub emoji: String,
    /// Its shortcodes, such as ["fire"].
    #[serde(rename = "aliases")]
    pub shortcodes: Vec<String>,
}

/// One candidate of a search: an emoji and the shortcode that matched best.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    pub emoji: String,
    pub shortcode: String,
}

/// All emoji, searchable by shortcode.
pub struct EmojiIndex {
    entries: Vec<Emoji>,
    exact: HashMap<String, usize>,
}

/// How well a shortcode matches a query. Lower is better.
fn rank(query: &str, shortcode: &str) -> Option<u8> {
    if shortcode == query {
        Some(0)
    } else if shortcode.starts_with(query) {
        Some(1)
    } else if shortcode
        .split('_')
        .skip(1)
        .any(|word| word.starts_with(query))
    {
        Some(2)
    } else if shortcode.contains(query) {
        Some(3)
    } else {
        None
    }
}

impl EmojiIndex {
    /// The index of gemoji's data, parsed once.
    pub fn gemoji() -> &'static EmojiIndex {
        static INDEX: OnceLock<EmojiIndex> = OnceLock::new();
        INDEX
            .get_or_init(|| EmojiIndex::from_json(GEMOJI).expect("the embedded gemoji data parses"))
    }

    /// An index of the emoji in a gemoji-shaped JSON array.
    pub fn from_json(json: &str) -> Result<EmojiIndex, serde_json::Error> {
        let entries: Vec<Emoji> = serde_json::from_str(json)?;
        let mut exact = HashMap::new();
        for (i, e) in entries.iter().enumerate() {
            for s in &e.shortcodes {
                exact.entry(s.clone()).or_insert(i);
            }
        }
        Ok(EmojiIndex { entries, exact })
    }

    /// Every emoji, in gemoji's order.
    pub fn entries(&self) -> &[Emoji] {
        &self.entries
    }

    /// The emoji whose shortcode is exactly `query`, ignoring case.
    pub fn exact(&self, query: &str) -> Option<&str> {
        self.exact
            .get(&query.to_lowercase())
            .map(|&i| self.entries[i].emoji.as_str())
    }

    /// The emoji that match `query`, best first.
    ///
    /// An exact shortcode ranks first, then a shortcode that starts with the
    /// query, then one with a `_`-separated word that starts with it, then one
    /// that holds it. Each emoji appears once, under its best shortcode, and
    /// ties keep gemoji's order.
    pub fn search(&self, query: &str) -> Vec<Match> {
        let query = query.to_lowercase();
        if query.is_empty() {
            return Vec::new();
        }
        let mut found: Vec<(u8, usize, &str)> = Vec::new();
        for (i, e) in self.entries.iter().enumerate() {
            let best = e
                .shortcodes
                .iter()
                .filter_map(|s| rank(&query, s).map(|r| (r, s.as_str())))
                .min_by_key(|(r, _)| *r);
            if let Some((r, s)) = best {
                found.push((r, i, s));
            }
        }
        found.sort_by_key(|(r, i, _)| (*r, *i));
        found
            .into_iter()
            .map(|(_, i, s)| Match {
                emoji: self.entries[i].emoji.clone(),
                shortcode: s.to_string(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn search(q: &str) -> Vec<(String, String)> {
        EmojiIndex::gemoji()
            .search(q)
            .into_iter()
            .map(|m| (m.emoji, m.shortcode))
            .collect()
    }

    fn pair(e: &str, s: &str) -> (String, String) {
        (e.to_string(), s.to_string())
    }

    #[test]
    fn the_data_holds_every_gemoji_entry() {
        assert_eq!(EmojiIndex::gemoji().entries().len(), 1870);
    }

    #[test]
    fn fire_ranks_exact_then_prefix_then_word_prefix() {
        let r = search("fire");
        assert_eq!(r[0], pair("🔥", "fire"));
        let names: Vec<&str> = r.iter().map(|(_, s)| s.as_str()).collect();
        assert_eq!(
            names,
            [
                "fire",
                "firefighter",
                "fire_engine",
                "fireworks",
                "firecracker",
                "fire_extinguisher",
                "heart_on_fire",
                "man_firefighter",
                "woman_firefighter"
            ]
        );
    }

    #[test]
    fn prefix_comes_before_substring() {
        let r = search("ire");
        let ireland = r.iter().position(|(_, s)| s == "ireland").unwrap();
        let tired = r.iter().position(|(_, s)| s == "tired_face").unwrap();
        assert!(ireland < tired);
        assert_eq!(ireland, 0);
    }

    #[test]
    fn ties_keep_gemoji_order() {
        assert_eq!(
            search("thumbs"),
            [pair("👍", "thumbsup"), pair("👎", "thumbsdown")]
        );
    }

    #[test]
    fn an_emoji_appears_once_under_its_best_shortcode() {
        let r = search("fist");
        let fists: Vec<_> = r.iter().filter(|(e, _)| e == "✊").collect();
        assert_eq!(fists, [&pair("✊", "fist")]);
    }

    #[test]
    fn plus_one_finds_thumbs_up() {
        assert_eq!(search("+1")[0], pair("👍", "+1"));
    }

    #[test]
    fn digits_and_underscores_match() {
        assert_eq!(search("1st_place")[0], pair("🥇", "1st_place_medal"));
    }

    #[test]
    fn case_is_ignored() {
        assert_eq!(search("FIRE")[0], pair("🔥", "fire"));
    }

    #[test]
    fn no_match_is_empty() {
        assert!(search("fireq").is_empty());
    }

    #[test]
    fn an_empty_query_matches_nothing() {
        assert!(search("").is_empty());
    }

    #[test]
    fn fi_has_more_than_one_page() {
        assert_eq!(search("fi").len(), 56);
    }

    #[test]
    fn exact_ignores_case() {
        assert_eq!(EmojiIndex::gemoji().exact("Fire"), Some("🔥"));
        assert_eq!(EmojiIndex::gemoji().exact("fir"), None);
    }
}
