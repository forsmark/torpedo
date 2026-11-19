# Design

## Context

The stack change left two skeleton crates and an e2e runner that passes with no tests. A spike built a working engine with zbus 5 against a private IBus 1.5.34 daemon and wrote down the wire protocol. Its durable findings move to `docs/ibus-protocol.md` in this change. The facts this design rests on:

- The daemon starts the `<exec>` of the component file when the engine is first selected. It calls `CreateEngine` on `/org/freedesktop/IBus/Factory` as soon as the bus name appears.
- Each IBus object travels as a struct `(s type, a{sv} attachments, ...)` wrapped in `v`. A zvariant `Value` inside a tuple or a `Vec` is already a `v`. A second wrap breaks the nested attribute list.
- The daemon reads `FocusId` and `ActiveSurroundingText` once per engine name. Only with `ActiveSurroundingText = true` does it ask each input context for surrounding text on focus-in.
- The daemon does not pass `IBUS_ADDRESS` to the engine it starts. The engine inherits `IBUS_ADDRESS_FILE` only when the daemon had it.
- Keyvals reach the engine as the compositor produced them. The `dk` layout gives `:` as keyval 0x3a with the Shift bit set, and `{` with the Mod5 bit set.

See proposal.md for the motivation.

## Goals / Non-Goals

**Goals:**

- All picker logic lives in `torpedo-core` and runs in unit tests without D-Bus.
- The engine crate only translates between D-Bus and the core.
- Every scenario has an e2e test that drives the real binary through a private daemon, and no test can reach the live IBus session.

**Non-Goals:**

- Aliases, the config file, the settings window, recent emoji and skin tones.
- A user-level component folder. IBus 1.5.34 has none (the code is under `#if 0`), so the component file goes to `/usr/share/ibus/component/`.
- A live test on GNOME Shell. The owner does that after install.

## Decisions

### The picker is a state machine in torpedo-core

`torpedo_core::picker::Picker` takes one event and returns a list of actions. Events: a key press (keyval and modifier state), a key release, focus-in, focus-out, reset, disable, surrounding text, and a candidate click. Actions: commit text, update the preedit, update or hide the candidate list, and a final verdict of consumed or passed through. The engine crate turns each action into a D-Bus signal in order, then returns the verdict.

`Picker::new(trigger: &str, emoji: &EmojiIndex)` takes the trigger as a parameter. The engine passes `"::"`. The config change later passes the configured value and changes nothing else.

The states are Idle, Prefix (a proper prefix of the trigger is held) and Search (the trigger and a query are held, with the ranked matches and the selected index). Every key falls into one class:

| Key class | Idle | Prefix | Search |
|---|---|---|---|
| Release, or a modifier key alone | pass | pass, keep held | pass, keep held |
| Ctrl, Alt or Super held | pass | commit held, pass | commit plain, close, pass |
| Dead key | pass | commit held, pass | commit plain, close, pass |
| Next trigger character | start Prefix or Search if at a boundary, else pass | extend; at full trigger open Search | (see `:` and others) |
| Query character | pass | commit held + char, consume | append; no match: commit plain, close |
| `:` | trigger rules | trigger rules | exact shortcode: commit emoji; else commit plain + `:` |
| Any other character | pass | commit held + char, consume | commit plain + char, close, consume |
| Backspace | pass | drop held, consume | pop query; empty query: drop all, close; consume |
| Up, Down, Page Up, Page Down | pass | commit held, pass | with list: move, consume. No list: commit plain, close, pass |
| Enter, Tab | pass | commit held, pass | with list: commit selected, consume. No list: commit plain, close, pass |
| Esc | pass | commit held, pass | commit plain, close, consume |
| Any other key without a character | pass | commit held, pass | commit plain, close, pass |

"Commit held + char" and "commit plain + char" re-run the character through the Idle rules with the committed text as the last character. In practice that always gives one commit of the held text and the character together.

Focus-in, focus-out, reset and disable put the picker in Idle and forget what it knew about the text before the cursor. They commit nothing (see "Held text is never lost").

