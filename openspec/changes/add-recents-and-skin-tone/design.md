# Design

## Context

See proposal.md for why. The code this change builds on:

- `EmojiIndex::search_with(query, aliases)` ranks hits as 0 exact, 1 prefix, 2 word prefix, 3 substring. It sorts by `(rank, order)`, where the aliases take the orders before gemoji's. It keeps one hit per text. `exact_with` serves the closing colon.
- `Picker` holds a `Config` and emits `Action::Commit(text)` in an `Outcome`. It does not say whether a commit was an emoji or plain text.
- `ConfigWatch` in the engine re-reads the config file by fingerprint (mtime in ns, size, device, inode) on focus-in and when a search opens.
- `config::write_atomically` writes a temp file next to the target, calls `sync_all`, and renames it.
- The e2e `privateEnv` sets `XDG_CONFIG_HOME`, `XDG_CACHE_HOME` and `XDG_DATA_HOME` into the run folder. It sets no `XDG_STATE_HOME` and keeps the real `HOME`. So an engine that writes usage would write the owner's `~/.local/state/torpedo/usage.toml` from every existing e2e test. The harness must change before the engine records anything.
- The picker fixture opens one session per test file and a new input context per test. Commits in one test would reorder the candidates of the next.

Facts checked for this change:

- gemoji's `emoji.json` has 1870 entries, and 305 of them have `"skin_tones": true`. Torpedo does not read that field yet.
- `IBus.EmojiData.load('/usr/share/ibus/dicts/emoji-en.dict')` (ibus 1.5.34) gives 5594 sequences, the toned ones among them.
- Unicode 16.0 `emoji-data.txt` lists 134 Emoji_Modifier_Base code points in 50 ranges.
- A Python prototype of the rule "modifier after each Emoji_Modifier_Base, drop a VS16 right after one" put 304 of the 305 entries in the dictionary in all five tones. The one miss is `people_holding_hands`, 🧑‍🤝‍🧑. The handshake 🤝 (U+1F91D) is a modifier base, but in this sequence only the two people take the tone: 🧑🏻‍🤝‍🧑🏻. With the added rule "a modifier base between two ZWJs stays bare", all 305 × 5 forms are in the dictionary.
- 👪 (U+1F46A, family) is a modifier base, but no family sequence takes a tone, and gemoji marks no family with `skin_tones`. Torpedo only tones entries with `skin_tones`, so the family entries stay bare.
- Emoji such as 👯 dancers and 🤼 wrestling hold a modifier base but have no `skin_tones` in gemoji. They stay bare too.

## Goals / Non-Goals

**Goals:**

- Ranking by usage that is pure logic in `torpedo-core`, with no clock inside the search.
- A usage file that can never stop typing and never grows past 200 entries.
- A skin tone rule with one exception, checked against the official sequences on this machine.
- No e2e test can write the owner's usage file.

**Non-Goals:**

- Mixed tones in multi-person emoji, such as 🧑🏻‍🤝‍🧑🏿. One tone applies to every person.
- A per-emoji tone choice, or a tone picker in the candidate list.
- A "recent" list for an empty query. The candidate list still needs one query character.
- Usage for plain text.

## Decisions

### The usage score

Each entry holds a `score` (a float) and `last` (Unix seconds of its last commit). The score at time `now` is:

```
score(now) = score × 2^(−(now − last) / H),  H = 14 days = 1 209 600 s
```

An age below 0, from a clock that went back, counts as 0. A commit sets `score = score(now) + 1` and `last = now`. A new entry starts at `score = 1`. So each commit counts 1 and loses half its weight every 14 days, which is "use count weighted by recency" in one number per entry.

Alternative considered: `count` and `last`, with a weight such as `count / (1 + days)`. It forgets nothing. An emoji used 300 times a year ago would beat one used 5 times this week. The decayed sum forgets old use by itself and needs no list of past commits.

### Ranking needs no clock

All scores fade by the same factor as time passes. So the order of two entries does not depend on `now`: `a(now) > b(now)` exactly when `log2(a.score) + a.last / H > log2(b.score) + b.last / H`. The picker sorts by that key, which it computes once per usage file. The search takes no time parameter, and a test needs no fake clock. The engine reads the clock only when it records a commit.

The sort key of a hit becomes `(rank, unused, −key, order)`, where `unused` is 0 for a text with usage and 1 otherwise. The rank comes first, so usage never lifts a hit into a better rank. Dedup by text still keeps the first hit after the sort, as it does now.

