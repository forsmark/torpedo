# torpedo

Torpedo is an inline emoji picker for Linux, like Rocket on macOS. You type a trigger such as "::" and a Slack-style name in any text field, and torpedo shows matching emoji at the cursor and inserts the one you pick. It is for the owner's GNOME Wayland desktop.

Stack: Rust (stable, via rustup) and Cargo. IBus input method engine over D-Bus with zbus. GTK4 with gtk4-rs for the settings window. Emoji data from gemoji, settings in TOML. Rust tests with cargo test, and scenario tests with `node --test`.

## Layout

- `crates/torpedo-core` is a library for the pure logic: trigger detection, search and config. It uses no D-Bus and no GTK.
- `crates/torpedo-engine` is the binary of the IBus engine. It reads the config file and the usage file on focus-in and when a search opens. It writes the usage file `$XDG_STATE_HOME/torpedo/usage.toml` after each emoji commit. Its "Torpedo settings" property starts the settings command: `TORPEDO_SETTINGS_COMMAND` when it is set, else `torpedo-settings` next to the engine binary.
- `crates/torpedo-settings` is the settings window, a GTK4 and libadwaita binary. With `--check`, `--set-trigger`, `--set-alias`, `--remove-alias`, `--set-skin-tone`, `--set-remember-usage` or `--clear-usage` it opens no window and needs no display. The config rules it uses live in `torpedo-core`.
- `e2e/` holds the scenario tests. They are `node --test` files that drive the real engine binary through a private IBus daemon. The coverage map in `e2e/coverage/` links each spec scenario to one of them.
- `e2e/harness/` runs that daemon. `session.mjs` starts `dbus-run-session` with `session.conf` and runs `driver.py` in it. The driver starts a private ibus-daemon, acts as the IBus client and panel, and answers JSON commands. `app.mjs` folds its events into the app's text. Use `openSession()` and never start an ibus-daemon another way. `openSession({ config, env, engine })` writes a private config file, adds variables, and can drive another engine binary. Every session removes `DISPLAY` and `WAYLAND_DISPLAY`, points `TORPEDO_SETTINGS_COMMAND` at a recording stub, and points `XDG_STATE_HOME` into its run folder. It refuses to start when the usage file would be outside the run folder.
- `e2e/harness/settings.mjs` runs `torpedo-settings` in its mode without a window, with a private `HOME`, `XDG_CONFIG_HOME` and `XDG_STATE_HOME` and no display.
- `docs/ibus-protocol.md` holds what torpedo knows about the IBus protocol.
- `install.sh` and `uninstall.sh` install the engine for the owner.
- Rust `#[test]`s cover internals. They map to no scenario.

## Commands

Rust is installed per user. Run `source ~/.cargo/env` before you call cargo by hand. The verify and e2e scripts find cargo without it. Building the workspace needs the system packages `libgtk-4-dev` and `libadwaita-1-dev`, for the settings window.

- `node scripts/verify.mjs --affected` runs the checks and the tests that the changes since origin/main can reach. Run it once, at the end of your work, on a clean tree.
- `node scripts/verify.mjs` runs everything. It runs before a release.
- `node tools/spec-checks/bin/spec-checks verify` runs the spec checks. Run it before you archive a change, and after a merge that touched `openspec/`.
- `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` are the lint steps.
- `cargo check --workspace --all-targets` is the typecheck step.
- `node scripts/heavy-lock.mjs cargo test --workspace` runs the Rust tests. The skin tone test in `torpedo-core` needs `python3`, `gir1.2-ibus-1.0` and `ibus-data`, as the e2e harness does, and fails without them.
- `node scripts/heavy-lock.mjs cargo build --workspace` builds the crates.
- `node scripts/e2e.mjs` builds the release engine and settings window and runs the scenario tests under `e2e/`, one file at a time.
- `node --test e2e/<file>.test.mjs` runs one scenario test file. Build first with `node scripts/heavy-lock.mjs cargo build --release -p torpedo-engine -p torpedo-settings`.
- `./install.sh --dry-run` and `./uninstall.sh --dry-run` print what the scripts would do and write nothing. Only the owner runs `./install.sh` or `./uninstall.sh` for real. They take `PREFIX`, `COMPONENT_DIR` and `SUDO`, and the tests set all three to a temp folder.
- `pnpm install` installs the Node dev dependencies. The coverage check needs TypeScript from them.

Heavy jobs wait for one machine-wide lock. Start other heavy commands with `node scripts/heavy-lock.mjs <command>`. The verify and e2e scripts take the lock themselves.

## Safety: never touch the live IBus session

The owner's desktop types through IBus. A mistake with the live IBus session stops all typing.

- Never change gsettings, such as `org.gnome.desktop.input-sources`.
- Never run `ibus restart`, `ibus engine`, `ibus write-cache` or any other `ibus` command against the live daemon.
- Never write to `~/.config/ibus`, `~/.cache/ibus`, `~/.config/environment.d`, `~/.config/systemd` or `/usr`.
- Tests run only against a private ibus-daemon inside `dbus-run-session`. Point `IBUS_ADDRESS_FILE`, `XDG_CONFIG_HOME`, `XDG_CACHE_HOME`, `XDG_DATA_HOME` and `XDG_STATE_HOME` at a temp dir. Without `IBUS_ADDRESS_FILE`, a private daemon overwrites the live address file.
- Stop each private ibus-daemon with SIGTERM and wait for it to exit before its `dbus-run-session` ends. Never leave one to die when its bus goes away. A crashed private daemon makes Ubuntu show the owner an "IBus Preferences has closed unexpectedly" crash dialog.
- Never open the settings window on the owner's display. A test that runs `torpedo-settings` gives it an action such as `--check`, and runs it with `DISPLAY` and `WAYLAND_DISPLAY` unset.
- Never write to the owner's `~/.config/torpedo` or `~/.local/state/torpedo`. Tests point `XDG_CONFIG_HOME`, `XDG_STATE_HOME` and `HOME` at a temp dir.
- Install onto the live desktop only when the owner asks.

## A change to behavior goes through OpenSpec

The specs in `openspec/specs/` say what the system does. A change to what a spec says, or new behavior no spec describes, goes through an OpenSpec change: propose, apply, archive. Load the skill `spec-checks:practice` before each step.

- The owner answers both gates: the proposal gate after propose, and the sync question before archive.
- The base branch is `main`. One change goes in one pull request, from proposal to archive.
- A fix that makes the code do what a spec already says needs no change.

## Words

`GLOSSARY.md` holds the domain words, and they are binding. Use its term, never a synonym. A change that brings a new concept adds its term to the glossary in the same change.

Write specs, docs and commit messages in plain English: one idea per sentence, active voice, one name per thing.
