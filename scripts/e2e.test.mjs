import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { after, describe, it } from 'node:test';
import { e2eTestFiles } from './e2e.mjs';

describe('the scenario test files', () => {
  const root = mkdtempSync(join(tmpdir(), 'torpedo-e2e-'));
  after(() => rmSync(root, { recursive: true, force: true }));

  it('are none while e2e holds only the coverage map', () => {
    mkdirSync(join(root, 'e2e/coverage'), { recursive: true });
    writeFileSync(join(root, 'e2e/coverage/not-yet.txt'), '');
    assert.deepEqual(e2eTestFiles(root), []);
  });

  it('are the .test.mjs files at any depth under e2e', () => {
    mkdirSync(join(root, 'e2e/trigger'), { recursive: true });
    writeFileSync(join(root, 'e2e/trigger/search.test.mjs'), '');
    writeFileSync(join(root, 'e2e/harness.mjs'), '');
    writeFileSync(join(root, 'e2e/a.test.mjs'), '');
    assert.deepEqual(e2eTestFiles(root), ['e2e/a.test.mjs', 'e2e/trigger/search.test.mjs']);
  });
});