### The usage module in torpedo-core

`torpedo_core::usage` holds:

- `Usage`: the entries, keyed by text.
- `usage_path(env)`: `$XDG_STATE_HOME/torpedo/usage.toml` when that is an absolute path, else `$HOME/.local/state/torpedo/usage.toml`. Like `config_path`, it takes the environment as a parameter.
- `parse(text) -> Result<Usage, Vec<Problem>>` and `to_text()`.
- `record(text, now)`: adds the commit and drops the entries with the lowest `score(now)` until at most 200 remain. A tie drops the older `last`.
- `ranking() -> HashMap<String, f64>`: the sort key of each text.
- `save(path)`: through `config::write_atomically`, which already creates the folder, writes a temp file and renames it.

The file format:

```toml
# Torpedo writes this file after each commit. It ranks the emoji you use.
[[used]]
text = "🔥"
score = 1.5
last = 1763560000
```

`to_text` writes the entries by score, highest first, with `toml_edit`, so any text is quoted correctly. `parse` refuses the whole file when it does not parse, or when an entry is not a table, has no `text` or an empty one, has a `text` with a control character, has a `score` that is not a finite number of 0 or more, has a `last` that is not an integer of 0 or more, or repeats a `text`. Keys it does not know are skipped. A file with more than 200 entries is read whole and cut to 200 at the next write.

### The engine records usage

- `Outcome` gains `used: Option<String>`: the untoned text of the emoji or alias that the step committed, or `None`. The picker sets it on Enter, Tab, a click and the closing colon. Plain text commits leave it `None`.
- The engine keeps a `UsageWatch`, built like `ConfigWatch`. Both share the fingerprint code, which moves to its own module. `check()` runs where the config check runs: at start, on FocusIn and FocusInId, and after a key that opened a search. A new fingerprint reads the file. A missing or bad file gives an empty `Usage`, and a bad file is logged once under `TORPEDO_DEBUG`.
- After an outcome with `used`, and only when `remember_usage` is true, the engine calls `record(text, SystemTime::now())`, saves, and stores the fingerprint of the file it wrote. So its own write does not count as a change, and a delete by the settings window does. A failed save is logged and typing goes on with the usage in memory.
- The engine passes the usage to the picker with `Picker::set_usage(&Usage)` after each read and each record. With `remember_usage = false` the engine passes an empty usage, so the search ignores the file. It still keeps the file untouched.
- A broken usage file reads as empty, so the next commit writes a file with that one entry. The broken text is lost. The usage file is state that torpedo owns, so this is fine.

### The skin tone

- `torpedo_core::tone` holds `SkinTone` (`None`, `Light`, `MediumLight`, `Medium`, `MediumDark`, `Dark`), its config strings, its modifier (U+1F3FB to U+1F3FF), and `apply(emoji, tone) -> String`.
- The Emoji_Modifier_Base table is a const array of the 50 ranges of Unicode 16.0 `emoji-data.txt`, with the source URL and date in a comment. It is checked against the dictionary by the test below.
- `apply` walks the code points. After a modifier base it inserts the modifier, unless the base sits between two U+200D. A U+FE0F right after a base that got a modifier is dropped. All else is copied.
- `Emoji` reads `skin_tones` from gemoji, default false. `Match` gains `toned: bool`, true when the hit came from a shortcode of an entry with `skin_tones`. An alias hit is never toned. `exact_with` returns the same flag.
- The picker keeps `Match.emoji` untoned. It applies the tone when it builds a candidate label and when it commits. `used` gets the untoned text. Dedup by text happens before the tone, so an alias `wave_hi = "👋"` and the shortcode `wave` stay one candidate. That candidate takes the form of its best hit: toned under `wave`, bare under `wave_hi`.

### The oracle test

A Rust test in `tone` runs `python3 -I -c <script>`. The script loads the dictionary with `gi` and `IBus.EmojiData.load` and prints each sequence as hex code points, one per line, so no encoding question arises. The test builds `apply` for every gemoji entry with `skin_tones` in each of the five tones, and fails with the list of forms that are not in the dictionary. When `python3`, `gi` or the dictionary is missing, the test fails and names `gir1.2-ibus-1.0` and `ibus-data`. The e2e tests already need both, so a quiet skip would only hide a broken check.

