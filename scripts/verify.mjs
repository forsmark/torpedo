#!/usr/bin/env node
/**
 * Check and test the repository.
 *
 *   node scripts/verify.mjs                            everything
 *   node scripts/verify.mjs --affected [--base <ref>]  the same checks, and only
 *                                                      the tests that the changes
 *                                                      since <ref> can reach
 *
 * A subagent runs `--affected` once, at the end of its work. The full run
 * happens once per batch, before a release. The scope rules live in
 * verify/affected.mjs.
 *
 * A run that passes on a clean tree leaves a record, so a later merge step can
 * skip its own run. See verify/record.mjs.
 */
import { spawnSync } from 'node:child_process';
import { withCargoOnPath } from './cargo-path.mjs';
import { holdHeavyLock } from './heavy-lock.mjs';
import { PACKAGE, SCRIPTS_DIR, scopeOf } from './verify/affected.mjs';
import { recordOf, snapshot, writeRecord } from './verify/record.mjs';

/** The whole Rust workspace, then the scenario tests under e2e/. */
const TESTS = [
  ['cargo', 'test', '--workspace'],
  ['node', 'scripts/e2e.mjs'],
];

const CONFIG = {
  /** The ref that work merges into. `--base` overrides it. */
  base: 'origin/main',
  /**
   * The checks that run before the tests, in order, on the whole repository.
   * The first that fails stops the run.
   */
  checks: [
    ['cargo', 'fmt', '--all', '--check'],
    ['cargo', 'clippy', '--workspace', '--all-targets', '--', '-D', 'warnings'],
    ['cargo', 'check', '--workspace', '--all-targets'],
  ],
  /** Every test, as commands that run in order. The first that fails stops the run. */
  testAll: TESTS,
  /**
   * The tests of the packages that the filters select, as commands that run in
   * order. A filter is `...[<commit>]` for the crates changed since that commit
   * and their dependents, or a crate name. Cargo has no filter for "changed
   * since a commit", so an affected run tests the whole workspace. The
   * workspace is small, and incremental builds keep that cheap.
   */
  testFiltered: (_filters) => TESTS,
  /** The tests of SCRIPTS_DIR. The workspace's test runner never sees them. */
  testScripts: ['node', '--test', `${SCRIPTS_DIR}/**/*.test.mjs`],
};

holdHeavyLock();

// cargo lives in ~/.cargo/bin, which a fresh shell may lack on its PATH.
const env = withCargoOnPath();
const run = ([cmd, ...args]) => spawnSync(cmd, args, { stdio: 'inherit', env }).status ?? 1;
/** Run the commands in order, and stop at the first that fails. */
const runEach = (commands) => {
  for (const command of commands) {
    const status = run(command);
    if (status !== 0) return status;
  }
  return 0;
};

const args = process.argv.slice(2);
const affected = args.includes('--affected');
const base = args.includes('--base') ? args[args.indexOf('--base') + 1] : CONFIG.base;
// Taken under the lock: the tree can change while a run waits for it.
const before = snapshot(process.cwd(), base);

/**
 * The merge base with `base`, and the files changed since it: committed or
 * not, and untracked files too.
 */
function changes() {
  const lines = (cmd) =>
    spawnSync('git', cmd, { encoding: 'utf8' }).stdout.split('\n').filter(Boolean);
  const merged = spawnSync('git', ['merge-base', base, 'HEAD'], { encoding: 'utf8' });
  if (merged.status !== 0) {
    console.error(`verify: found no common commit with ${base}. Fetch it, or pass --base <ref>.`);
    process.exit(1);
  }
  const mergeBase = merged.stdout.trim();
  // --no-renames lists a moved file at both paths, so the package it left still runs its tests.
  const files = [
    ...lines(['diff', '--name-only', '--no-renames', mergeBase]),
    ...lines(['ls-files', '--others', '--exclude-standard']),
  ];
  return { mergeBase, files };
}

/** One line that says what an affected run tests. */
function describeScope(scope, files) {
  const changed = [...new Set(files.map((file) => PACKAGE.exec(file)?.[0]).filter(Boolean))];
  const packages =
    scope.packages === 'all'
      ? 'every package'
      : scope.packages === 'changed'
        ? `${changed.join(', ')} and their dependents`
        : 'no package';
  const extra = scope.extra.length ? `, plus ${scope.extra.join(', ')}` : '';
  return `verify: affected since ${base}: ${packages}${extra}. Script tests: ${scope.scripts ? 'yes' : 'no'}.`;
}

let scope = { packages: 'all', extra: [], scripts: SCRIPTS_DIR !== null };
let mergeBase;
if (affected) {
  const found = changes();
  mergeBase = found.mergeBase;
  scope = scopeOf(found.files);
  console.log(describeScope(scope, found.files));
}

let status = 0;
for (const check of CONFIG.checks) {
  status = run(check);
  if (status !== 0) break;
}

if (status === 0 && scope.packages !== 'none') {
  status = runEach(
    scope.packages === 'all'
      ? CONFIG.testAll
      : // The merge base, not `base`: a base that moved on would add the files it changed.
        CONFIG.testFiltered([`...[${mergeBase}]`, ...scope.extra]),
  );
} else if (status === 0 && scope.extra.length > 0) {
  status = runEach(CONFIG.testFiltered(scope.extra));
}

if (status === 0 && scope.scripts) status = run(CONFIG.testScripts);

// Last, so a run that fails or is stopped leaves no record.
const record = recordOf({
  before,
  after: snapshot(process.cwd(), base),
  status,
  scope: affected ? 'affected' : 'full',
});
if (record) {
  try {
    console.log(`verify: recorded the pass in ${writeRecord(record)}`);
  } catch (error) {
    console.log(`verify: could not record the pass: ${error.message}`);
  }
} else if (status === 0) {
  console.log('verify: recorded nothing, as the tree was not clean or moved during the run');
}
process.exit(status);
