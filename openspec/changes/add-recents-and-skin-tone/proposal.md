# Proposal

## Why

Torpedo ranks emoji the same way every time, so the emoji the owner types most can sit low in a long list such as `::fi`. Torpedo also always commits the yellow default for hands and people. Rocket on macOS learns what you use and keeps your skin tone. The `add-settings` change left room for both: a later key `skin_tone` in the config file, and a separate file for recent emoji.

## What Changes

- Torpedo records usage. Each commit of an emoji or an alias text through Enter, Tab, a click or the closing colon adds 1 to that text's usage score. A score halves every 14 days without a commit. Plain text is never recorded.
- Inside each rank of the search (exact, prefix, word prefix, substring), used emoji come first in order of their usage score. The rest keep the existing order: aliases by name, then shortcodes in gemoji's order. The ranks themselves do not change, so an unused exact match still comes before a used prefix match.
- The usage lives in the usage file `$XDG_STATE_HOME/torpedo/usage.toml`, default `~/.local/state/torpedo/usage.toml`. It is state, not config. The engine writes it with a temp file and a rename, once per commit, and keeps at most 200 entries by dropping the lowest scores. A missing or broken usage file means no usage and never stops typing. The engine reads the file again on focus-in and when a search opens, so a cleared file takes effect at once.
- The config key `remember_usage` (boolean, default `true`) turns usage off. When it is `false`, the engine records nothing and the search ignores the usage. Torpedo keeps the usage file.
- The config key `skin_tone` takes `none` (default), `light`, `medium-light`, `medium`, `medium-dark` or `dark`. Any other value makes the config file bad. When set, each emoji that gemoji marks with `skin_tones` is shown and committed with that tone. The modifier goes after each Emoji_Modifier_Base character, and a VS16 right after such a character is removed. A modifier base between two ZWJs stays bare, which only the handshake in 🧑‍🤝‍🧑 needs. Alias texts are committed exactly as written. Usage records the emoji without its tone, so a new tone keeps the ranking.
- The settings window gains an "Emoji" group: a skin tone selector that shows the six choices with a 👋 in each tone, a switch for `remember_usage`, and a "Clear usage" button. The button asks for a confirmation in an `AdwAlertDialog` and then deletes the usage file.
- `torpedo-settings` gains `--set-skin-tone <tone>`, `--set-remember-usage true|false` and `--clear-usage`, which open no window. `--check` checks the two new keys.
- Two existing scenarios used `skin_tone` as an example of an unknown key. They now use another unknown key.
- The e2e harness gives each session and each settings run a private `XDG_STATE_HOME`. The picker fixture deletes the usage file before each test, so commits in one test do not reorder the next.
- `GLOSSARY.md` gains Skin tone, Usage, Usage file and Usage score, and Config file and Settings window are brought up to date.

## Capabilities

### New Capabilities

- `usage`: what the engine records when the user commits, the usage score, the usage file with its location, size limit and atomic write, a missing or broken usage file, the `remember_usage` switch, and how the engine notices a changed or deleted usage file.
- `skin-tone`: how the engine applies the skin tone to emoji in the candidate list and in commits, which emoji take it, the rule for the modifier, and that alias texts never take it.

### Modified Capabilities

- `inline-picker`: *Ranking* gives way, because shortcodes no longer always keep gemoji's order among equal ranks. A new requirement puts used emoji first within a rank.
- `config-file`: new requirements for the values of `skin_tone` and `remember_usage`. *A missing config file means defaults* names their defaults. The scenario *A key from a later version* uses another unknown key, because `skin_tone` is now known.
- `settings-window`: new requirements for the skin tone selector, the `remember_usage` switch and "Clear usage". *The settings command runs without a window* gains the three new flags. *Saving refuses a bad value* gains a bad skin tone. The scenario *A comment and an unknown key survive* uses another unknown key.

## Impact

