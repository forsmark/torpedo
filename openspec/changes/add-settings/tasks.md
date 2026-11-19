# Tasks

## 1. Config rules in torpedo-core

- [x] 1.1 Add `toml_edit` to `torpedo-core` and the `config` module: `Config`, `Alias`, `Problem`, `config_path(env)`, `check_trigger`, `check_alias_name`, `check_alias_text` and `parse`, as design.md says. Verify with unit tests for each scenario of `config-file` *The trigger value*, *Alias names* and *Alias texts*, for unknown keys and tables, for a file that does not parse (with its line), for a missing `[aliases]`, and for each branch of `config_path`.
- [x] 1.2 Add `ConfigDocument` with `set_trigger`, `set_alias`, `rename_alias`, `remove_alias` and `save` (folder, temp file, `sync_all`, rename). Verify with unit tests in a temp folder: comments and unknown keys survive each edit, a refused edit leaves the document as it was, a save creates the folder, the inode changes, and no temp file is left. Also check that `¯\_(ツ)_/¯` and a name like `+2` round-trip.

## 2. Search and picker in torpedo-core

- [x] 2.1 Make the search take aliases: ranks for alias names, aliases first among equal ranks in name order, shortcodes hidden by an alias of the same name, one entry per emoji text, and `exact` with aliases first. Verify with unit tests for each alias scenario of `inline-picker` (*Ranking*, *Each emoji appears once*, *The candidate list*, *Aliases are searched like shortcodes*, *An alias wins over a shortcode of the same name*, *A query with no match ends the search*), and that the existing search tests still pass with no aliases.
- [x] 2.2 Make the picker take a `Config`: `Picker::new(config, index)`, `set_config`, and `search_opened()`. Prefix and Search keep the trigger they were typed with, and `set_config` re-runs an open query. Verify with unit tests for the triggers `:`, `;;` and `:;:`: boundary, held prefix, Backspace on a prefix, the closing `:`, Esc, and a config swap while a prefix or a search is held, with a longer and a shorter trigger.

## 3. The engine

- [x] 3.1 Add `ConfigWatch` to `torpedo-engine`: the path from `config_path`, the fingerprint (mtime in ns, size, device, inode, or missing), the last good config, and a `check()` that reads and parses only when the fingerprint changed and logs problems under `TORPEDO_DEBUG` once. Call it at start, on FocusIn and FocusInId, and after a key for which `search_opened()` is true. Verify with unit tests in a temp folder for a changed file, a same-second rewrite of another size, a broken file after a good one, and a deleted file.
- [x] 3.2 Add the property: the IBusProperty and IBusPropList shapes in `ibus_types`, the `RegisterProperties` and `UpdateProperty` signals, and `RegisterProperties` on FocusIn and FocusInId. Verify with a unit test of the serialized signature `(sa{sv}suvsvbbuvv)` inside `(sa{sv}av)`.
- [x] 3.3 Make `PropertyActivate("torpedo-settings", _)` start the settings command: `TORPEDO_SETTINGS_COMMAND`, else `torpedo-settings` next to `current_exe()`, without a shell, stdin and stdout on `/dev/null`, reaped on a thread, and a failed start logged. Verify with a unit test of the command lookup over an environment map.
- [x] 3.4 Add the IBusProperty and IBusPropList shapes, the routing of `RegisterProperties` to the panel without the PROPERTY capability, and the panel's `PropertyActivate` path to `docs/ibus-protocol.md`. Also add the GNOME Settings lookup of `ibus-setup-<engine>.desktop`. Verify that each fact names its source file in IBus or gnome-control-center.

## 4. The harness

- [ ] 4.1 Extend `e2e/harness/session.mjs`: `openSession({ config, env })`, the default stub for `TORPEDO_SETTINGS_COMMAND`, removal of `DISPLAY` and `WAYLAND_DISPLAY` in `privateEnv`, `writeConfig`, `removeConfig`, `settingsStarts()` and `engineLog()`. Extend `driver.py`: make the panel a subclass of `IBus.PanelService` that records `do_register_properties` as an event, and add a `property_activate` command. Let `openSession` remove `TORPEDO_SETTINGS_COMMAND` and take another `TORPEDO_ENGINE` for the default-command test. Verify with `e2e/harness/harness.test.mjs` cases for each helper, and that `pgrep -af ibus-daemon` then shows only PID 7674.
- [ ] 4.2 Make `scripts/e2e.mjs` build `torpedo-settings` along with the engine, and update `scripts/e2e.test.mjs`. Verify with `node --test "scripts/**/*.test.mjs"`.

