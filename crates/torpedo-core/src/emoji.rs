//! The emoji data and the search over shortcodes.
//!
//! The data is gemoji's `db/emoji.json`, embedded in the binary. Each alias in
//! it is a shortcode. See `data/gemoji/README.md` for its source.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

use crate::config::Alias;

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

/// One candidate of a search: an emoji, or an alias's text, and the shortcode
/// or alias name that matched best.
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

    /// The text of the alias or shortcode that is exactly `query`, ignoring
    /// case. An alias wins over a shortcode of the same name.
    pub fn exact_with(&self, query: &str, aliases: &[Alias]) -> Option<String> {
        let query = query.to_lowercase();
        aliases
            .iter()
            .find(|a| a.name.to_lowercase() == query)
            .map(|a| a.text.clone())
            .or_else(|| self.exact(&query).map(str::to_string))
    }

    /// The emoji that match `query`, best first, over gemoji's shortcodes
    /// only.
    pub fn search(&self, query: &str) -> Vec<Match> {
        self.search_with(query, &[])
    }

    /// The emoji and texts that match `query` over the aliases and gemoji's
    /// shortcodes, best first.
    ///
    /// An exact name ranks first, then a name that starts with the query, then
    /// one with a `_`-separated word that starts with it, then one that holds
    /// it. A shortcode with the name of an alias does not match. Each text
    /// appears once, under its best name. Among equal ranks an alias comes
    /// first, in the order of `aliases`, and shortcodes keep gemoji's order.
    pub fn search_with(&self, query: &str, aliases: &[Alias]) -> Vec<Match> {
        let query = query.to_lowercase();
        if query.is_empty() {
            return Vec::new();
        }
        let hidden: Vec<String> = aliases.iter().map(|a| a.name.to_lowercase()).collect();
        // (rank, order, text, name): aliases take the orders before gemoji's.
        let mut found: Vec<(u8, usize, String, String)> = Vec::new();
        for (i, a) in aliases.iter().enumerate() {
            if let Some(r) = rank(&query, &a.name.to_lowercase()) {
                found.push((r, i, a.text.clone(), a.name.clone()));
            }
        }
        for (i, e) in self.entries.iter().enumerate() {
            let best = e
                .shortcodes
                .iter()
                .filter(|s| !hidden.contains(s))
                .filter_map(|s| rank(&query, s).map(|r| (r, s.as_str())))
                .min_by_key(|(r, _)| *r);
            if let Some((r, s)) = best {
                found.push((r, aliases.len() + i, e.emoji.clone(), s.to_string()));
            }
        }
        found.sort_by_key(|(r, order, _, _)| (*r, *order));
        let mut seen: Vec<String> = Vec::new();
        let mut out = Vec::new();
        for (_, _, text, name) in found {
            if seen.contains(&text) {
                continue;
            }
            seen.push(text.clone());
            out.push(Match {
                emoji: text,
                shortcode: name,
            });
        }
        out
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

    fn alias(name: &str, text: &str) -> Alias {
        Alias {
            name: name.to_string(),
            text: text.to_string(),
        }
    }

    fn search_with(q: &str, aliases: &[Alias]) -> Vec<(String, String)> {
        EmojiIndex::gemoji()
            .search_with(q, aliases)
            .into_iter()
            .map(|m| (m.emoji, m.shortcode))
            .collect()
    }

    #[test]
    fn an_exact_alias_ranks_before_prefix_matches() {
        assert_eq!(search_with("fi", &[alias("fi", "🇫🇮")])[0], pair("🇫🇮", "fi"));
    }

    #[test]
    fn aliases_come_first_among_equal_ranks() {
        let aliases = [alias("thumbsall", "🙌"), alias("thumbsboth", "👍👎")];
        assert_eq!(
            search_with("thumbs", &aliases),
            [
                pair("🙌", "thumbsall"),
                pair("👍👎", "thumbsboth"),
                pair("👍", "thumbsup"),
                pair("👎", "thumbsdown")
            ]
        );
    }

    #[test]
    fn an_alias_shows_its_name_and_ignores_case() {
        let aliases = [alias("lgtm", "👍")];
        assert_eq!(search_with("LGTM", &aliases)[0], pair("👍", "lgtm"));
        assert_eq!(search_with("lg", &aliases)[0], pair("👍", "lgtm"));
        let upper = [alias("LGTM", "👍")];
        assert_eq!(search_with("lgtm", &upper)[0], pair("👍", "LGTM"));
    }

    #[test]
    fn an_emoji_with_a_shortcode_and_an_alias_appears_once() {
        let r = search_with("fire", &[alias("fireball", "🔥")]);
        let fires: Vec<_> = r.iter().filter(|(e, _)| e == "🔥").collect();
        assert_eq!(fires, [&pair("🔥", "fire")]);
    }

    #[test]
    fn an_alias_hides_the_shortcode_of_its_name() {
        let aliases = [alias("fire", "🚒")];
        let r = search_with("fire", &aliases);
        assert_eq!(r[0], pair("🚒", "fire"));
        assert!(!r.contains(&pair("🔥", "fire")));
        let index = EmojiIndex::gemoji();
        assert_eq!(index.exact_with("fire", &aliases).as_deref(), Some("🚒"));
        assert_eq!(index.exact_with("FIRE", &[]).as_deref(), Some("🔥"));
    }

    #[test]
    fn a_query_that_matches_only_an_alias() {
        assert!(search("zzb").is_empty());
        assert_eq!(
            search_with("zzb", &[alias("zzbot", "🤖")]),
            [pair("🤖", "zzbot")]
        );
    }

    #[test]
    fn a_text_alias_is_its_own_entry() {
        let shrug = "¯\\_(ツ)_/¯";
        assert_eq!(
            search_with("shrug", &[alias("shrug", shrug)])[0],
            pair(shrug, "shrug")
        );
        assert_eq!(
            EmojiIndex::gemoji()
                .exact_with("shrug", &[alias("shrug", shrug)])
                .as_deref(),
            Some(shrug)
        );
    }
}
