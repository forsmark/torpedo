# Tasks

## 1. A private state folder for every test

- [x] 1.1 In `e2e/harness/session.mjs`, make `privateEnv` set `XDG_STATE_HOME` to `<run>/state`, and refuse a session whose `XDG_STATE_HOME` is outside the run folder, or removed while `HOME` is outside it. In `e2e/harness/settings.mjs`, add a `state` folder to `sandbox()`, set `XDG_STATE_HOME` in `settingsEnv`, and add `--set-skin-tone`, `--set-remember-usage` and `--clear-usage` to `ACTIONS`. Verify with new cases in `e2e/harness/harness.test.mjs` for both guards, and that the existing harness tests pass.
- [x] 1.2 Add `usageFile`, `writeUsage(entries)`, `readUsage()` and `removeUsage()` to the session, as design.md says, with `readUsage` through Python's `tomllib`. Make `usePicker` remove the usage file before each test. Verify with harness tests that write, read and remove a usage file in a private session.

## 2. Usage in torpedo-core

- [x] 2.1 Add `torpedo_core::usage`: `Usage`, `usage_path(env)`, `parse`, `to_text`, `record(text, now)`, `ranking()` and `save(path)`, with the score, the limit of 200 and the bad entries of design.md. Verify with unit tests for each branch of `usage_path`, the score after 0, 14 and 28 days, a clock that went back, a commit on a faded score, the cut to 200 with a tie, each kind of bad entry, a file of more than 200 entries, a round trip of `¯\_(ツ)_/¯`, and that `ranking()` orders entries as `score(now)` does for several values of `now`.
- [x] 2.2 Make the search rank used texts first within each rank: sort by `(rank, unused, −key, order)`. Verify with unit tests for each scenario of inline-picker *Used emoji come first within a rank*, and that every existing search test passes with no usage.

## 3. The skin tone in torpedo-core

- [x] 3.1 Add `torpedo_core::tone`: `SkinTone` with its six config strings and five modifiers, the Emoji_Modifier_Base ranges of Unicode 16.0 with their source, and `apply`. Verify with unit tests for each scenario of skin-tone *The toned form of an emoji*, and that `apply` with `None` returns its input.
- [x] 3.2 Add the oracle test: Python gi prints the sequences of `/usr/share/ibus/dicts/emoji-en.dict` as hex, and every gemoji entry with `skin_tones` in each of the five tones must be one of them. Verify that it passes, and that it fails when the ZWJ rule is taken out of `apply`.
- [x] 3.3 Read `skin_tones` in `Emoji`, add `toned` to `Match` and to the result of `exact_with`, and note the field in `crates/torpedo-core/data/gemoji/README.md`. Verify with unit tests that 👋 `wave` is toned, an alias hit of 👋 is not, and 🔥 is not.

## 4. Config keys in torpedo-core

- [x] 4.1 Add `skin_tone` and `remember_usage` to `Config`, `parse`, `ConfigDocument` (`set_skin_tone`, `set_remember_usage`) and `with_values`. Verify with unit tests for each scenario of config-file *The skin_tone value* and *The remember_usage value* that `--check` shows, that a refused edit leaves the document as it was, and that comments and unknown keys survive both edits.

## 5. The picker

- [x] 5.1 Add `Picker::set_usage(&Usage)` and `Outcome::used`. Apply the tone to candidate labels and commits when `toned` is true, and set `used` to the untoned text on Enter, Tab, a click and the closing colon. Verify with unit tests: each commit path sets `used`, Esc, a key that passes through and a query with no match leave it `None`, an alias commit is never toned, and `set_usage` during an open search re-sorts the candidates.

## 6. The engine

- [x] 6.1 Move the fingerprint into its own module and add `UsageWatch`: check at start, on FocusIn and FocusInId and when a search opens, an empty usage for a missing or bad file, a log once under `TORPEDO_DEBUG`, the fingerprint of its own write, and the rename of a broken file to `usage.toml.broken` before the next write replaces it. Verify with unit tests in a temp folder for a new file, a deleted file, a broken file, the rename over an older `usage.toml.broken`, no rename of a missing file, and the engine's own write.
- [ ] 6.2 Record each outcome with `used` when `remember_usage` is true, with `SystemTime::now()`, save it, and pass the usage to the picker. Pass an empty usage when `remember_usage` is false. Verify with `cargo test -p torpedo-engine` and the e2e tests of group 8.

## 7. The settings command and window

- [x] 7.1 Add `--set-skin-tone`, `--set-remember-usage` and `--clear-usage` to `torpedo-settings`, with no GTK call on that path. Verify with `cargo test -p torpedo-settings` for the argument parsing.
- [x] 7.2 Add the "Emoji" group to the window: the skin tone combo row with a 👋 in each tone, the switch row, and the "Clear usage" row with its `AdwAlertDialog`. Verify with `cargo clippy -p torpedo-settings --all-targets -- -D warnings`. Do not start the window in any test or on the owner's display.
- [x] 7.3 Add the new flags, the private state folder, and one line that says `cargo test` needs Python gi and `ibus-data` for the skin tone oracle test, to `AGENTS.md`. Verify that `AGENTS.md` names every flag that `torpedo-settings --help` prints.

## 8. Scenario tests

- [ ] 8.1 Add `e2e/usage/*.test.mjs` with one test for each scenario of `usage`, and `e2e/coverage/usage.json`. Verify with `node scripts/e2e.mjs` and `node tools/spec-checks/bin/spec-checks coverage`.
- [ ] 8.2 Add `e2e/skin-tone/*.test.mjs` with one test for each scenario of `skin-tone` but the oracle, and `e2e/coverage/skin-tone.json` with the `none` entry that design.md gives. Verify as in 8.1.
- [ ] 8.3 Add the tests of inline-picker *Used emoji come first within a rank*, and check that the *Ranking* tests still hold with the fixture's empty usage. Update `e2e/coverage/inline-picker.json`. Verify as in 8.1.
- [ ] 8.4 Add the tests of config-file *The skin_tone value*, *The remember_usage value* and the defaults, and change *A key from a later version* to `theme` and `[later]`. Update `e2e/coverage/config-file.json`. Verify as in 8.1.
- [ ] 8.5 Add the tests of the new settings-window scenarios for the flags, change *A comment and an unknown key survive* to `theme = "dark"`, and add the four `none` entries that design.md gives. Update `e2e/coverage/settings-window.json`. Verify as in 8.1.

## 9. Integration

- [ ] 9.1 On a clean tree, run `node scripts/verify.mjs --affected`, `node --test "scripts/**/*.test.mjs"` and `node tools/spec-checks/bin/spec-checks verify`. Verify that they pass (coverage may fail before the sync only on scenarios that exist only in the delta specs), that `pgrep -af ibus-daemon` shows only PID 7674, that `~/.config/ibus/bus/` is unchanged, and that `~/.config/torpedo` and `~/.local/state/torpedo` do not exist or are unchanged.

## Workflow follow-up

- Sync gate: send the reviewer the output of `spec-checks gate add-recents-and-skin-tone` and wait for the answer.
- Sync the delta specs and archive the change. Check that the main specs `usage` and `skin-tone` exist with their Purpose, and that `inline-picker` *Ranking* holds the new text.
- Run `node tools/spec-checks/bin/spec-checks verify`, `node scripts/verify.mjs` and `node --test "scripts/**/*.test.mjs"`, push, and mark the PR ready.
