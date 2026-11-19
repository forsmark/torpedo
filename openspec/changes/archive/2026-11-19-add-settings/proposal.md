# Proposal

## Why

Torpedo works with a fixed trigger and gemoji's shortcodes only. The owner wants two things that Rocket on macOS has: custom names for emoji, edited in a settings window, and a trigger of their own choice. The owner uses `::`, but other people type `:` or `;;`. Both need a config file that the engine reads and the settings window writes.

## What Changes

- Add a config file at `$XDG_CONFIG_HOME/torpedo/config.toml`, default `~/.config/torpedo/config.toml`. It holds the `trigger` key and an `[aliases]` table. A missing file means defaults. A file that does not parse, or that holds a bad value, never stops typing: the engine keeps its last good config, or the defaults, and logs the problem under `TORPEDO_DEBUG=1`. Unknown keys are ignored, so a later change can add `skin_tone` and a recents file without breaking old files.
- The trigger becomes configurable, with the default `::`. A trigger is 1 to 3 characters, and no character may be a query character, whitespace or a control character. The boundary rule, the held trigger prefix, Backspace on a held prefix and the closing `:` after an exact shortcode hold with any trigger.
- Aliases join the search. An alias name uses query characters and matches without regard to case. Its text is 1 to 16 characters, so a custom emoji sequence or `¯\_(ツ)_/¯` fits. An alias ranks like a shortcode, comes first among equal ranks, and shows its own name in the candidate list. An alias with the name of a gemoji shortcode wins over it. `::lgtm:` commits the alias.
- The engine notices a changed config file without an IBus restart. It compares the file's change time, size and identity on every focus-in and when a search opens.
- Add the crate `crates/torpedo-settings` with the binary `torpedo-settings`: a GTK4 and libadwaita window that edits the trigger and the aliases. It checks each value with the rules of `torpedo-core`, shows a problem next to the field, and never writes a bad file. It writes the file in one step, with a temp file and a rename, and keeps the keys and comments it does not manage. It also has a mode without a window, for checks and scripted edits, which the tests use.
- Three ways open the settings window:
  - The engine registers an IBus property "Torpedo settings". GNOME Shell shows it in the input source menu, and activating it starts `torpedo-settings`. `TORPEDO_SETTINGS_COMMAND` overrides the command.
  - `install.sh` installs `ibus-setup-torpedo.desktop`. GNOME Settings shows a settings button for the input source when that file exists, and the app launcher lists it as "Torpedo Settings".
  - The component file names `torpedo-settings` in `<setup>`, for IBus tools that read it.
- `install.sh` builds and installs `torpedo-settings` next to the engine, and the desktop file in `$PREFIX/share/applications`. `uninstall.sh` removes both. The component file still goes through the one sudo copy.
- `docs/ibus-protocol.md` gains the shapes of IBusProperty and IBusPropList, and the path of RegisterProperties and PropertyActivate.
- The e2e harness gains options for a private config file, a stub settings command and the panel's property events.
- `GLOSSARY.md` gains the terms Config file and Settings window, and Alias, Query and Trigger are brought up to date.

### A change to decision 6

The brief says that `<setup>` in the component file makes GNOME Settings show a settings button. It does not. GNOME Settings looks for a desktop file named `ibus-setup-<engine>.desktop` (`panels/keyboard/cc-input-source-ibus.c`, `cc_input_source_ibus_get_app_info`). So the desktop file that the brief puts in the app launcher gets that name, and one file does both jobs. The component file keeps `<setup>` as the brief says.

Out of scope: recent emoji and skin tones. A search for the alias's emoji inside the window is also out of scope. The window takes the emoji in a text field.

## Capabilities

### New Capabilities

- `config-file`: where the config file lives, which values it holds and which it refuses, what the engine does with a missing, broken or changed file.
- `settings-window`: the `torpedo-settings` program. What it edits, how it checks and writes the config file, its mode without a window, and how the user opens it from the input source menu.

### Modified Capabilities

- `inline-picker`: the trigger comes from the config file, and the trigger rules hold for any trigger. Aliases join the search, the ranking, the candidate list and the closing colon.
- `installation`: install and uninstall add and remove the settings window and its desktop file. The component file names the settings window in `<setup>`. The next steps mention the settings window.

## Impact

- New crate: `crates/torpedo-settings`, with `gtk4` 0.10 and `libadwaita` 0.8 (system libraries gtk4 4.22.4 and libadwaita-1 1.9.1). The workspace build now needs the GTK4 and libadwaita development files. The owner's machine has them, and the repo has no CI.
- New module `config` in `torpedo-core`, with `toml_edit`. The engine and the window share it.
- Changed code: `torpedo-core` search and picker, `torpedo-engine` engine object.
- Changed files: `install.sh`, `uninstall.sh`, `e2e/harness/session.mjs`, `e2e/harness/driver.py`, `e2e/installation.test.mjs`, `e2e/coverage/*.json`, `docs/ibus-protocol.md`, `GLOSSARY.md`, `AGENTS.md`.
- New e2e files under `e2e/config-file/` and `e2e/settings-window/`, and new ones under `e2e/inline-picker/`.
- No test opens a window. Each test that runs `torpedo-settings` uses its mode without a window and unsets `DISPLAY` and `WAYLAND_DISPLAY`.

## Claims checked

`spec-checks claims add-settings --all` listed three claims in requirements the change does not touch, and eight of its own. I also read every requirement of `inline-picker` that names a shortcode or the trigger, because aliases and the trigger reach further than the shared glossary terms.

- inline-picker › The engine asks every app for surrounding text: holds. The config check on focus-in adds a `stat` and asks nothing else of the app.
- inline-picker › Without surrounding text the last typed character decides: holds. A config reload does not touch the last character.
- inline-picker › Enter and Tab without a candidate list: holds. An alias only adds candidates.
- inline-picker › The trigger is two colons: gives way. It says the trigger SHALL be `::`. The change renames it to *The trigger is configurable* and modifies it.
- inline-picker › The candidate list, Each emoji appears once, Ranking, A colon after an exact shortcode commits it, Another character closes the search, A query with no match ends the search: each names shortcodes only, or says ties keep gemoji's order. The change modifies each to include aliases. Ranking now says aliases come first among equal ranks.
- inline-picker › The trigger counts only at a boundary, A trigger prefix is held, Backspace removes a held trigger prefix: their scenarios use `::`. They hold for any trigger, which the new requirement *The trigger rules hold for any trigger* states with `;;`. *The trigger is configurable* says the other scenarios use the default trigger.
- inline-picker › Held text is never lost: holds. A reload at focus-in runs after the picker drops its state, and a swap during held text keeps the typed trigger (design.md, "The picker takes a config").
- config-file › A missing config file means defaults, A bad config file never stops typing, The trigger value, Alias names, Alias texts: consistent with each other. One bad value makes the whole file bad, and a missing file is the defaults.
- settings-window › The settings window edits the trigger, The settings window shows a bad value at its field: consistent with *Saving refuses a bad value*. Both use the rules that `config-file` owns, which no other spec repeats.
- installation › Install writes the component file: gives way. It gains `<setup>`, and the one `$SUDO` call stays. *The component file goes through sudo* still holds.
- installation › Uninstall removes what install added: gives way. It now also removes the settings window and the desktop file.
