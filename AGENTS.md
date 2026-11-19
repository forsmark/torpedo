# torpedo

Torpedo is an inline emoji picker for Linux, like Rocket on macOS. You type a trigger such as "::" and a Slack-style name in any text field, and torpedo shows matching emoji at the cursor and inserts the one you pick. It is for the owner's GNOME Wayland desktop.

Stack: Rust (stable, via rustup) and Cargo. IBus input method engine over D-Bus with zbus. GTK4 with gtk4-rs for the settings window. Emoji data from gemoji, settings in TOML. Tests with cargo test.

## Commands

- `node scripts/verify.mjs --affected` runs the checks and the tests that the changes since origin/main can reach. Run it once, at the end of your work, on a clean tree.
- `node scripts/verify.mjs` runs everything. It runs before a release.
- `node tools/spec-checks/bin/spec-checks verify` runs the spec checks. Run it before you archive a change, and after a merge that touched `openspec/`.
- Heavy jobs wait for one machine-wide lock. Start other heavy commands with `node scripts/heavy-lock.mjs <command>`.

The first OpenSpec change sets up the stack. It adds the lint, typecheck and test commands to CONFIG in `scripts/verify.mjs`, the test runners to `coverage.adapters` in `openspec/checks.json`, and the commands to this list.

## A change to behavior goes through OpenSpec

The specs in `openspec/specs/` say what the system does. A change to what a spec says, or new behavior no spec describes, goes through an OpenSpec change: propose, apply, archive. Load the skill `spec-checks:practice` before each step.

- The owner answers both gates: the proposal gate after propose, and the sync question before archive.
- The base branch is `main`. One change goes in one pull request, from proposal to archive.
- A fix that makes the code do what a spec already says needs no change.

## Words

`GLOSSARY.md` holds the domain words, and they are binding. Use its term, never a synonym. A change that brings a new concept adds its term to the glossary in the same change.

Write specs, docs and commit messages in plain English: one idea per sentence, active voice, one name per thing.
