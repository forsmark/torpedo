# Tasks

## 1. Data and docs

- [ ] 1.1 Vendor gemoji: copy `db/emoji.json` and `LICENSE` from github/gemoji at commit `fadaeaf1f1a9be82b321316a6c5502e43138b2f6` into `crates/torpedo-core/data/gemoji/`, and add a `README.md` there that names the source, the commit, its date and the MIT license. Verify that the file parses with `python3 -I -c "import json; json.load(open(...))"` and holds 1870 entries.
- [ ] 1.2 Add `docs/ibus-protocol.md` with the durable parts of the spike's FINDINGS.md: the component path and cache, the address lookup, the GVariant shapes and the double-wrap pitfall, the D-Bus surface, the harness facts, the commands and the safety traps. Leave out spike-only details such as the env switches. Verify that every section of FINDINGS.md that this design relies on has a counterpart.

## 2. The engine on the wire

- [ ] 2.1 Add `zbus` 5 (no default features, `tokio`), `tokio` (`rt`, `macros`, `time`, `sync`) and `futures-util` to `torpedo-engine`. Move the spike's GVariant helpers into `ibus_types` with the spike's shape test turned into assertions on the signatures. Verify with `cargo test -p torpedo-engine`.
- [ ] 2.2 Add the `address` module: `IBUS_ADDRESS`, then `IBUS_ADDRESS_FILE`, then the default path from `XDG_CONFIG_HOME`, the machine id, and `WAYLAND_DISPLAY` or `DISPLAY`, and the `IBUS_DAEMON_PID` liveness check. It reads from an environment map and a file reader passed in. Verify with unit tests for each branch, including an X11 `DISPLAY` of `:0` and of `host:1.0`.
- [ ] 2.3 Write `main`: without `--ibus` print the name and version and exit 0. With it, look up the address, serve the factory, request `org.freedesktop.IBus.Torpedo`, and exit 0 when the message stream ends. Port the factory, the Engine object with its properties (`FocusId` false, `ActiveSurroundingText` true) and `Service.Destroy` from the spike, with a stub picker that passes every key. Verify with `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] 2.4 Move the spike harness into `e2e/harness/`: `session.conf`, `driver.py` as a JSON-lines server, and `session.mjs` for Node, as design.md describes. The driver stops the daemon with SIGTERM and waits on "stop", end of stdin, SIGTERM, SIGINT and SIGHUP. Add the guard on `~/.config/ibus/bus/`. Verify with a smoke test that starts a session, selects `torpedo` and stops, and then check that `pgrep -af ibus-daemon` shows only the owner's daemon.
- [ ] 2.5 Make `scripts/e2e.mjs` run `cargo build --release -p torpedo-engine` before the tests, and extend `scripts/e2e.test.mjs` for the build command. Verify with `node --test "scripts/**/*.test.mjs"` and `node scripts/e2e.mjs`.
- [ ] 2.6 Add `e2e/daemon-connection.test.mjs` with one test for each scenario of `daemon-connection`, and `e2e/coverage/daemon-connection.json` mapping each one. Verify with `node scripts/e2e.mjs` and `node tools/spec-checks/bin/spec-checks coverage`.

## 3. The picker in torpedo-core

- [ ] 3.1 Add `serde` and `serde_json` to `torpedo-core`. Add the `emoji` module: embed `emoji.json` with `include_str!`, parse it once, and rank matches as design.md says. Verify with unit tests for `fire`, `ire`, `thumbs`, `fist`, `+1`, `1st_place`, `FIRE` and `fireq`.
- [ ] 3.2 Add the `keys` module: keyval to character, dead keys, modifier keys, keypad keys, and the pass-through modifier mask. Verify with unit tests for `:` with Shift, `{` with Mod5, `æ`, `KP_1`, `dead_acute`, `Shift_L`, `ISO_Level3_Shift`, and Ctrl, Alt and Super states.
- [ ] 3.3 Add the `boundary` module: surrounding text with cursor and anchor, the stale mark, and the last character. Verify with unit tests for start of text, after a space, after a letter, a selection, stale text, and an unknown last character.
- [ ] 3.4 Add the `picker` module: the states and the key table of design.md, with the trigger as a parameter. Verify with one unit test per cell of the table, and with a trigger of one character and of three characters.

## 4. The picker in the engine

- [ ] 4.1 Measure the wire order: make the engine commit the held text and return false for a key without a character, and add `e2e/harness/order.test.mjs`, which sends Ctrl+`a` after `:` 200 times with `process_key_event_async` and records whether `commit-text` arrives before the reply. Record the result in `docs/ibus-protocol.md`, and pick the branch in design.md "Committed text and passed keys keep their order". Verify that the test passes for the chosen branch.
- [ ] 4.2 Wire the picker into the Engine object: key events, focus-in, focus-out, reset, disable, surrounding text, candidate clicks, and the panel's cursor and page calls. Send the preedit with mode COMMIT while text is held, and clear it with mode CLEAR before the engine commits. Verify with `cargo test --workspace`.
- [ ] 4.3 Add the e2e tests for `inline-picker`, one test for each scenario, grouped by requirement in files under `e2e/inline-picker/`. Add `e2e/coverage/inline-picker.json` mapping each one. Verify with `node scripts/e2e.mjs` and `node tools/spec-checks/bin/spec-checks coverage`. Then check that `pgrep -af ibus-daemon` shows only PID 7674 and that `~/.config/ibus/bus/` is unchanged.

## 5. Installation

- [ ] 5.1 Add `install.sh` and `uninstall.sh` as design.md describes, with `PREFIX`, `COMPONENT_DIR`, `SUDO` and `--dry-run`. Verify with `sh -n` on both and with `./install.sh --dry-run`, which writes nothing.
- [ ] 5.2 Add `e2e/installation.test.mjs` with one test for each scenario of `installation`, using a temp folder, recording stubs and the guard on the three variables. Add `e2e/coverage/installation.json`. Verify with `node scripts/e2e.mjs` and `node tools/spec-checks/bin/spec-checks coverage`.
- [ ] 5.3 Add the install and uninstall commands and the harness to `AGENTS.md`, and say that only the owner runs `install.sh` against the real system. Verify that each listed command runs as written in `--dry-run` or against a temp folder.

## 6. Integration

- [ ] 6.1 On a clean tree, run `node scripts/verify.mjs`, `node --test "scripts/**/*.test.mjs"` and `node tools/spec-checks/bin/spec-checks verify`. Verify that all three pass, that `pgrep -af ibus-daemon` shows only the owner's daemon, and that `~/.config/ibus/bus/` is unchanged.

## Workflow follow-up

- Stop at the sync gate and send the reviewer the `spec-checks gate add-emoji-engine` printout.
- Archive the change after the reviewer answers. Then run `node tools/spec-checks/bin/spec-checks verify`, `node scripts/verify.mjs` and `node --test "scripts/**/*.test.mjs"`, push, and mark the PR ready.
