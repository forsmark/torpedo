//! The inline picker: the trigger, the held text, the search and dead keys.
//!
//! `Picker` takes one event at a time and returns an `Outcome`: the actions
//! the engine sends to IBus, in order, and whether the key was consumed. It
//! knows nothing of D-Bus. The design of the change `add-emoji-engine` holds
//! the table of what each key does in each state.

use std::collections::HashMap;

use crate::boundary::Context;
use crate::config::{Config, is_query_char};
use crate::emoji::{EmojiIndex, Match};
use crate::keys::{self, Accent};
use crate::tone;
use crate::usage::Usage;

pub use crate::config::DEFAULT_TRIGGER;
/// The character that commits an exact shortcode or alias, whatever the
/// trigger is.
pub const CLOSING_CHAR: char = ':';
/// Candidates per page of the candidate list.
pub const PAGE_SIZE: usize = 9;

/// One thing the engine tells IBus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Commit text to the app.
    Commit(String),
    /// Show held text in the preedit, underlined, with mode COMMIT.
    ShowPreedit(String),
    /// Clear the preedit, with mode CLEAR.
    ClearPreedit,
    /// Show the candidate list with these texts and this selected index.
    ShowCandidates { items: Vec<String>, cursor: usize },
    /// Hide the candidate list.
    HideCandidates,
}

/// What came of one event.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Outcome {
    /// The actions to send, in order.
    pub actions: Vec<Action>,
    /// Whether the engine used the key. False lets the key reach the app.
    pub consumed: bool,
    /// The emoji, without its skin tone, or the alias text that this event
    /// committed from a search. None for plain text.
    pub used: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
    Idle,
    /// A proper prefix of `trigger` is held. The prefix keeps the trigger it
    /// was typed toward, so a config change cannot cut it short.
    Prefix {
        held: String,
        trigger: Vec<char>,
    },
    /// The trigger is held with a query. `trigger` is the trigger as typed.
    Search {
        trigger: String,
        query: String,
        matches: Vec<Match>,
        cursor: usize,
    },
    /// A dead key is held.
    Dead(Accent),
}

/// The text to commit and the verdict of the key being handled.
#[derive(Default)]
struct Step {
    commit: String,
    consumed: bool,
    used: Option<String>,
}

/// The inline picker for one engine.
pub struct Picker {
    trigger: Vec<char>,
    config: Config,
    index: &'static EmojiIndex,
    /// The usage key of each text with usage.
    ranking: HashMap<String, f64>,
    opened: bool,
    state: State,
    context: Context,
    shown_preedit: Option<String>,
    shown_candidates: Option<(Vec<String>, usize)>,
}

fn is_enter_or_tab(keyval: u32) -> bool {
    matches!(
        keyval,
        keys::RETURN | keys::KP_ENTER | keys::TAB | keys::ISO_LEFT_TAB
    )
}

impl Picker {
    /// A picker with this config over this emoji data.
    pub fn new(config: Config, index: &'static EmojiIndex) -> Picker {
        assert!(
            !config.trigger.is_empty(),
            "the trigger has at least one character"
        );
        Picker {
            trigger: config.trigger.chars().collect(),
            config,
            index,
            ranking: HashMap::new(),
            opened: false,
            state: State::Idle,
            context: Context::default(),
            shown_preedit: None,
            shown_candidates: None,
        }
    }

    /// A picker with the default config over gemoji.
    pub fn with_defaults() -> Picker {
        Picker::new(Config::default(), EmojiIndex::gemoji())
    }

    /// Use another config from now on. Held text keeps the trigger it was
    /// typed with, and an open query is searched again with the new aliases.
    pub fn set_config(&mut self, config: Config) -> Outcome {
        if config.trigger.is_empty() {
            return Outcome::default();
        }
        self.trigger = config.trigger.chars().collect();
        self.config = config;
        if let State::Search {
            query,
            matches,
            cursor,
            ..
        } = &mut self.state
        {
            *matches = self
                .index
                .search_ranked(query, &self.config.aliases, &self.ranking);
            *cursor = 0;
        }
        self.finish(Step::default())
    }

    /// Rank by this usage from now on. An open query is searched again.
    /// The engine passes no usage when `remember_usage` is off.
    pub fn set_usage(&mut self, usage: &Usage) -> Outcome {
        self.ranking = usage.ranking();
        if let State::Search {
            query,
            matches,
            cursor,
            ..
        } = &mut self.state
        {
            *matches = self
                .index
                .search_ranked(query, &self.config.aliases, &self.ranking);
            *cursor = 0;
        }
        self.finish(Step::default())
    }

    /// The config in use.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Whether the last key press opened a search.
    pub fn search_opened(&self) -> bool {
        self.opened
    }

    fn search(&self, query: &str) -> Vec<Match> {
        self.index
            .search_ranked(query, &self.config.aliases, &self.ranking)
    }

    /// The text of a match as the candidate list shows it and a commit gives
    /// it: in the skin tone when the match takes one.
    fn shown(&self, m: &Match) -> String {
        if m.toned {
            tone::apply(&m.emoji, self.config.skin_tone)
        } else {
            m.emoji.clone()
        }
    }

    /// Commit a match, and note its untoned text as used.
    fn commit_match(&mut self, m: &Match, step: &mut Step) {
        let text = self.shown(m);
        self.commit(&text, step);
        step.used = Some(m.emoji.clone());
    }

