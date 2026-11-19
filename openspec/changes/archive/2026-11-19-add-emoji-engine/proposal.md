# Proposal

## Why

Torpedo has a Rust workspace but no engine. The owner wants to type `::fire` in any text field and get 🔥, like Rocket on macOS. A spike proved that a Rust engine can speak the IBus protocol with zbus. This change builds the first usable engine on that result, with the emoji search, the tests that drive it through a private IBus daemon, and a way to install it.

## What Changes

- Add the inline picker to `torpedo-core`, as pure logic with unit tests:
  - a state machine for the trigger, the held text and the search,
  - the boundary rule that decides where a trigger counts: the start of the text, or after whitespace, an emoji, an opening bracket or an opening quote,
  - dead key composing, as the Danish xkb engine does today, so that `idé` still types under torpedo,
  - the search and ranking over gemoji shortcodes.
  The trigger is a parameter with the default `::`. A later change passes in a value from the config file.
- Vendor gemoji's `db/emoji.json` at a pinned commit, with its MIT license and a README that names the commit.
- Turn `torpedo-engine` into a real IBus engine with zbus 5, tokio and futures-util. It finds the daemon by `IBUS_ADDRESS`, then `IBUS_ADDRESS_FILE`, then the default address file. It connects only when started with `--ibus`, and it exits when the daemon goes away.
- Add `install.sh` and `uninstall.sh`. Install builds the release binary, copies it to `~/.local/lib/torpedo/`, and copies a component file to `/usr/share/ibus/component/` with sudo. Neither script restarts IBus or changes input sources. Both print the owner's next steps.
- Add the e2e harness under `e2e/harness/`, moved from the spike. It runs a private ibus-daemon inside `dbus-run-session` and drives it with a Python client. Add one `node --test` test for each scenario, and map each in `e2e/coverage/`.
- `scripts/e2e.mjs` builds the engine before it runs the tests.
- Add `docs/ibus-protocol.md` with the durable findings of the spike.
- Add the new domain terms to `GLOSSARY.md`.

Out of scope: aliases, the config file and the settings window come in a later change. Recent emoji and skin tones come in another later change.

## Capabilities

### New Capabilities

- `inline-picker`: what the user sees and gets while typing. Which keys pass through, how dead keys compose, where the trigger counts, the search with its preedit and candidate list, the keys during a search, and that held text is never lost.
- `daemon-connection`: how the engine finds the IBus daemon, when it connects, and when it exits.
- `installation`: what `install.sh` and `uninstall.sh` do and what they never touch.

### Modified Capabilities

None. `openspec/specs/` holds no spec yet.

## Impact

- New code: `crates/torpedo-core/src/` modules for the trigger, the boundary and the search. `crates/torpedo-engine/src/` modules for the D-Bus objects and the address lookup.
- New data: `crates/torpedo-core/data/gemoji/` with `emoji.json`, `LICENSE` and `README.md`.
- New dependencies: `serde`, `serde_json` and `unicode-normalization` in `torpedo-core`. `zbus` 5 (no default features, with `tokio`), `tokio` and `futures-util` in `torpedo-engine`.
- New files: `install.sh`, `uninstall.sh`, `e2e/harness/`, `e2e/*.test.mjs`, `e2e/coverage/*.json`, `docs/ibus-protocol.md`.
- Changed files: `scripts/e2e.mjs` and its test, `GLOSSARY.md`, `AGENTS.md` (install commands and the harness).
- The e2e tests need `/usr/bin/ibus-daemon`, `dbus-run-session` and Python 3 with the IBus GObject bindings. The owner's machine has them.
- The owner runs `install.sh` once by hand, with sudo. No test runs it against the real system.

## Claims checked

`spec-checks claims add-emoji-engine` found no claim in another spec, because `openspec/specs/` holds none yet. It listed seven claims of this change that name a glossary term. I read each against the rest of the change:

- inline-picker › A dead key is held: the pass-through of an unknown dead key agrees with *Keys the engine does not use pass through*. A known dead key is a key the engine uses.
- inline-picker › A dead key that does not compose: agrees with *A dead key alone gives its accent*, which covers space and the same dead key, and with *A dead key before another key*, which covers keys without a character.
- inline-picker › The engine asks every app for surrounding text: agrees with the boundary rules, which use surrounding text first.
- inline-picker › Without surrounding text the last typed character decides: agrees with *The trigger counts only at a boundary*, which applies when surrounding text exists. Committed emoji count as the last character, which gives *Two emoji in a row*.
- inline-picker › Enter and Tab without a candidate list: agrees with *Enter and Tab commit the selected emoji*, which applies only while a candidate list is shown.
- inline-picker › A query with no match ends the search: agrees with *Query characters* and *Another character closes the search*.
- inline-picker › Held text is never lost: agrees with every rule that commits held text, because each one leaves no held text behind. A held dead key is held text too.
