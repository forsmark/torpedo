# Design

## Context

The engine of `add-emoji-engine` takes its trigger as a parameter. `Picker::new(trigger, index)` gets `"::"` from the engine, and the design of that change says a config change passes another value and changes nothing else. The search ranks gemoji shortcodes only. The engine object answers `PropertyActivate`, `PropertyShow` and `PropertyHide` with empty methods and registers no property.

Facts this design rests on, checked for this change:

- GNOME Settings shows a settings button for an IBus input source when a desktop file named `ibus-setup-<engine>.desktop` exists. It does not read `<setup>` (`panels/keyboard/cc-input-source-ibus.c`, `cc_input_source_ibus_get_app_info`, which splits the engine name at `:` and calls `g_desktop_app_info_new`). So torpedo's file is `ibus-setup-torpedo.desktop`. GIO finds it in `$XDG_DATA_HOME/applications`, which is `~/.local/share/applications` by default.
- `<setup>` is a child of `<engine>` in the component file (`IBusEngineDesc`). Some IBus tools read it.
- An IBusProperty serializes as `(sa{sv}suvsvbbuvv)`: key, type, label (IBusText), icon, tooltip (IBusText), sensitive, visible, state, sub-properties (IBusPropList) and symbol (IBusText). An IBusPropList is `(sa{sv}av)`. Type 0 is NORMAL and state 0 is UNCHECKED. This was read from `IBus.Property.serialize_object()` of libibus 1.5.34.
- The daemon sends an engine's `RegisterProperties` to the client only when the client has the PROPERTY capability. Otherwise it goes to the panel, for the focused input context (`bus/inputcontext.c`, `bus_input_context_register_properties`, and `bus/panelproxy.c`, `_context_register_properties_cb`). GNOME Shell is the panel and shows the properties in the input source menu.
- A panel's `PropertyActivate(s name, u state)` signal reaches the engine of the focused input context as the method `PropertyActivate` (`bus/panelproxy.c`, `bus_panel_proxy_property_activate`). A client can also call `PropertyActivate` on its input context.
- gtk4-rs 0.10 and libadwaita-rs 0.8 build against the system gtk4 4.22.4 and libadwaita-1 1.9.1 (checked with a throwaway `cargo check`). `toml_edit` 0.25 parses TOML and edits it with its comments kept.

## Goals / Non-Goals

**Goals:**

- One set of config rules, in `torpedo-core`, used by the engine and the settings window.
- A bad or half-written config file can never stop typing.
- The engine sees a saved change without an IBus restart and without a thread of its own.
- Every engine-visible scenario has an e2e test through the private daemon. The save rules of the settings window have e2e tests through its mode without a window. No test opens a window.

**Non-Goals:**

- Recent emoji and skin tones. The file format leaves room for them (see "Room for later keys").
- An emoji search inside the settings window. The alias text is a plain text field.
- Config for the layout `dk` in the component file.

## Decisions

### The config file format

```toml
# Torpedo settings. torpedo-settings keeps this comment.
trigger = "::"

[aliases]
lgtm = "👍"
shrug = '¯\_(ツ)_/¯'
"+2" = "👍👍"
```

- `trigger` is a top-level string. `[aliases]` is a table of strings.
- `¯\_(ツ)_/¯` needs a literal string in single quotes, or `\\` in a basic string, because `\_` is not a TOML escape. The settings window writes a basic string with the escapes that `toml_edit` picks, so the user never has to know this.
- A name with `+` or `-` at the start needs a quoted key. `toml_edit` quotes keys when needed.
- A length counts Unicode code points (`char` in Rust). `¯\_(ツ)_/¯` is 9. The longest gemoji emoji, a flag with tag characters, is 7. 16 leaves room for a short sequence of emoji.
- An alias text may hold no control character, so a commit can never hold a line break or a tab.

### One bad value makes the whole file bad