### Keyvals become characters

- A keyval in 0x20 to 0x7e or 0xa0 to 0xff is its own code point. That covers ASCII and the Danish `æ ø å`.
- A keyval of 0x01000000 plus a code point is that code point.
- The keypad keys `KP_0` to `KP_9`, `KP_Add`, `KP_Subtract`, `KP_Decimal`, `KP_Multiply`, `KP_Divide` and `KP_Space` map to their characters.
- Every other keyval has no character. Dead keys are 0xfe50 to 0xfe8f. Modifier keys are 0xffe1 to 0xffee and 0xfe01 to 0xfe0f, which includes `ISO_Level3_Shift` for AltGr.

The pass-through modifier mask is Control (1<<2), Mod1 (1<<3), Mod4 (1<<6), Super (1<<26), Hyper (1<<27) and Meta (1<<28). Shift, Lock (Caps Lock), Mod2 (Num Lock) and Mod5 (AltGr) are not in it. Release is 1<<30.

### The boundary uses surrounding text first, then the last character

The picker keeps two facts: the surrounding text from the app, and the last character it saw.

- `SetSurroundingText(text, cursor, anchor)` stores the character before `min(cursor, anchor)`, or "start of text" when that index is 0.
- A key the picker passes through, or text it commits, sets the last character and marks the stored surrounding text stale. The app sends fresh surrounding text after it applies the key, and that clears the stale mark.
- At a trigger character: fresh surrounding text decides. Otherwise the last character decides. A key with no character, such as Left or Ctrl+v, makes the last character unknown. Unknown counts as a boundary.
- Whitespace is `char::is_whitespace`.

Alternative considered: trust surrounding text always. It can lag one key behind the key events, so `a::` typed fast could open a search.

### Committed text and passed keys keep their order

A character key that closes held text never passes through. The engine commits the held text and the character in one `CommitText` and consumes the key. The app then gets one string in the right order, whatever path the compositor uses. In GNOME Shell a committed string goes through text-input-v3, while a passed key goes through `wl_keyboard`. The harness cannot measure the order between those two paths, so this design does not depend on it.

A key without a character cannot be committed as text: Ctrl+a, Left, Enter, a dead key. For these the engine commits the held text and returns false. The daemon handles this in two ways:

- A client in sync mode sets `EffectivePostProcessKeyEvent`. The daemon then queues signals that arrive while a key is in flight and hands them to the client with the reply (`bus/inputcontext.c`, `bus_input_context_make_post_process_key_event`). The order then holds by construction.
- An async client, such as GNOME Shell, gets the `CommitText` signal and the `ProcessKeyEvent` reply as two messages on one connection.

Task 4.1 measures the async case in the harness first. The harness calls `process_key_event_async` and records the arrival of each signal and of the reply. The result picks one of two branches, both decided here:

1. The commit arrives before the reply in every run: keep commit and return false.
2. It does not: commit the held text, return true, and emit `ForwardKeyEvent` with the same keyval, keycode and state.

The measurement and its result go into `docs/ibus-protocol.md`.

### Held text is never lost

Every preedit update that shows held text uses preedit mode COMMIT, which is 1 (`IBUS_ENGINE_PREEDIT_COMMIT` in `src/ibustypes.h`). The daemon then commits the visible preedit itself on focus-out, reset, disable and engine switch (`bus_input_context_clear_preedit_text`, called from `bus_input_context_focus_out`, `_ic_reset`, `bus_input_context_disable` and `bus_input_context_unset_engine`). The engine only drops its state on those calls and commits nothing, so the text arrives once.

When the engine commits by itself, it first sends `UpdatePreeditText("", 0, false, CLEAR)` and then `CommitText`. The daemon then holds no COMMIT preedit that it could commit a second time.

### Search over gemoji shortcodes