## 5. The settings window

- [ ] 5.1 Add `crates/torpedo-settings` with `gtk4` 0.10 and `libadwaita` 0.8, and the binary `torpedo-settings`. Parse the arguments first. Implement `--check`, `--set-trigger`, `--set-alias`, `--remove-alias` and `--config` through `torpedo_core::config`, with no GTK call on that path. Verify with `cargo test -p torpedo-settings` for the argument parsing, and by running `--check` with `DISPLAY` and `WAYLAND_DISPLAY` unset.
- [ ] 5.2 Build the window: a single-instance `adw::Application`, the trigger row, the alias rows with add and delete, a problem shown under each bad field, a save on each good apply, and a read-only banner for a file that does not parse. Verify with `cargo clippy -p torpedo-settings --all-targets -- -D warnings`. Do not start the window in any test or on the owner's display.

## 6. Install

- [ ] 6.1 Update `install.sh`: build both binaries, install `torpedo-settings` next to the engine, write `$PREFIX/share/applications/ibus-setup-torpedo.desktop` without sudo, add `<setup>` to the component file, and add the settings paragraph to the next steps. Update `uninstall.sh` to remove the settings binary and the desktop file. Verify with `sh -n` on both, and `./install.sh --dry-run` and `./uninstall.sh --dry-run` with `HOME` in a temp folder.

## 7. Scenario tests

- [ ] 7.1 Add `e2e/config-file/*.test.mjs` with one test for each scenario of `config-file`, through the private daemon with a private config file, or through `torpedo-settings --check` with no display. Add `e2e/coverage/config-file.json`. Verify with `node scripts/e2e.mjs` and `node tools/spec-checks/bin/spec-checks coverage`.
- [ ] 7.2 Add `e2e/settings-window/*.test.mjs` with one test for each scenario of `settings-window` that a test can show: the save rules through the mode without a window, and the property scenarios through the panel. Add `e2e/coverage/settings-window.json`, with `none` and a reason for the five window-only scenarios that design.md lists. Verify as in 7.1.
- [ ] 7.3 Add the new `inline-picker` scenarios to `e2e/inline-picker/` (a new `trigger.test.mjs` and `aliases.test.mjs`), and update `e2e/coverage/inline-picker.json` for the renamed requirement *The trigger is configurable* and the new and changed scenarios. Verify as in 7.1.
- [ ] 7.4 Update `e2e/installation.test.mjs` for the new and changed `installation` scenarios: the settings binary, the desktop file and its name, `<setup>`, no sudo for the desktop file, uninstall of both, the sudo line and the settings paragraph. *The next steps after install* now checks the order of the steps, not the last six lines, because the settings paragraph comes after them. Update `e2e/coverage/installation.json`. Verify as in 7.1.

## 8. Words and docs

- [ ] 8.1 Check the glossary terms that the proposal added (Config file, Settings window) and changed (Alias, Query, Trigger) against the final code. Add the settings window, `TORPEDO_SETTINGS_COMMAND` and the new harness options to `AGENTS.md`. Verify that `spec-checks claims add-settings` shows no claim that the change contradicts.

## 9. Integration

- [ ] 9.1 On a clean tree, run `node scripts/verify.mjs --affected`, `node --test "scripts/**/*.test.mjs"` and `node tools/spec-checks/bin/spec-checks verify`. Verify that they pass (coverage may fail before the sync only on scenarios that exist only in the delta specs), that `pgrep -af ibus-daemon` shows only PID 7674, that `~/.config/ibus/bus/` is unchanged, and that `~/.config/torpedo` does not exist or is unchanged.

## Workflow follow-up

- Sync gate: send the reviewer the output of `spec-checks gate add-settings` and wait for the answer.
- Sync the delta specs and archive the change. Check that the main `inline-picker` spec has *The trigger is configurable* and no *The trigger is two colons*.
- Run `node tools/spec-checks/bin/spec-checks verify`, `node scripts/verify.mjs` and `node --test "scripts/**/*.test.mjs"`, push, and mark the PR ready.
