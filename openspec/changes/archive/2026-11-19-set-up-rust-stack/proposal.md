# Proposal

## Why

The repository has no build yet. The lint, typecheck and test steps in `scripts/verify.mjs` are empty, and spec-checks has no test runner to read. Every later change needs a Rust workspace to put code in, commands that check it, and a place for scenario tests. This change sets that up before any engine code is written.

## What Changes

- Add `rust-toolchain.toml`. It pins the `stable` channel with the `rustfmt` and `clippy` components.
- Add a Cargo workspace at the repository root with two crates:
  - `crates/torpedo-core`, a library for the pure logic: trigger detection, search and config. It uses no D-Bus and no GTK.
  - `crates/torpedo-engine`, the binary of the IBus engine.
- Each crate gets a minimal skeleton and one trivial `#[test]`, so that every command runs. The skeleton adds no dependency it does not use. The engine change adds `zbus` 5, `tokio` and `futures-util` when its code needs them.
- Fill CONFIG in `scripts/verify.mjs`:
  - Lint runs `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`.
  - Typecheck runs `cargo check --workspace --all-targets`.
  - Tests run `cargo test --workspace` and then the `node --test` files under `e2e/`.
- Add `scripts/e2e.mjs`. It runs the `node --test` files under `e2e/` and passes with a note while there are none.
- Fill the source layout in `scripts/verify/affected.mjs` for `crates/`, and update its tests to match.
- Set `coverage.adapters` in `openspec/checks.json` to a `node-test` adapter with the root `e2e`.
- Add a root `package.json`, private and with `"type": "module"`. It has `typescript` as a dev dependency, because the coverage check loads TypeScript from the project. Commit `pnpm-lock.yaml`, made with pnpm 12.
- Add `target/` to `.gitignore`.
- List the new commands in `AGENTS.md`. Add a section "Safety: never touch the live IBus session".
- The change writes no engine code. The settings window crate comes in a later change.

## Capabilities

### New Capabilities

None. This change adds tooling that no capability covers.

### Modified Capabilities

None.

The change sets `skip_specs: true` in its `.openspec.yaml`. With that marker, `openspec validate --strict` accepts a change with no delta specs, and `openspec archive` archives it. No spec changes, so `openspec/specs/` stays empty after the archive.

## Impact

- New files: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `crates/torpedo-core/`, `crates/torpedo-engine/`, `package.json`, `pnpm-lock.yaml`, `scripts/e2e.mjs` and its test.
- Changed files: `scripts/verify.mjs`, `scripts/verify/affected.mjs`, `scripts/verify/affected.test.mjs`, `openspec/checks.json`, `.gitignore`, `AGENTS.md`, `openspec/config.yaml`.
- `node scripts/verify.mjs` now needs the Rust toolchain from rustup and Node with pnpm.
- Scenario tests are `node --test` files under `e2e/`. They drive the real engine binary through a private IBus daemon. The engine change adds that harness. Rust `#[test]`s cover internals and map to no scenario.

## Claims checked

- None. `spec-checks claims set-up-rust-stack` has nothing to check. The change has no delta specs, and `openspec/specs/` holds no spec. The command finds a change by its `specs/` folder, so it stops with "claims: no change named set-up-rust-stack under openspec/changes".