The brief says a bad value must not stop typing and the engine keeps its last good config. The simplest rule that holds that: a file with any bad value is not applied at all. A partial apply would make the result depend on which value was bad, and the user could not tell which half of the file is live. The settings window never writes a bad file, so only a hand edit gets here. `TORPEDO_DEBUG=1` and `torpedo-settings --check` name every problem.

### The config module in torpedo-core

`torpedo_core::config` holds:

- `Config { trigger: String, aliases: Vec<Alias> }`, with `Alias { name, text }`. `Config::default()` is `::` and no aliases. Aliases are sorted by lower-case name, which gives the tie order of the ranking.
- `config_path(env) -> PathBuf`: `$XDG_CONFIG_HOME/torpedo/config.toml` when that is an absolute path, else `$HOME/.config/torpedo/config.toml`. It takes the environment as a parameter, so unit tests cover each branch.
- `parse(text) -> Result<Config, Vec<Problem>>`. It parses with `toml_edit`, reads `trigger` and `[aliases]`, skips every other key and table, and checks each value. A `Problem` names the key (`trigger`, or `aliases.<name>`), says what is wrong in one sentence, and holds the line when it has one.
- `check_trigger(&str)`, `check_alias_name(&str)` and `check_alias_text(&str)`. The settings window calls these per field. `parse` calls the same functions.
- `ConfigDocument`: a `toml_edit::DocumentMut` with `set_trigger`, `set_alias`, `rename_alias` and `remove_alias`. Each edit checks the whole result with `parse` before it is allowed. `save(path)` writes it (see "Saving replaces the file in one step").

The crate is still free of D-Bus and GTK. Reading and writing one file is the only I/O it adds.

Alternative considered: the `toml` crate with serde for reading and `toml_edit` for writing. Two parsers could disagree on an edge case, and then the window would write a file that the engine refuses. `toml_edit` alone parses and edits.

### Search with aliases

`EmojiIndex::search(query)` becomes `search(query, aliases)` with the same ranks: 0 exact, 1 prefix, 2 word prefix, 3 substring. The steps:

1. Lower-case the query.
2. Rank each alias name, lower-cased, against the query.
3. Rank each gemoji shortcode, but skip a shortcode whose name equals an alias name. That is how an alias wins over a shortcode of the same name.
4. Group the hits by the emoji text. An emoji takes its best hit. Among equal ranks an alias hit beats a shortcode hit.
5. Sort by rank, then aliases before gemoji entries, then alias name order or gemoji's order.

The candidate shows the alias name as the file writes it. `exact(query, aliases)` returns an alias text first, then a shortcode's emoji that no alias hides. An alias whose text is a gemoji emoji merges with that emoji's entry in step 4, so the emoji shows once.

### The picker takes a config

`Picker::new(config, index)` and `Picker::set_config(config)`. `set_config` may run in any state.

- `State::Prefix` and `State::Search` keep the trigger they were typed with. The plain text of a search is that trigger plus the query, and the next character of a prefix comes from that trigger. So a config swap in the middle of held text cannot index past a shorter trigger, and the plain text stays what the user typed.
- `set_config` runs the search again for an open query, so new aliases show at once.
- The closing character stays `:` with any trigger, as the brief says. It is a constant, not a config value.

`Picker::search_opened()` tells whether the last key opened a search. The engine uses it to time the reload.

### Reload by fingerprint, without a thread

The engine keeps a `ConfigWatch` per engine object: the path, the last fingerprint and the last good config. A fingerprint is the file's modification time in nanoseconds, its size, its device and its inode, or "missing". Size and inode catch two saves within one timestamp tick, and a rename always gives a new inode.

- On FocusIn and after a key that opened a search, the engine takes one `stat`. When the fingerprint differs from the last one, it reads and parses the file.
- A good file becomes the config, and the engine calls `set_config`. A bad file leaves the config as it was and logs each problem under `TORPEDO_DEBUG`. A missing file gives the defaults.
- The engine logs only when the fingerprint changed, so a bad file logs once, not on every focus-in.
- At start the engine reads the file once. A bad file then means the defaults.