- `crates/torpedo-core/data/gemoji/emoji.json` comes from github/gemoji at commit `fadaeaf1f1a9be82b321316a6c5502e43138b2f6` (2026-09-28). `LICENSE` and a `README.md` naming the commit sit beside it. The data has 1870 emoji with 1913 aliases, all in `[a-z0-9_+-]`.
- The core embeds the file with `include_str!` and parses it once with `serde_json` at start-up. The binary needs no data file at run time, which keeps install to one binary.
- The query is lowercased. Each shortcode gets a rank: 0 exact, 1 prefix, 2 a `_`-separated word after the first starts with the query, 3 substring. An emoji takes its best rank, and among equal ranks the shortcode that comes first in its `aliases`. The list sorts by rank and then by the emoji's position in the file. Examples from the data: `fire` gives 🔥, then firefighter, fire_engine, fireworks, firecracker, fire_extinguisher, then heart_on_fire and two more. `fi` gives 56 matches.
- The list holds every match. The largest list, for one common letter, is under 2000 small texts per update.

Alternative considered: a fuzzy matcher. The ranking in the brief is predictable and easy to test.

### The candidate list

- The engine sends an `IBusLookupTable` with page size 9, no labels, `round` false, cursor visible, and vertical orientation (1). Each candidate is the text `<emoji> <shortcode>`, such as `🔥 fire`. Vertical fits the shortcodes.
- The engine keeps the cursor. Up and Down move it by one and Page Up and Page Down by 9, clamped to the list. Each move sends the table again.
- `CandidateClicked(index, button, state)` gives an index within the shown page, as `ibusenginesimple.c` reads it. The engine adds the page start: `cursor - cursor % 9`.
- The engine also implements the `CursorUp`, `CursorDown`, `PageUp` and `PageDown` methods that a panel can call, with the same moves.

### The engine crate

- The spike's code becomes modules: `ibus_types` for the GVariant shapes, `engine` for the Engine object, `factory`, and `address`.
- Each `CreateEngine` exports a new object path with its own `Picker` and a `Service` object for `Destroy`.
- Properties: `FocusId = false`, `ActiveSurroundingText = true`, and `ContentType` stored but unused.
- The main loop reads the connection's message stream. When the stream ends, the process exits with status 0.

### The engine connects only with --ibus

Without `--ibus` the binary prints its name and version and exits. The real engine reads the default address file, which on the owner's desktop points at the live daemon. A stray `cargo run` would otherwise register on the live daemon.

### Address lookup follows libibus

The order is `IBUS_ADDRESS`, then the file named by `IBUS_ADDRESS_FILE`, then `$XDG_CONFIG_HOME/ibus/bus/<machine-id>-<host>-<display>` (`ibus_get_socket_path` in `src/ibusshare.c`). With `WAYLAND_DISPLAY` set, host is `unix` and display is its value. Otherwise `DISPLAY` splits into host and display number, and an empty host becomes `unix`. The machine id comes from `/var/lib/dbus/machine-id`, else `/etc/machine-id`. The file's `IBUS_ADDRESS=` line gives the address. When an `IBUS_DAEMON_PID=` line names a dead process, the file does not count. The lookup is a pure function over an environment map and a file reader, so unit tests cover each branch.

### The e2e harness

- `e2e/harness/driver.py` is the spike's `harness.py` turned into a JSON-lines server. Node writes one command per line to its stdin and reads one reply per line from its stdout. Commands: create an input context, focus in and out, reset, set surrounding text, send a key, click a candidate, switch the global engine, start the engine by hand with a given environment, and stop.
- `e2e/harness/session.mjs` starts `dbus-run-session --config-file e2e/harness/session.conf -- python3 -I e2e/harness/driver.py`. It makes a temp dir under `/tmp` with a short name, because a socket path has a limit of 107 bytes. It sets `IBUS_ADDRESS_FILE`, `XDG_CONFIG_HOME`, `XDG_CACHE_HOME`, `XDG_DATA_HOME`, `GSETTINGS_BACKEND=memory` and `GIO_USE_VFS=local`, and removes `IBUS_ADDRESS`.
- The driver starts `/usr/bin/ibus-daemon` with `--cache=none --panel=disable --emoji-extension=disable --config=disable --single --verbose` and a private `IBUS_COMPONENT_PATH`. The component folder holds `simple.xml` from the system and a `torpedo.xml` that points at `target/release/torpedo-engine`.
- The driver stops the daemon with SIGTERM and waits for it to exit when it reads "stop", at end of stdin, and on SIGTERM, SIGINT or SIGHUP. Node killing the test therefore closes stdin, and the daemon still stops before the D-Bus session ends.
- One daemon serves one test file. Each test makes a new input context and focuses it, so the engine starts each test in Idle.
- A key command returns the verdict and the events in arrival order: commits, preedit updates, list updates, forwarded keys and the reply. The Node side folds these into the app's text: a commit appends its text, and a reply of false appends the key's character, each at its place in arrival order. A test asserts on that text, so a wrong order fails the test.
- After each test file, a guard compares a listing of `~/.config/ibus/bus/` with the one taken before. It fails the run when the folder changed.

