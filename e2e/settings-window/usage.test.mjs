/**
 * settings-window: the flags for the skin tone and usage, in the mode without
 * a window. The window's selector, switch and "Clear usage" button save and
 * delete through the same torpedo-core functions.
 */
import assert from 'node:assert/strict';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { afterEach, beforeEach, describe, it } from 'node:test';
import { runSettings, sandbox } from '../harness/settings.mjs';

describe('settings-window: skin tone and usage flags', () => {
  let box;
  beforeEach(() => {
    box = sandbox();
  });
  afterEach(() => box.remove());

  function write(path, text) {
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, text);
  }

  it('Set the skin tone', () => {
    write(box.config, 'trigger = ";;"\n');
    const r = runSettings(box, ['--set-skin-tone', 'medium-light']);
    assert.equal(r.status, 0, r.err);
    const text = readFileSync(box.config, 'utf8');
    assert.match(text, /^skin_tone = "medium-light"$/m);
    assert.match(text, /^trigger = ";;"$/m);
  });

  it('Turn usage off', () => {
    write(box.config, 'trigger = "::"\n');
    const r = runSettings(box, ['--set-remember-usage', 'false']);
    assert.equal(r.status, 0, r.err);
    assert.match(readFileSync(box.config, 'utf8'), /^remember_usage = false$/m);
  });

  it('Clear the usage', () => {
    write(box.usage, '[[used]]\ntext = "🔥"\nscore = 1.0\nlast = 1\n');
    const r = runSettings(box, ['--clear-usage']);
    assert.equal(r.status, 0, r.err);
    assert.equal(existsSync(box.usage), false);
  });

  it('Clear the usage when there is none', () => {
    const r = runSettings(box, ['--clear-usage']);
    assert.equal(r.status, 0, r.err);
    assert.equal(existsSync(box.usage), false);
  });

  it('The default usage file location', () => {
    const usage = join(box.home, '.local', 'state', 'torpedo', 'usage.toml');
    write(usage, '[[used]]\ntext = "🔥"\nscore = 1.0\nlast = 1\n');
    const r = runSettings(box, ['--clear-usage'], { env: { XDG_STATE_HOME: null } });
    assert.equal(r.status, 0, r.err);
    assert.equal(existsSync(usage), false);
  });
});