"The first key of each search" in the brief is read as the key that completes the trigger. At that moment the search is open with an empty query, so new aliases apply to the query that follows. A new trigger applies from the next key that finds no text held, as `config-file` says. The settings window takes the focus while the user edits, so the focus-in when the user comes back always picks up a new trigger.

Alternative considered: an inotify thread. It would react faster, but it adds a thread, a channel into the zbus object server, and the edge cases of watching a folder that may not exist yet. The two checks cost one `stat` each and always run before the change matters. No inotify thread.

### Room for later keys

- The engine and the window ignore every key and table they do not know, and the window keeps them when it saves. A later change adds `skin_tone = "..."` as a top-level key.
- Recent emoji go in their own file, such as `$XDG_STATE_HOME/torpedo/recents`. The engine writes them on every commit. Keeping them out of `config.toml` means the engine never writes the user's config file, and the window never races the engine.

### The settings property

- The engine emits `RegisterProperties` with one IBusProperty on every FocusIn: key `torpedo-settings`, type NORMAL, label "Torpedo settings", icon `preferences-system`, tooltip "Open the Torpedo settings window", sensitive and visible, state UNCHECKED, no sub-properties, empty symbol. IBus engines register their properties on focus-in, because the panel shows the properties of the focused input context.
- `PropertyActivate("torpedo-settings", _)` starts the settings command. Other names do nothing.
- The settings command is `TORPEDO_SETTINGS_COMMAND` when it is set and not empty. Otherwise it is `torpedo-settings` in the folder of `std::env::current_exe()`. Install puts both binaries in `$PREFIX/lib/torpedo/`, so this finds the installed window. The value is one program path, run without a shell and without arguments.
- The engine starts it with stdin and stdout on `/dev/null` and stderr inherited, and reaps it on a small thread so no zombie stays. A failed start is logged under `TORPEDO_DEBUG` and changes nothing else.
- The window is a single-instance `adw::Application` with the id `io.github.forsmark.Torpedo.Settings`. A second start shows the open window.

### The settings window

- An `adw::ApplicationWindow` with an `adw::PreferencesPage` of two groups.
- "Trigger": one `adw::EntryRow` with an apply button. Apply checks with `check_trigger`. A good value is saved. A bad value adds the `error` style to the row and shows the problem in a label under it.
- "Aliases": one row per alias, with an entry for the name, an entry for the text and a delete button. A row at the end adds an alias. Each apply checks the name and the text and saves when both are good. A problem shows under the row.
- Each good edit saves at once, as libadwaita preference windows do. There is no Save button.
- When the file does not parse, the window shows an `adw::Banner` with the problem and its line, and the fields are read-only. The user fixes the file by hand. Saving a parsed copy would drop the user's text.
- When the file parses but holds a bad value, the window shows the values with the problem at each bad field. A save checks the whole file, so it waits until every bad field is fixed.

### The mode without a window

`torpedo-settings` parses its arguments before it touches GTK. With any of `--check`, `--set-trigger <t>`, `--set-alias <name>=<text>`, `--remove-alias <name>`, it runs the edit or the check through `torpedo_core::config` and exits. `--config <file>` replaces the default path. `--check` prints `<path>: ok`, or one line per problem as `<path>:<line>: <key>: <problem>`, and exits 1 on a problem. An edit that is refused prints the problem and exits 1. It never calls `gtk::init`, so it runs with no display.

The window's apply handlers call the same `ConfigDocument` functions as these flags. The e2e tests of the save rules use the flags. What only the window shows, such as where an error appears, has a `none` entry in the coverage map.

### Saving replaces the file in one step

`ConfigDocument::save` creates the folder, writes the whole text to a temp file in the same folder (`.config.toml.<pid>.tmp`), calls `sync_all`, and renames it over `config.toml`. A rename in one folder is atomic, so the engine reads the old file or the new one. A failed write removes the temp file.

### Install and uninstall

