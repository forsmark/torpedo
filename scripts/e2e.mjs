#!/usr/bin/env node
/**
 * Run the scenario tests: the `node --test` files under e2e/.
 *
 *   node scripts/e2e.mjs
 *
 * The tests drive the real engine binary through a private IBus daemon, so
 * the run first builds the release binaries of the engine and the settings
 * window, which the harness and the settings tests use.
 * While e2e/ holds no test file, the run prints a note and passes. The heavy
 * lock guards the run. A run inside verify already holds it and does not wait.
 */
import { spawnSync } from 'node:child_process';
import { globSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { withCargoOnPath } from './cargo-path.mjs';
import { holdHeavyLock } from './heavy-lock.mjs';

/** The folder of the scenario tests, from the repository root. */
export const E2E_DIR = 'e2e';

/** The command that builds the engine and settings binaries the tests drive. */
export const BUILD = ['cargo', ['build', '--release', '-p', 'torpedo-engine', '-p', 'torpedo-settings']];

/** The scenario test files under `root`, as sorted paths from `root`. */
export function e2eTestFiles(root) {
  return globSync(`${E2E_DIR}/**/*.test.mjs`, { cwd: root }).sort();
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  holdHeavyLock();
  const root = join(import.meta.dirname, '..');
  const files = e2eTestFiles(root);
  if (files.length === 0) {
    console.log(`e2e: no test files yet under ${E2E_DIR}/`);
    process.exit(0);
  }
  const env = withCargoOnPath();
  const build = spawnSync(BUILD[0], BUILD[1], { cwd: root, stdio: 'inherit', env });
  if (build.status !== 0) {
    console.error('e2e: the build failed');
    process.exit(build.status ?? 1);
  }
  const result = spawnSync(process.execPath, ['--test', '--test-concurrency=1', ...files], {
    cwd: root,
    stdio: 'inherit',
    env,
  });
  process.exit(result.status ?? 1);
}