    fn open_search(&mut self, trigger: String) {
        self.opened = true;
        self.state = State::Search {
            trigger,
            query: String::new(),
            matches: Vec::new(),
            cursor: 0,
        };
    }

    /// A key press or release.
    pub fn key(&mut self, keyval: u32, state: u32) -> Outcome {
        self.opened = false;
        if keys::is_release(state) || keys::is_modifier_key(keyval) {
            return Outcome::default();
        }
        let mut step = Step::default();
        let current = std::mem::replace(&mut self.state, State::Idle);
        match current {
            State::Idle => self.idle_key(keyval, state, &mut step),
            State::Prefix { held, trigger } => {
                self.prefix_key(held, trigger, keyval, state, &mut step)
            }
            State::Search {
                trigger,
                query,
                matches,
                cursor,
            } => self.search_key(trigger, query, matches, cursor, keyval, state, &mut step),
            State::Dead(accent) => self.dead_key(accent, keyval, state, &mut step),
        }
        self.finish(step)
    }

    /// The app sent its surrounding text.
    pub fn surrounding_text(&mut self, text: &str, cursor: u32, anchor: u32) {
        self.context.set_surrounding(text, cursor, anchor);
    }

    /// Focus came in. The picker starts from nothing.
    pub fn focus_in(&mut self) -> Outcome {
        let out = self.drop_state();
        self.context.reset();
        out
    }

    /// Focus went out, the app reset the input context, or the engine was
    /// switched off. The daemon commits a visible preedit with mode COMMIT
    /// itself, so the picker only drops its state and commits nothing.
    pub fn drop_state(&mut self) -> Outcome {
        self.state = State::Idle;
        self.context.saw_unknown();
        self.finish(Step::default())
    }

    /// A click on the candidate at `index` of the shown page.
    pub fn candidate_clicked(&mut self, index: u32) -> Outcome {
        let mut step = Step::default();
        if let State::Search {
            matches, cursor, ..
        } = &self.state
        {
            let at = cursor - cursor % PAGE_SIZE + index as usize;
            if let Some(m) = matches.get(at).cloned() {
                self.state = State::Idle;
                self.commit_match(&m, &mut step);
            }
        }
        self.finish(step)
    }

    /// Move the selection by `delta` candidates, within the list. The panel
    /// calls this through CursorUp, CursorDown, PageUp and PageDown.
    pub fn move_selection(&mut self, delta: isize) -> Outcome {
        if let State::Search {
            matches, cursor, ..
        } = &mut self.state
            && !matches.is_empty()
        {
            let last = matches.len() as isize - 1;
            *cursor = (*cursor as isize + delta).clamp(0, last) as usize;
        }
        self.finish(Step::default())
    }

    fn commit(&mut self, text: &str, step: &mut Step) {
        step.commit.push_str(text);
        self.context.saw_text(text);
    }

    /// Let a character through. With text to commit, it joins that text, so
    /// the app gets both in one commit and in the right order.
    fn pass_char(&mut self, c: char, step: &mut Step) {
        if step.commit.is_empty() {
            step.consumed = false;
            self.context.saw_char(c);
        } else {
            step.consumed = true;
            self.commit(&c.to_string(), step);
        }
    }

    /// Let a key without a character through.
    fn pass_other(&mut self, step: &mut Step) {
        step.consumed = false;
        self.context.saw_unknown();
    }

    fn hold_dead(&mut self, accent: Accent, step: &mut Step) {
        self.state = State::Dead(accent);
        step.consumed = true;
    }

    fn idle_key(&mut self, keyval: u32, state: u32, step: &mut Step) {
        if keys::has_pass_through_modifier(state) {
            return self.pass_other(step);
        }
        if let Some(accent) = keys::accent(keyval) {
            return self.hold_dead(accent, step);
        }
        match keys::char_of(keyval) {
            Some(c) if c == self.trigger[0] && self.context.at_boundary() => {
                if self.trigger.len() == 1 {
                    self.open_search(c.to_string());
                } else {
                    self.state = State::Prefix {
                        held: c.to_string(),
                        trigger: self.trigger.clone(),
                    };
                }
                step.consumed = true;
            }
            Some(c) => self.pass_char(c, step),
            None => self.pass_other(step),
        }
    }

    fn prefix_key(
        &mut self,
        mut held: String,
        trigger: Vec<char>,
        keyval: u32,
        state: u32,
        step: &mut Step,
    ) {
        if keys::has_pass_through_modifier(state) {
            self.commit(&held, step);
            return self.pass_other(step);
        }
        if keyval == keys::BACKSPACE {
            held.pop();
            if !held.is_empty() {
                self.state = State::Prefix { held, trigger };
            }
            step.consumed = true;
            return;
        }
        let next = trigger[held.chars().count()];
        if keys::char_of(keyval) == Some(next) {
            held.push(next);
            if held.chars().count() == trigger.len() {
                self.open_search(held);
            } else {
                self.state = State::Prefix { held, trigger };
            }
            step.consumed = true;
            return;
        }
        self.commit(&held, step);
        self.idle_key(keyval, state, step);
    }

