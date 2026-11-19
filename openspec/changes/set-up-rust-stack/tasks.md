# Tasks

## 1. Rust workspace

- [x] 1.1 Add `rust-toolchain.toml` with channel `stable` and the components `rustfmt` and `clippy`. Verify that `rustup show` in the repository names the stable toolchain from that file.
- [x] 1.2 Add the root `Cargo.toml` as a virtual workspace with `members = ["crates/*"]`, resolver 3, and edition 2024 and `publish = false` in `[workspace.package]`. Verify that `cargo metadata --no-deps` lists both crates once they exist.
- [x] 1.3 Add `crates/torpedo-core`, a library with a minimal skeleton and one trivial `#[test]`. It has no D-Bus or GTK dependency. Verify that `cargo test -p torpedo-core` runs one test and passes.
- [x] 1.4 Add `crates/torpedo-engine`, a binary that depends on `torpedo-core` by path, with a minimal `main` and one trivial `#[test]`. Add no other dependency. Verify that `cargo test -p torpedo-engine` runs one test and passes.
- [x] 1.5 Add `target/` to `.gitignore` and commit `Cargo.lock`. Verify that `git status` shows no file under `target/`.
- [x] 1.6 Verify that `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo check --workspace --all-targets` pass on the skeleton.

## 2. Node project for spec-checks

- [x] 2.1 Add a root `package.json`, private, with `"type": "module"`, `packageManager` set to the installed pnpm 12, and `typescript` as a dev dependency. Run `pnpm install` and commit `pnpm-lock.yaml`. Verify that `node -e "import('typescript')"` resolves from the root.
- [x] 2.2 Set `coverage.adapters` in `openspec/checks.json` to `[{ "kind": "node-test", "roots": ["e2e"] }]`. Verify that `node tools/spec-checks/bin/spec-checks verify` passes.

## 3. Verify scripts

- [ ] 3.1 Add `scripts/e2e.mjs`. It takes the heavy lock, adds the cargo bin folder to the PATH, finds `e2e/**/*.test.mjs`, prints "e2e: no test files yet" and exits 0 when there are none, and otherwise runs `node --test` on them. Add `scripts/e2e.test.mjs` for the finding of files, with and without test files. Verify that `node scripts/e2e.mjs` passes on the empty `e2e/` and that the new test passes.
- [ ] 3.2 Fill CONFIG in `scripts/verify.mjs`: the fmt and clippy lint steps, the `cargo check` typecheck step, and the tests `cargo test --workspace` then `node scripts/e2e.mjs`. Let `testAll` and `testFiltered` be lists of commands that run in order. Add the cargo bin folder to the PATH of the commands it starts. Verify that `node scripts/verify.mjs` passes in a shell without `~/.cargo/bin` on its PATH.
- [ ] 3.3 Fill the layout in `scripts/verify/affected.mjs`: `PACKAGE` matches `crates/<name>/`, and `NO_TESTS` holds `e2e/coverage/` in place of `e2e/`. Move the samples in `affected.test.mjs` to `crates/`, and change its e2e test so that an e2e test file runs everything and the coverage map runs nothing. Verify that `node --test "scripts/**/*.test.mjs"` passes.

## 4. Docs

- [ ] 4.1 List the new commands in `AGENTS.md`: `source ~/.cargo/env`, the cargo fmt, clippy, check and test commands, `node scripts/e2e.mjs`, and the heavy-lock wrapper for cargo build and test run by hand. Replace the note that the first change sets up the stack. Say that scenario tests are `node --test` files under `e2e/`, and that Rust `#[test]`s cover internals. Verify that each listed command runs as written.
- [ ] 4.2 Add the section "Safety: never touch the live IBus session" to `AGENTS.md`. It says why: the owner's desktop types through IBus, and a mistake stops all typing. Its rules:
  - Never change gsettings, such as `org.gnome.desktop.input-sources`.
  - Never run `ibus restart`, `ibus engine`, `ibus write-cache` or any other `ibus` command against the live daemon.
  - Never write to `~/.config/ibus`, `~/.cache/ibus`, `~/.config/environment.d`, `~/.config/systemd` or `/usr`.
  - Tests run only against a private ibus-daemon inside `dbus-run-session`, with `IBUS_ADDRESS_FILE`, `XDG_CONFIG_HOME`, `XDG_CACHE_HOME` and `XDG_DATA_HOME` pointing at a temp dir. Without `IBUS_ADDRESS_FILE`, a private daemon overwrites the live address file.
  - Stop each private ibus-daemon with SIGTERM and wait for it to exit before its `dbus-run-session` ends. Never leave one to die when its bus goes away. A crashed private daemon makes Ubuntu show the owner an "IBus Preferences has closed unexpectedly" crash dialog.
  - Installing onto the live desktop happens only when the owner asks.

  Verify that the section holds each rule.
- [ ] 4.3 Update the stack line in `AGENTS.md` and the context in `openspec/config.yaml` to name the e2e tests with `node --test` beside `cargo test`. Verify that `openspec validate --all --strict` still passes.

## 5. Integration

- [ ] 5.1 On a clean tree, run `node scripts/verify.mjs`, `node --test "scripts/**/*.test.mjs"` and `node tools/spec-checks/bin/spec-checks verify`. Verify that all three pass.

## Workflow follow-up

- Stop at the sync gate and send the reviewer the `spec-checks gate set-up-rust-stack` printout.
- Archive the change after the reviewer answers, then run the three commands of task 5.1 again.