- `install.sh` runs `cargo build --release -p torpedo-engine -p torpedo-settings`, then `install -D -m 755` for both binaries into `$PREFIX/lib/torpedo/`.
- It writes `$PREFIX/share/applications/ibus-setup-torpedo.desktop` without sudo, with `Name=Torpedo Settings`, `Exec=<absolute path>`, `Icon=preferences-desktop-keyboard`, `Type=Application` and `Categories=Settings;`. `PREFIX` defaults to `~/.local`, so the file lands in `~/.local/share/applications`. The tests already point `PREFIX` at a temp folder.
- The component file gets `<setup>` with the absolute path of the settings window, inside `<engine>`. It still goes through the one `$SUDO install` call, so the existing test of exactly one sudo call holds.
- `uninstall.sh` removes both binaries, the folder and the desktop file, and the component file through `$SUDO`.
- The next steps add one paragraph: the settings window is "Torpedo Settings" in the app launcher, the settings button of the input source in GNOME Settings, and "Torpedo settings" in the input source menu of the top bar.

### The e2e harness

- `openSession({ config, env })`. `config` is the text of `config.toml`, written to `<run>/config/torpedo/config.toml` before the daemon starts. `env` adds variables, such as `TORPEDO_DEBUG=1`.
- `privateEnv` always sets `TORPEDO_SETTINGS_COMMAND` to a stub script in the run folder that appends a line to `<run>/settings-starts` and exits. A test that needs a long-running stub passes its own. `privateEnv` also removes `DISPLAY` and `WAYLAND_DISPLAY`. The daemon and the engine need neither, and with them gone no test can start a window on the owner's display.
- `session.writeConfig(text)` and `session.removeConfig()` change the file during a test. `writeConfig` writes a temp file and renames it, as the window does.
- The driver connects the panel's `register-properties` signal and records the property list as an event. A new command `property_activate` emits `PropertyActivate` from the panel, as `click` emits `CandidateClicked`.
- `session.engineLog()` reads the engine's stderr from `daemon.log`, which `--verbose` fills.
- Settings tests run `target/release/torpedo-settings` with a private `HOME` and `XDG_CONFIG_HOME`, and with `DISPLAY` and `WAYLAND_DISPLAY` removed. A helper refuses to run it when either is set.
- `scripts/e2e.mjs` builds `torpedo-settings` along with the engine.

### Coverage

The coverage adapter reads `node --test` files under `e2e/` only. So each scenario maps to an e2e file, as the existing ones do with `unit` entries. The Rust tests in `torpedo-core` cover the same rules in more detail and map to no scenario. These scenarios get `none`, because only a person looking at the window can see them: *The trigger field*, *The alias list*, *Add, change and delete an alias*, *A bad trigger in the window* and *A bad value is not written*. The save rules behind the last two have their own scenarios under *Saving refuses a bad value*, which the mode without a window shows.

## Risks / Trade-offs

- [The workspace build now needs the GTK4 and libadwaita development files] → The owner's machine has them, and the repo has no CI. `cargo check --workspace` fails on a machine without them. A later change can move the window out of the default members if that matters.
- [A changed trigger waits for the next focus-in] → The settings window takes the focus, so the user always comes back through a focus-in. A hand edit in a terminal also moves the focus. Only a script that edits the file while the user keeps typing in one field sees the old trigger until the next focus change.
- [A file with one bad value is ignored as a whole] → The window never writes one, and `torpedo-settings --check` names each problem.
- [GNOME Shell's input source menu is not tested] → The harness checks the property list that the panel receives and the activation path. The owner checks the menu after install.
- [The `<setup>` tag has no visible effect in GNOME] → It costs one line and helps other IBus tools. The desktop file gives the GNOME Settings button.
- [The engine starts a program from an environment variable] → The variable comes from the environment of ibus-daemon, which the user controls. The engine never runs it through a shell.

## Migration Plan

1. The owner runs `./install.sh` again. It asks for the sudo password once, for the component file with its new `<setup>`.
2. The owner runs `ibus restart`. The engine starts with the defaults, so typing works as before with no config file.
3. To roll back, the owner runs `./uninstall.sh`, or installs the previous version. A config file left behind does no harm.