    #[allow(clippy::too_many_arguments)]
    fn search_key(
        &mut self,
        trigger: String,
        mut query: String,
        matches: Vec<Match>,
        cursor: usize,
        keyval: u32,
        state: u32,
        step: &mut Step,
    ) {
        let plain = format!("{trigger}{query}");
        let has_list = !matches.is_empty();
        if keys::has_pass_through_modifier(state) {
            self.commit(&plain, step);
            return self.pass_other(step);
        }
        if keys::is_dead_key(keyval) {
            self.commit(&plain, step);
            return self.idle_key(keyval, state, step);
        }
        step.consumed = true;
        match keyval {
            keys::BACKSPACE => {
                if query.pop().is_some() {
                    let matches = self.search(&query);
                    self.state = State::Search {
                        trigger,
                        query,
                        matches,
                        cursor: 0,
                    };
                }
                return;
            }
            keys::UP | keys::DOWN | keys::PAGE_UP | keys::PAGE_DOWN if has_list => {
                let delta = match keyval {
                    keys::UP => -1,
                    keys::DOWN => 1,
                    keys::PAGE_UP => -(PAGE_SIZE as isize),
                    _ => PAGE_SIZE as isize,
                };
                let last = matches.len() as isize - 1;
                let cursor = (cursor as isize + delta).clamp(0, last) as usize;
                self.state = State::Search {
                    trigger,
                    query,
                    matches,
                    cursor,
                };
                return;
            }
            k if is_enter_or_tab(k) && has_list => {
                return self.commit_match(&matches[cursor], step);
            }
            keys::ESCAPE => return self.commit(&plain, step),
            _ => {}
        }
        match keys::char_of(keyval) {
            Some(c) if is_query_char(c) => {
                query.push(c);
                let matches = self.search(&query);
                if matches.is_empty() {
                    let plain = format!("{trigger}{query}");
                    self.commit(&plain, step);
                } else {
                    self.state = State::Search {
                        trigger,
                        query,
                        matches,
                        cursor: 0,
                    };
                }
            }
            Some(CLOSING_CHAR) if !query.is_empty() && self.exact(&query).is_some() => {
                if let Some(m) = self.exact(&query) {
                    self.commit_match(&m, step);
                }
            }
            Some(_) => {
                self.commit(&plain, step);
                self.idle_key(keyval, state, step);
            }
            None => {
                self.commit(&plain, step);
                self.pass_other(step);
            }
        }
    }

    fn exact(&self, query: &str) -> Option<Match> {
        self.index.exact_match(query, &self.config.aliases)
    }

    fn dead_key(&mut self, accent: Accent, keyval: u32, state: u32, step: &mut Step) {
        let spacing = accent.spacing.to_string();
        if keys::has_pass_through_modifier(state) {
            self.commit(&spacing, step);
            return self.pass_other(step);
        }
        if keyval == keys::BACKSPACE {
            step.consumed = true;
            return;
        }
        if keyval == accent.keyval {
            step.consumed = true;
            return self.commit(&spacing, step);
        }
        if let Some(next) = keys::accent(keyval) {
            self.commit(&spacing, step);
            return self.hold_dead(next, step);
        }
        match keys::char_of(keyval) {
            Some(' ') => {
                step.consumed = true;
                self.commit(&spacing, step);
            }
            Some(c) => match keys::compose(c, accent) {
                Some(composed) => {
                    step.consumed = true;
                    self.commit(&composed.to_string(), step);
                }
                None => {
                    self.commit(&spacing, step);
                    self.idle_key(keyval, state, step);
                }
            },
            None => {
                self.commit(&spacing, step);
                self.pass_other(step);
            }
        }
    }

    /// What the preedit and the candidate list should show in this state.
    fn wanted(&self) -> (Option<String>, Option<(Vec<String>, usize)>) {
        match &self.state {
            State::Idle => (None, None),
            State::Prefix { held, .. } => (Some(held.clone()), None),
            State::Dead(accent) => (Some(accent.spacing.to_string()), None),
            State::Search {
                trigger,
                query,
                matches,
                cursor,
            } => {
                let preedit = format!("{trigger}{query}");
                let list = (!matches.is_empty()).then(|| {
                    let items = matches
                        .iter()
                        .map(|m| format!("{} {}", self.shown(m), m.shortcode))
                        .collect();
                    (items, *cursor)
                });
                (Some(preedit), list)
            }
        }
    }