The daemon-connection tests start the engine by hand through the driver, with an environment the test gives. For the default address file the test removes `IBUS_ADDRESS` and `IBUS_ADDRESS_FILE` from the engine's environment only, sets `XDG_CONFIG_HOME` to the private folder and `WAYLAND_DISPLAY` to `torpedo-e2e`, and writes the address file under that name. The daemon itself always runs with `IBUS_ADDRESS_FILE` set.

### Install and uninstall scripts

- Both are POSIX `sh` with `set -eu`. Overrides: `PREFIX` (default `$HOME/.local`), `COMPONENT_DIR` (default `/usr/share/ibus/component`) and `SUDO` (default `sudo`, and an empty value means no sudo). `${SUDO-sudo}` keeps an empty value empty.
- `install.sh` runs `cargo build --release -p torpedo-engine`, then `install -D -m 755` to `$PREFIX/lib/torpedo/ibus-engine-torpedo`. It writes the component file to a temp file and runs `$SUDO install -D -m 644` into `$COMPONENT_DIR/torpedo.xml`.
- `--dry-run` prints each step and writes nothing. The test of the defaults uses it, so no test ever writes to `/usr`.
- The e2e test helper always sets `PREFIX`, `COMPONENT_DIR` and `SUDO`, and puts recording stubs for `ibus`, `gsettings`, `dconf` and `sudo` first on the PATH. It refuses to run the script when any of the three variables is unset or points outside the test's temp folder, except in `--dry-run`.

### The e2e runner builds the engine

`scripts/e2e.mjs` runs `cargo build --release -p torpedo-engine` before the tests, under the lock it already holds. The harness and the install test both use the release binary, so `install.sh` in a test finds a finished build.

## Risks / Trade-offs

- [The order between a commit and a passed key in GNOME Shell is not measured] → Character keys never depend on it. For keys without a character the harness measures the daemon side, and the owner checks the Shell side after install.
- [A passed dead key composes only if the app or compositor composes it] → The `dk` layout under the `xkb:dk::dan` engine composes in the engine. Under torpedo the compose step moves to the app. The owner checks `´` + `e` in GTK, Firefox and kitty after install. If it fails, a later change adds compose to torpedo.
- [Surrounding text can lag] → A key the engine passes marks it stale until the app sends fresh text.
- [Kitty may send no surrounding text] → The last character since focus-in decides. After a cursor move by mouse the engine knows nothing and counts a boundary.
- [The layout `dk` is fixed in the component file] → It matches the owner's desktop. A later change can make it an install option.
- [A trigger right after an emoji does not count, because an emoji is not whitespace] → This follows the brief. See Open Questions.

## Migration Plan

1. The owner runs `./install.sh`. It asks for the sudo password once.
2. The owner runs `ibus restart` and adds Torpedo in GNOME Settings, keeping Danish as a fallback.
3. To roll back, the owner switches to Danish, removes Torpedo from the input sources, runs `./uninstall.sh` and `ibus restart`.

## Open Questions

- Should the trigger also count right after an emoji, so that `::fire:::rocket:` gives 🔥🚀? Today it does not. The answer changes one rule in the boundary check and one scenario.