- `torpedo-core`: new modules for usage (score, limit, parse, write, path) and for the skin tone (the Emoji_Modifier_Base table and the tone rule). The search takes the usage and the time. The picker takes the usage, applies the tone, and reports the untoned text of each commit. `Config` gains `skin_tone` and `remember_usage`. The gemoji data model reads `skin_tones`.
- `torpedo-engine`: a watch on the usage file like the config watch, and a write after each commit.
- `torpedo-settings`: the new flags, the "Emoji" group, and `AdwAlertDialog`, which needs the libadwaita feature `v1_5` that the crate already enables.
- e2e: `e2e/harness/session.mjs`, `e2e/harness/settings.mjs`, `e2e/inline-picker/fixture.mjs`, new test files under `e2e/usage/` and `e2e/skin-tone/`, changed tests in `e2e/config-file/` and `e2e/settings-window/`, and the coverage maps.
- A Rust test in `torpedo-core` checks every toned form against `/usr/share/ibus/dicts/emoji-en.dict` through Python gi. So `cargo test` needs `python3`, `gir1.2-ibus-1.0` and `ibus-data`, which the e2e tests already need.
- Docs: `GLOSSARY.md`, `AGENTS.md` (the new flags and the private state folder), `crates/torpedo-core/data/gemoji/README.md` (the `skin_tones` field).
- No new crate dependency. No change to install or uninstall. Uninstall leaves the usage file, as it leaves the config file.

## Claims checked

`spec-checks claims add-recents-and-skin-tone --all` listed eight claims in requirements the change does not touch, and fourteen of its own. I also read every requirement of `inline-picker` that names a commit, a candidate or the ranking, because usage and the skin tone reach further than the shared glossary terms.

- inline-picker › Ranking: gives way. "Shortcodes SHALL keep gemoji's order" among equal ranks no longer holds once a shortcode has usage. The change modifies it to cover the candidates with no usage, keeps its five scenarios, and adds *Used emoji come first within a rank*.
- inline-picker › The candidate list: holds. "Each candidate SHALL show the emoji" now shows the emoji in the skin tone, which `skin-tone` states.
- inline-picker › Each emoji appears once: holds. Dedup by text runs before the tone, so an alias and a shortcode of 👋 still give one candidate.
- inline-picker › A colon after an exact shortcode commits it, Enter and Tab commit the selected emoji, A click commits a candidate: hold. They commit "its emoji", which is the toned form when a skin tone is set.
- inline-picker › Aliases are searched like shortcodes: holds. "Committing an alias SHALL commit its whole text" agrees with *Alias texts never take a skin tone*.
- inline-picker › A query with no match ends the search, Enter and Tab without a candidate list, Held text is never lost: hold. They commit plain text, which *Plain text records nothing* keeps out of the usage.
- config-file › A bad config file never stops typing: holds. A bad `skin_tone` or `remember_usage` makes the whole file bad like any other bad value.
- config-file › The trigger value, Alias texts, Alias names: hold. "Any other value SHALL make the config file bad" is per key, and the new keys have their own rule.
- config-file › A missing config file means defaults: gives way. It named only the trigger and the aliases. It now adds "the default of each other key", which the new value requirements state.
- config-file › Unknown keys are ignored: holds, but its scenario used `skin_tone` as an unknown key. The scenario now uses `theme` and `[later]`.
- settings-window › Saving keeps what the window does not manage: holds, but its scenario used `skin_tone` as an unmanaged key. The scenario now uses `theme = "dark"`.
- settings-window › The settings command runs without a window: gives way. Its list of flags gains the three new ones.
- settings-window › The settings window edits the trigger, The settings window shows a bad value at its field: hold. The skin tone selector offers only good values, and the switch is a boolean.
- usage › A commit records usage and usage › Turning usage off: checked against each other. The first now starts "While `remember_usage` is `true`", so they do not contradict.
- usage › A missing or broken usage file never stops typing, The engine reads a changed usage file: consistent. A deleted file means no usage in both.
- installation › Uninstall removes what install added: holds. Install never writes the usage file, so uninstall leaves it, as it leaves the config file.
