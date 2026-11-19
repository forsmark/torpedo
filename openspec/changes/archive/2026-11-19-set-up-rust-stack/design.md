# Design

## Context

The repository holds the verify scripts, the heavy lock and spec-checks, all taken from a pnpm and turbo template. `scripts/verify.mjs` has empty lint and typecheck steps, and its test commands call `pnpm turbo`. `scripts/verify/affected.mjs` expects packages under `apps/` and `packages/`. `openspec/checks.json` lists no coverage adapter. Rust 1.100.0 is installed per user with rustup, in `~/.cargo/bin`. That folder is not on the PATH of every shell an agent starts. See proposal.md for the motivation.

## Goals / Non-Goals

**Goals:**

- `node scripts/verify.mjs` and `node scripts/verify.mjs --affected` run the Rust checks and both kinds of tests, and pass on the skeleton.
- `node --test "scripts/**/*.test.mjs"` stays green.
- `spec-checks verify` reads scenario tests from `e2e/`.

**Non-Goals:**

- Engine code, the private IBus harness and the first e2e test. The engine change adds them.
- The settings window crate with GTK4 and libadwaita.
- Per-crate test selection in `verify --affected`.

## Decisions

### A Cargo workspace with two crates

The root `Cargo.toml` is a virtual workspace with `members = ["crates/*"]` and resolver 3. Shared fields such as the edition (2024) and `publish = false` live in `[workspace.package]`. `torpedo-core` is a library. `torpedo-engine` is a binary that depends on `torpedo-core` by path. The core holds logic that tests can reach without D-Bus or GTK. Each crate has one trivial `#[test]`, so `cargo test` runs a test in each.

The skeleton adds no dependency it does not use. `zbus` 5 (with `default-features = false` and the `tokio` feature), `tokio` and `futures-util` come with the engine change, when its code uses them. An unused dependency would only slow the build and need updates.

### Lint, typecheck and test commands in CONFIG

- Checks, in order: `cargo fmt --all --check`, then `cargo clippy --workspace --all-targets -- -D warnings`, then `cargo check --workspace --all-targets`.
- Tests: `cargo test --workspace`, then `node scripts/e2e.mjs`.

`testAll` and `testFiltered` each become a list of commands, and verify runs them in order and stops at the first failure. The template allowed one command each. The change to `verify.mjs` is this loop and the CONFIG values.

### Affected runs test the whole workspace

`testFiltered` receives turbo filters such as `...[<commit>]`. Cargo has no filter for "changed since a commit, and dependents". So an affected run with any changed crate runs the same tests as a full run. With two crates and incremental builds, the saving from a finer filter is small. The scope logic in `affected.mjs` still decides whether tests run at all. A change to only docs or scripts runs no cargo tests.

Alternative considered: map the changed crates to `cargo test -p`. It needs a crate dependency graph in the script and saves little today. A later change can add it when the workspace grows.

### Source layout in affected.mjs

- `PACKAGE` matches `crates/<name>/`.
- `NO_TESTS` drops `e2e/` and adds `e2e/coverage/`. The e2e tests drive the engine binary, so a change to them must run them. A change under `e2e/` outside `e2e/coverage/` then counts as a root file and runs everything. The coverage map holds no test, so it runs nothing.
- Root files such as `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `package.json` and `pnpm-lock.yaml` run everything. The template already treats an unknown root file this way.
- `affected.test.mjs` uses a sample workspace under `apps/` and `packages/`. Its samples move to `crates/`, and the e2e test changes to the new rule.
- The guard on reads across packages reads only JavaScript and TypeScript files. It does not see Rust code that reads another crate's files. That gap does no harm while every affected run tests the whole workspace.

### An e2e runner that passes with no tests

`scripts/e2e.mjs` finds the files `e2e/**/*.test.mjs`. With none, it prints "e2e: no test files yet" and exits 0. With some, it runs `node --test` on them. It takes the heavy lock first, as `heavy-lock.mjs` asks of heavy scripts. A run inside verify already holds the lock and does not wait. The finding of files is a function, and `scripts/e2e.test.mjs` tests it. The engine change extends this runner to build the engine binary first.

### Cargo on the PATH

`verify.mjs` and `e2e.mjs` add `$CARGO_HOME/bin`, or `~/.cargo/bin`, to the PATH of the commands they start, when that folder exists. A shell that never ran `source ~/.cargo/env` then still finds `cargo`. `AGENTS.md` tells agents to run `source ~/.cargo/env` before they call cargo by hand.

### Coverage through the node-test adapter

`openspec/checks.json` sets `coverage.adapters` to `[{ "kind": "node-test", "roots": ["e2e"] }]`. spec-checks treats `node-test` as a unit adapter. So a scenario's entry in the coverage map has the form `{ "unit": "e2e/<file> › <test name>" }`, and the file must sit under `e2e/`. The coverage check parses test files with the TypeScript from the project. A root `package.json`, private and with `"type": "module"`, gives it `typescript` as a dev dependency. `packageManager` pins pnpm 12, and `pnpm-lock.yaml` is committed.

### Safety rules in AGENTS.md

The owner's desktop types through IBus. A test or command that touches the live IBus session can stop all typing. `AGENTS.md` gets a section "Safety: never touch the live IBus session" with the rules in tasks.md. The engine change follows them when it adds the harness.

## Risks / Trade-offs

- [Affected runs test more than they must] → The workspace is small. A later change can add per-crate selection.
- [No scenario exists yet, so nothing shows the node-test adapter and the TypeScript loading at work] → The first change with a scenario maps it, and `spec-checks verify` then reads an e2e file.
- [`pnpm-lock.yaml` and `package.json` are root files, so a change to them runs every test] → This is the template's rule, and it is safe.
- [The cargo PATH fallback hides a missing toolchain behind a later error] → When cargo is missing, the spawn fails with "cargo: not found" and verify exits non-zero.