The coverage adapter reads `node --test` files only. So the scenario *Every toned form is a sequence Unicode recommends* gets a `none` entry that names this Rust test. Its five sample forms each have their own scenario with an e2e test.

### The config keys

`Config` gains `skin_tone: SkinTone` and `remember_usage: bool`, with defaults `None` and `true`. `parse` checks both with `check_skin_tone` and a boolean check, and each problem names its key. `ConfigDocument` gains `set_skin_tone` and `set_remember_usage`, and `with_values` takes both, so the window still saves all its fields in one step.

### The settings command and window

- New flags: `--set-skin-tone <tone>`, `--set-remember-usage true|false` and `--clear-usage`. They open no window. `--clear-usage` removes `usage_path(env)`, and a missing file counts as success. It does not read `--config`, because the usage file is not config.
- The window gains an "Emoji" group under the existing two:
  - An `AdwComboRow` "Skin tone" with six items, each a 👋 in that tone and the tone's name, such as "👋🏽 Medium".
  - An `AdwSwitchRow` "Remember used emoji" for `remember_usage`.
  - An `AdwActionRow` "Clear usage" with a destructive button. The button opens an `AdwAlertDialog` with "Cancel" and "Clear", where "Clear" has the destructive appearance. "Clear" removes the usage file and the row's subtitle then says "Usage cleared". `AdwAlertDialog` needs libadwaita 1.5, which the crate's `v1_5` feature already enables.
- The combo row and the switch save through the same path as the other fields: `check_values`, `with_values` and `save`.

### The e2e harness

- `privateEnv` sets `XDG_STATE_HOME` to `<run>/state`. It refuses to start when, after the extras, `XDG_STATE_HOME` is outside the run folder, or is removed while `HOME` is outside it. So the default location is reachable only with a private `HOME`.
- `settings.mjs`: `sandbox()` adds a `state` folder, `settingsEnv` sets `XDG_STATE_HOME` to it, and `ACTIONS` gains the three new flags.
- The session gains `usageFile`, `writeUsage(entries)`, `readUsage()` and `removeUsage()`. `readUsage` parses with Python's `tomllib` through `spawnSync`, so the harness needs no TOML package. `writeUsage` writes the format above with a temp file and a rename, and takes `last` as seconds before now.
- `usePicker` removes the usage file before each test. The new input context takes the focus, so the engine reads the missing file and starts the test with no usage. Test files that share a session without the fixture, and check an order after a commit, do the same.

### Coverage

New e2e files: `e2e/usage/*.test.mjs`, `e2e/skin-tone/*.test.mjs`, and new tests in `e2e/inline-picker/`, `e2e/config-file/` and `e2e/settings-window/`. Five scenarios get `none`:

- skin-tone › *Every toned form is a sequence Unicode recommends*: the Rust oracle test checks it, and the adapter reads node tests only.
- settings-window › *The skin tone selector*, *The usage switch*, *Clear usage confirmed*, *Clear usage cancelled*: only a person looking at the window sees them, and no test opens a window. The flags `--set-skin-tone`, `--set-remember-usage` and `--clear-usage` show the same saves and the delete through e2e tests.

The changed scenarios *A key from a later version* and *A comment and an unknown key survive* keep their keys and get their tests updated.

## Risks / Trade-offs

- [The usage file is written on every emoji commit] → It is at most 200 short entries, written with one rename. Emoji commits come seconds apart at most.
- [The settings window and the engine can race] → A delete and a write at the same moment leave one of them. The usage file is state, so the worst case is one lost clear or one lost commit.
- [A cleared usage file comes back if the engine commits before its next check] → The engine checks on focus-in and when a search opens, and every commit comes from a search. So a commit always follows a check.
- [The IBus dictionary may lag behind gemoji] → Today gemoji's newest toned emoji are in the dictionary. A gemoji update with new toned emoji that the dictionary lacks makes the oracle test fail and name them, which is the point.
- [`cargo test` now needs Python gi and ibus-data] → The e2e tests need them already, and the owner's machine has them.
- [A clock that jumps forward makes every score fade at once] → The order stays the same, because all scores fade by one factor. Only the stored numbers shrink.

## Migration Plan

1. The owner runs `./install.sh` and `ibus restart`. No config change is needed. Usage starts empty and the skin tone starts at `none`, so torpedo behaves as before until the first commits.
2. To roll back, install the previous version. It ignores `skin_tone` and `remember_usage` as unknown keys, and never reads the usage file.