    /// Turn a step into actions. A commit first clears the preedit, so that
    /// the daemon holds no COMMIT preedit it could commit a second time.
    fn finish(&mut self, step: Step) -> Outcome {
        let mut actions = Vec::new();
        if !step.commit.is_empty() {
            if self.shown_preedit.take().is_some() {
                actions.push(Action::ClearPreedit);
            }
            if self.shown_candidates.take().is_some() {
                actions.push(Action::HideCandidates);
            }
            actions.push(Action::Commit(step.commit));
        }
        let (preedit, list) = self.wanted();
        if preedit != self.shown_preedit {
            actions.push(match &preedit {
                Some(text) => Action::ShowPreedit(text.clone()),
                None => Action::ClearPreedit,
            });
            self.shown_preedit = preedit;
        }
        if list != self.shown_candidates {
            actions.push(match &list {
                Some((items, cursor)) => Action::ShowCandidates {
                    items: items.clone(),
                    cursor: *cursor,
                },
                None => Action::HideCandidates,
            });
            self.shown_candidates = list;
        }
        Outcome {
            actions,
            consumed: step.consumed,
            used: step.used,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::*;

    /// The app's text: what was committed, and each passed character.
    #[derive(Default)]
    struct App {
        text: String,
        passed: Vec<u32>,
    }

    struct Rig {
        picker: Picker,
        app: App,
        last: Outcome,
        state: u32,
        used: Vec<String>,
    }

    impl Rig {
        fn new() -> Rig {
            Rig::with_trigger("::")
        }

        fn with_trigger(trigger: &str) -> Rig {
            Rig {
                picker: Picker::new(
                    Config {
                        trigger: trigger.to_string(),
                        ..Config::default()
                    },
                    EmojiIndex::gemoji(),
                ),
                app: App::default(),
                last: Outcome::default(),
                state: 0,
                used: Vec::new(),
            }
        }

        fn apply(&mut self, out: Outcome, keyval: Option<u32>) {
            if let Some(u) = &out.used {
                self.used.push(u.clone());
            }
            for a in &out.actions {
                if let Action::Commit(t) = a {
                    self.app.text.push_str(t);
                }
            }
            if let Some(k) = keyval
                && !out.consumed
            {
                self.app.passed.push(k);
                if let Some(c) = char_of(k).filter(|_| !has_pass_through_modifier(self.state)) {
                    self.app.text.push(c);
                }
            }
            self.last = out;
        }

        fn press(&mut self, keyval: u32, state: u32) -> &mut Rig {
            let out = self.picker.key(keyval, state);
            self.state = state;
            self.apply(out, Some(keyval));
            self.state = 0;
            self
        }

        fn key(&mut self, keyval: u32) -> &mut Rig {
            self.press(keyval, 0)
        }

        fn typed(&mut self, s: &str) -> &mut Rig {
            for c in s.chars() {
                let k = if (c as u32) < 0x100 {
                    c as u32
                } else {
                    0x0100_0000 + c as u32
                };
                self.key(k);
            }
            self
        }

        fn preedit(&self) -> Option<String> {
            self.picker.shown_preedit.clone()
        }

        fn list(&self) -> Option<(Vec<String>, usize)> {
            self.picker.shown_candidates.clone()
        }
    }

    // Idle

    #[test]
    fn idle_release_and_modifier_pass() {
        let mut r = Rig::new();
        assert!(!r.picker.key(0x61, RELEASE_MASK).consumed);
        assert!(!r.picker.key(SHIFT_L, 0).consumed);
    }

    #[test]
    fn idle_ctrl_passes() {
        let mut r = Rig::new();
        r.press(0x63, CONTROL_MASK);
        assert_eq!(r.app.passed, [0x63]);
    }

    #[test]
    fn idle_known_dead_key_is_held() {
        let mut r = Rig::new();
        r.key(DEAD_ACUTE);
        assert!(r.last.consumed);
        assert_eq!(r.preedit().as_deref(), Some("´"));
    }

    #[test]
    fn idle_unknown_dead_key_passes() {
        let mut r = Rig::new();
        r.key(DEAD_BELOWDOT);
        assert!(!r.last.consumed);
        assert_eq!(r.preedit(), None);
    }

    #[test]
    fn idle_trigger_at_a_boundary_is_held() {
        let mut r = Rig::new();
        r.typed(":");
        assert!(r.last.consumed);
        assert_eq!(r.preedit().as_deref(), Some(":"));
    }

    #[test]
    fn idle_trigger_after_a_word_passes() {
        let mut r = Rig::new();
        r.picker.surrounding_text("std", 3, 3);
        r.typed("::");
        assert_eq!(r.app.text, "::");
        assert_eq!(r.preedit(), None);
    }

    #[test]
    fn idle_other_characters_and_keys_pass() {
        let mut r = Rig::new();
        r.typed("a").key(LEFT).key(BACKSPACE);
        assert_eq!(r.app.passed, [0x61, LEFT, BACKSPACE]);
    }

    // Prefix

    #[test]
    fn prefix_completes_into_a_search() {
        let mut r = Rig::new();
        r.typed("::");
        assert_eq!(r.preedit().as_deref(), Some("::"));
        assert_eq!(r.list(), None);
        assert_eq!(r.app.text, "");
    }

    #[test]
    fn prefix_keeps_held_text_over_a_modifier() {
        let mut r = Rig::new();
        r.typed(":")
            .key(SHIFT_L)
            .press(SHIFT_L, RELEASE_MASK)
            .typed(":");
        assert_eq!(r.preedit().as_deref(), Some("::"));
    }

    #[test]
    fn prefix_ctrl_commits_held_then_passes() {
        let mut r = Rig::new();
        r.typed(":").press(0x76, CONTROL_MASK);
        assert_eq!(
            r.last.actions,
            [Action::ClearPreedit, Action::Commit(":".into())]
        );
        assert!(!r.last.consumed);
    }

    #[test]
    fn prefix_character_commits_held_with_it() {
        let mut r = Rig::new();
        r.typed(":a");
        assert_eq!(
            r.last.actions,
            [Action::ClearPreedit, Action::Commit(":a".into())]
        );
        assert!(r.last.consumed);
        assert_eq!(r.app.text, ":a");
    }

    #[test]
    fn prefix_backspace_drops_held() {
        let mut r = Rig::new();
        r.typed(":").key(BACKSPACE);
        assert!(r.last.consumed);
        assert_eq!(r.last.actions, [Action::ClearPreedit]);
        assert_eq!(r.app.text, "");
    }

    #[test]
    fn prefix_dead_key_commits_held_and_holds_the_accent() {
        let mut r = Rig::new();
        r.typed(":").key(DEAD_ACUTE);
        assert_eq!(r.app.text, ":");
        assert_eq!(r.preedit().as_deref(), Some("´"));
    }

    #[test]
    fn prefix_key_without_character_commits_held_then_passes() {
        let mut r = Rig::new();
        r.typed(":").key(LEFT);
        assert_eq!(r.app.passed, [LEFT]);
        assert_eq!(r.app.text, ":");
    }

    #[test]
    fn prefix_escape_commits_held_then_passes() {
        let mut r = Rig::new();
        r.typed(":").key(ESCAPE);
        assert!(!r.last.consumed);
        assert_eq!(r.app.text, ":");
    }

    // Search

    #[test]
    fn search_shows_trigger_and_query_with_the_list() {
        let mut r = Rig::new();
        r.typed("::fi");
        assert_eq!(r.preedit().as_deref(), Some("::fi"));
        let (items, cursor) = r.list().unwrap();
        assert_eq!(items.len(), 56);
        assert_eq!(cursor, 0);
    }

    #[test]
    fn search_candidate_text_is_emoji_and_shortcode() {
        let mut r = Rig::new();
        r.typed("::fire");
        assert_eq!(r.list().unwrap().0[0], "🔥 fire");
    }

    #[test]
    fn search_ctrl_commits_plain_and_passes() {
        let mut r = Rig::new();
        r.typed("::fi").press(0x61, CONTROL_MASK);
        assert_eq!(
            r.last.actions,
            [
                Action::ClearPreedit,
                Action::HideCandidates,
                Action::Commit("::fi".into())
            ]
        );
        assert!(!r.last.consumed);
    }

    #[test]
    fn search_dead_key_commits_plain_and_holds_the_accent() {
        let mut r = Rig::new();
        r.typed("::fi").key(DEAD_ACUTE);
        assert_eq!(r.app.text, "::fi");
        assert_eq!(r.preedit().as_deref(), Some("´"));
        assert_eq!(r.list(), None);
    }

    #[test]
    fn search_unknown_dead_key_commits_plain_and_passes() {
        let mut r = Rig::new();
        r.typed("::fi").key(DEAD_BELOWDOT);
        assert_eq!(r.app.text, "::fi");
        assert!(!r.last.consumed);
    }

    #[test]
    fn search_query_with_no_match_commits_plain() {
        let mut r = Rig::new();
        r.typed("::fireq");
        assert_eq!(r.app.text, "::fireq");
        assert!(r.last.consumed);
        assert_eq!(r.preedit(), None);
    }

    #[test]
    fn search_colon_after_exact_shortcode_commits_emoji() {
        let mut r = Rig::new();
        r.typed("::fire:");
        assert_eq!(r.app.text, "🔥");
    }

    #[test]
    fn search_colon_after_partial_shortcode_commits_plain() {
        let mut r = Rig::new();
        r.typed("::fir:");
        assert_eq!(r.app.text, "::fir:");
    }

    #[test]
    fn search_other_character_commits_plain_with_it() {
        let mut r = Rig::new();
        r.typed("::fi ");
        assert_eq!(r.last.actions.last(), Some(&Action::Commit("::fi ".into())));
        r.typed("::fi").key(0xe6);
        assert_eq!(r.app.text, "::fi ::fiæ");
    }

    #[test]
    fn search_altgr_character_closes_as_a_character() {
        let mut r = Rig::new();
        r.typed("::fi").press(0x7b, MOD5_MASK);
        assert_eq!(r.app.text, "::fi{");
        assert!(r.last.consumed);
    }

    #[test]
    fn search_backspace_edits_the_query() {
        let mut r = Rig::new();
        r.typed("::fir").key(BACKSPACE);
        assert_eq!(r.preedit().as_deref(), Some("::fi"));
        r.key(BACKSPACE).key(BACKSPACE);
        assert_eq!(r.preedit().as_deref(), Some("::"));
        assert_eq!(r.list(), None);
        r.key(BACKSPACE);
        assert_eq!(r.preedit(), None);
        assert_eq!(r.app.text, "");
        assert!(r.last.consumed);
    }

    #[test]
    fn search_arrows_move_within_the_list() {
        let mut r = Rig::new();
        r.typed("::fire").key(UP);
        assert_eq!(r.list().unwrap().1, 0);
        assert!(r.last.consumed);
        r.key(DOWN);
        assert_eq!(r.list().unwrap().1, 1);
        r.key(RETURN);
        assert_eq!(r.app.text, "🧑\u{200d}🚒");
    }

    #[test]
    fn search_page_keys_move_by_a_page() {
        let mut r = Rig::new();
        r.typed("::fi").key(PAGE_DOWN);
        assert_eq!(r.list().unwrap().1, 9);
        r.key(PAGE_UP);
        assert_eq!(r.list().unwrap().1, 0);
        for _ in 0..10 {
            r.key(PAGE_DOWN);
        }
        assert_eq!(r.list().unwrap().1, 55);
    }

    #[test]
    fn search_arrows_without_a_list_commit_plain_and_pass() {
        let mut r = Rig::new();
        r.typed("::").key(DOWN);
        assert_eq!(r.app.text, "::");
        assert_eq!(r.app.passed, [DOWN]);
    }

    #[test]
    fn search_enter_and_tab_commit_the_selection() {
        let mut r = Rig::new();
        r.typed("::fire").key(RETURN);
        assert_eq!(r.app.text, "🔥");
        assert!(r.last.consumed);
        r.typed(" ::fire").key(TAB);
        assert_eq!(r.app.text, "🔥 🔥");
    }

    #[test]
    fn search_enter_without_a_list_commits_plain_and_passes() {
        let mut r = Rig::new();
        r.typed("::").key(RETURN);
        assert_eq!(r.app.text, "::");
        assert_eq!(r.app.passed, [RETURN]);
    }

    #[test]
    fn search_escape_commits_plain_and_is_consumed() {
        let mut r = Rig::new();
        r.typed("::fi").key(ESCAPE);
        assert_eq!(r.app.text, "::fi");
        assert!(r.last.consumed);
        r.typed(" ::").key(ESCAPE);
        assert_eq!(r.app.text, "::fi ::");
    }

    #[test]
    fn search_other_key_commits_plain_and_passes() {
        let mut r = Rig::new();
        r.typed("::fi").key(LEFT);
        assert_eq!(r.app.text, "::fi");
        assert_eq!(r.app.passed, [LEFT]);
    }

    #[test]
    fn search_click_commits_on_the_shown_page() {
        let mut r = Rig::new();
        r.typed("::fire");
        let out = r.picker.candidate_clicked(2);
        r.apply(out, None);
        assert_eq!(r.app.text, "🚒");
        r.typed(" ::fi").key(PAGE_DOWN);
        let tenth = EmojiIndex::gemoji().search("fi")[9].emoji.clone();
        let out = r.picker.candidate_clicked(0);
        r.apply(out, None);
        assert!(r.app.text.ends_with(&tenth));
    }

    #[test]
    fn search_release_keeps_the_search() {
        let mut r = Rig::new();
        r.typed("::fi");
        assert!(!r.picker.key(0x69, RELEASE_MASK).consumed);
        assert_eq!(r.preedit().as_deref(), Some("::fi"));
    }

    // Dead

    #[test]
    fn dead_composes_with_a_character() {
        let mut r = Rig::new();
        r.key(DEAD_ACUTE).typed("e");
        r.key(DEAD_DIAERESIS).typed("u");
        r.key(DEAD_TILDE).typed("n");
        r.key(DEAD_ACUTE).press(0x45, SHIFT_MASK);
        assert_eq!(r.app.text, "éüñÉ");
    }

    #[test]
    fn dead_with_space_or_itself_gives_the_accent() {
        let mut r = Rig::new();
        r.key(DEAD_ACUTE).typed(" ");
        r.key(DEAD_CIRCUMFLEX).key(DEAD_CIRCUMFLEX);
        assert_eq!(r.app.text, "´^");
        assert!(r.last.consumed);
    }

    #[test]
    fn dead_without_a_precomposed_form_gives_accent_and_character() {
        let mut r = Rig::new();
        r.key(DEAD_ACUTE).typed("q");
        assert_eq!(r.app.text, "´q");
        assert!(r.last.consumed);
    }

    #[test]
    fn dead_then_another_dead_key() {
        let mut r = Rig::new();
        r.key(DEAD_ACUTE).key(DEAD_DIAERESIS);
        assert_eq!(r.app.text, "´");
        assert_eq!(r.preedit().as_deref(), Some("¨"));
    }

    #[test]
    fn dead_then_unknown_dead_key_commits_and_passes() {
        let mut r = Rig::new();
        r.key(DEAD_ACUTE).key(DEAD_BELOWDOT);
        assert_eq!(r.app.text, "´");
        assert!(!r.last.consumed);
    }

    #[test]
    fn dead_backspace_drops_the_accent() {
        let mut r = Rig::new();
        r.key(DEAD_ACUTE).key(BACKSPACE);
        assert_eq!(r.app.text, "");
        assert_eq!(r.preedit(), None);
        assert!(r.last.consumed);
    }

    #[test]
    fn dead_keeps_the_accent_over_shift() {
        let mut r = Rig::new();
        r.key(DEAD_ACUTE).key(SHIFT_L).press(0x45, SHIFT_MASK);
        assert_eq!(r.app.text, "É");
    }

    #[test]
    fn dead_then_key_without_character_commits_and_passes() {
        let mut r = Rig::new();
        r.key(DEAD_ACUTE).key(LEFT);
        assert_eq!(r.app.text, "´");
        assert_eq!(r.app.passed, [LEFT]);
    }

    #[test]
    fn dead_then_ctrl_commits_and_passes() {
        let mut r = Rig::new();
        r.key(DEAD_ACUTE).press(0x61, CONTROL_MASK);
        assert_eq!(r.app.text, "´");
        assert!(!r.last.consumed);
    }

    // Boundary and focus

    #[test]
    fn two_emoji_in_a_row() {
        let mut r = Rig::new();
        r.typed("::fire:::rocket:");
        assert_eq!(r.app.text, "🔥🚀");
    }

    #[test]
    fn a_trigger_after_a_typed_letter_passes() {
        let mut r = Rig::new();
        r.typed("d::");
        assert_eq!(r.app.text, "d::");
    }

    #[test]
    fn a_trigger_after_a_typed_space_opens() {
        let mut r = Rig::new();
        r.typed("d ::");
        assert_eq!(r.preedit().as_deref(), Some("::"));
    }

    #[test]
    fn dropping_state_commits_nothing() {
        let mut r = Rig::new();
        r.typed("::fi");
        let out = r.picker.drop_state();
        assert_eq!(out.actions, [Action::ClearPreedit, Action::HideCandidates]);
        r.typed("a");
        assert_eq!(r.app.passed, [0x61]);
    }

    #[test]
    fn focus_in_forgets_the_last_character() {
        let mut r = Rig::new();
        r.typed("d");
        r.picker.focus_in();
        r.typed("::");
        assert_eq!(r.preedit().as_deref(), Some("::"));
    }

    // Other triggers

    #[test]
    fn a_one_character_trigger_opens_at_once() {
        let mut r = Rig::with_trigger(";");
        r.typed(";fire");
        assert_eq!(r.preedit().as_deref(), Some(";fire"));
        r.key(RETURN);
        assert_eq!(r.app.text, "🔥");
    }

    #[test]
    fn a_three_character_trigger_holds_two() {
        let mut r = Rig::with_trigger(";;;");
        r.typed(";;");
        assert_eq!(r.preedit().as_deref(), Some(";;"));
        r.typed(";fi");
        assert_eq!(r.preedit().as_deref(), Some(";;;fi"));
        r.key(ESCAPE);
        assert_eq!(r.app.text, ";;;fi");
    }

    #[test]
    fn a_three_character_trigger_broken_off_commits_held() {
        let mut r = Rig::with_trigger(";;;");
        r.typed(";;x");
        assert_eq!(r.app.text, ";;x");
    }

    fn config(trigger: &str, aliases: &[(&str, &str)]) -> Config {
        Config {
            trigger: trigger.to_string(),
            aliases: aliases
                .iter()
                .map(|(n, t)| crate::config::Alias {
                    name: n.to_string(),
                    text: t.to_string(),
                })
                .collect(),
            ..Config::default()
        }
    }

    impl Rig {
        fn with_config(c: Config) -> Rig {
            let mut r = Rig::new();
            r.picker = Picker::new(c, EmojiIndex::gemoji());
            r
        }

        fn surrounding(&mut self, text: &str) -> &mut Rig {
            let n = text.chars().count() as u32;
            self.picker.surrounding_text(text, n, n);
            self
        }
    }

    #[test]
    fn other_triggers_count_only_at_a_boundary() {
        for t in [":", ";;", ":;:"] {
            let mut r = Rig::with_trigger(t);
            r.surrounding("std");
            r.typed(t);
            assert_eq!(r.app.text, t, "{t}");
            assert_eq!(r.preedit(), None, "{t}");
        }
    }

    #[test]
    fn other_triggers_open_a_search_and_close_with_a_colon() {
        for t in [":", ";;", ":;:"] {
            let mut r = Rig::with_trigger(t);
            r.typed(t);
            assert_eq!(r.preedit().as_deref(), Some(t), "{t}");
            r.typed("fire:");
            assert_eq!(r.app.text, "🔥", "{t}");
        }
    }

    #[test]
    fn other_triggers_hold_a_prefix_and_backspace_drops_it() {
        let mut r = Rig::with_trigger(";;");
        r.typed(";");
        assert_eq!(r.preedit().as_deref(), Some(";"));
        assert_eq!(r.app.text, "");
        r.key(BACKSPACE);
        assert!(r.last.consumed);
        assert_eq!(r.preedit(), None);
        assert_eq!(r.app.text, "");
        assert!(r.app.passed.is_empty());
    }

    #[test]
    fn other_triggers_escape_commits_the_plain_text() {
        let mut r = Rig::with_trigger(":;:");
        r.typed(":;:fi");
        r.key(ESCAPE);
        assert_eq!(r.app.text, ":;:fi");
    }

    #[test]
    fn two_colons_pass_with_another_trigger() {
        let mut r = Rig::with_trigger(";;");
        r.typed("::fire");
        assert_eq!(r.app.text, "::fire");
        assert_eq!(r.preedit(), None);
    }

    #[test]
    fn a_colon_commits_an_exact_alias() {
        let mut r = Rig::with_config(config("::", &[("lgtm", "👍")]));
        r.typed("::LGTM:");
        assert_eq!(r.app.text, "👍");
    }

    #[test]
    fn enter_commits_the_whole_text_of_an_alias() {
        let shrug = "¯\\_(ツ)_/¯";
        let mut r = Rig::with_config(config("::", &[("shrug", shrug)]));
        r.typed("::shrug");
        assert_eq!(r.list().unwrap().0[0], format!("{shrug} shrug"));
        r.key(RETURN);
        assert_eq!(r.app.text, shrug);
    }

    #[test]
    fn an_alias_named_like_a_shortcode_wins() {
        let mut r = Rig::with_config(config("::", &[("fire", "🚒")]));
        r.typed("::fire:");
        assert_eq!(r.app.text, "🚒");
    }

    #[test]
    fn a_new_config_keeps_the_held_prefix() {
        let mut r = Rig::with_trigger(":;:");
        r.typed(":;");
        r.picker.set_config(config(";", &[]));
        assert_eq!(r.preedit().as_deref(), Some(":;"));
        r.typed(":fire:");
        assert_eq!(r.app.text, "🔥");
    }

    #[test]
    fn a_new_config_keeps_the_typed_trigger_of_a_search() {
        let mut r = Rig::with_trigger("::");
        r.typed("::fi");
        r.picker.set_config(config(";;;", &[]));
        assert_eq!(r.preedit().as_deref(), Some("::fi"));
        r.key(ESCAPE);
        assert_eq!(r.app.text, "::fi");
        r.typed(" ;;;fire:");
        assert_eq!(r.app.text, "::fi 🔥");
    }

    #[test]
    fn a_shorter_trigger_during_a_prefix() {
        let mut r = Rig::with_trigger(";;;");
        r.typed(";;");
        r.picker.set_config(config(":", &[]));
        r.typed(";");
        assert_eq!(r.preedit().as_deref(), Some(";;;"));
    }

    #[test]
    fn a_new_config_searches_an_open_query_again() {
        let mut r = Rig::with_trigger("::");
        r.typed("::lg");
        assert!(r.list().is_some());
        let out = r.picker.set_config(config("::", &[("lgtm", "👍")]));
        assert!(
            out.actions
                .iter()
                .any(|a| matches!(a, Action::ShowCandidates { .. }))
        );
        assert_eq!(r.list().unwrap().0[0], "👍 lgtm");
    }

    #[test]
    fn search_opened_marks_the_key_that_completes_the_trigger() {
        let mut r = Rig::with_trigger("::");
        r.typed(":");
        assert!(!r.picker.search_opened());
        r.typed(":");
        assert!(r.picker.search_opened());
        r.typed("f");
        assert!(!r.picker.search_opened());
        let mut one = Rig::with_trigger(":");
        one.typed(":");
        assert!(one.picker.search_opened());
    }

    // Usage and the skin tone

    fn toned(tone: crate::tone::SkinTone, aliases: &[(&str, &str)]) -> Rig {
        Rig::with_config(Config {
            skin_tone: tone,
            ..config("::", aliases)
        })
    }

    fn first(r: &Rig) -> String {
        r.list().expect("a candidate list").0[0].clone()
    }

    #[test]
    fn each_commit_path_notes_the_used_text() {
        let mut r = Rig::new();
        r.typed("::fire").key(RETURN);
        r.typed(" ::fire").key(TAB);
        r.typed(" ::fire:");
        r.typed(" ::fire");
        let out = r.picker.candidate_clicked(0);
        r.apply(out, None);
        assert_eq!(r.app.text, "🔥 🔥 🔥 🔥");
        assert_eq!(r.used, ["🔥", "🔥", "🔥", "🔥"]);
    }

    #[test]
    fn plain_text_notes_nothing() {
        let mut r = Rig::new();
        r.typed("::fi").key(ESCAPE);
        r.typed(" ::fireq");
        r.typed(" ::fi").press(0x61, CONTROL_MASK);
        r.typed(" ::fir:");
        r.typed(" ::fi ");
        assert!(r.used.is_empty(), "{:?}", r.used);
    }

    #[test]
    fn an_alias_commit_notes_its_text() {
        let mut r = Rig::with_config(config("::", &[("lgtm", "👍")]));
        r.typed("::lgtm:");
        assert_eq!(r.used, ["👍"]);
    }

    #[test]
    fn the_tone_shows_and_commits_but_the_bare_emoji_is_used() {
        let mut r = toned(crate::tone::SkinTone::Medium, &[]);
        r.typed("::wave");
        assert_eq!(first(&r), "👋🏽 wave");
        r.key(RETURN);
        r.typed(" ::+1:");
        r.typed(" ::fire:");
        assert_eq!(r.app.text, "👋🏽 👍🏽 🔥");
        assert_eq!(r.used, ["👋", "👍", "🔥"]);
    }

    #[test]
    fn an_alias_never_takes_the_tone() {
        let mut r = toned(
            crate::tone::SkinTone::Dark,
            &[("hi", "👋"), ("wave_hi", "👋")],
        );
        r.typed("::hi:");
        assert_eq!(r.app.text, "👋");
        r.typed(" ::wave");
        assert_eq!(first(&r), "👋🏿 wave");
        assert!(!r.list().unwrap().0.contains(&"👋 wave_hi".to_string()));
        r.key(ESCAPE);
        r.typed(" ::wave_h");
        assert_eq!(first(&r), "👋 wave_hi");
    }

    #[test]
    fn set_usage_re_sorts_an_open_search() {
        let mut r = Rig::new();
        r.typed("::thumbs");
        assert_eq!(first(&r), "👍 thumbsup");
        let mut usage = Usage::new();
        usage.record("👎", 1_000);
        let out = r.picker.set_usage(&usage);
        r.apply(out, None);
        assert_eq!(first(&r), "👎 thumbsdown");
        assert!(matches!(
            r.last.actions.last(),
            Some(Action::ShowCandidates { .. })
        ));
        let out = r.picker.set_usage(&Usage::new());
        r.apply(out, None);
        assert_eq!(first(&r), "👍 thumbsup");
    }
}
