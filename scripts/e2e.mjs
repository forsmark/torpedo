#!/usr/bin/env node
/**
 * Run the scenario tests: the `node --test` files under e2e/.
 *
 *   node scripts/e2e.mjs
 *
 * The tests drive the real engine binary through a private IBus daemon. While
 * e2e/ holds no test file, the run prints a note and passes. The heavy lock
 * guards the run. A run inside verify already holds it and does not wait.
 */
import { spawnSync } from 'node:child_process';
import { globSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { withCargoOnPath } from './cargo-path.mjs';
import { holdHeavyLock } from './heavy-lock.mjs';

/** The folder of the scenario tests, from the repository root. */
export const E2E_DIR = 'e2e';

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
  const result = spawnSync(process.execPath, ['--test', ...files], {
    cwd: root,
    stdio: 'inherit',
    env: withCargoOnPath(),
  });
  process.exit(result.status ?? 1);
}
