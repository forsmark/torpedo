/**
 * settings-window: the save rules and the mode without a window. The window's
 * apply handlers call the same torpedo-core functions as these flags.
 */
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { afterEach, beforeEach, describe, it } from 'node:test';
import { SETTINGS, runSettings, sandbox, settingsEnv } from '../harness/settings.mjs';

describe('settings-window: saving', () => {
  let box;
  beforeEach(() => {
    box = sandbox();
  });
  afterEach(() => box.remove());

  function writeConfig(text) {
    mkdirSync(dirname(box.config), { recursive: true });
    writeFileSync(box.config, text);
  }
  const readConfig = () => readFileSync(box.config, 'utf8');

  it('Saving a bad trigger', () => {
    writeConfig('trigger = ";;"\n');
    const r = runSettings(box, ['--set-trigger', 'ab']);
    assert.equal(r.status, 1);
    assert.match(r.err, /: trigger: /);
    assert.equal(readConfig(), 'trigger = ";;"\n');
  });

  it('Saving a bad alias', () => {
    writeConfig('trigger = "::"\n');
    const r = runSettings(box, ['--set-alias', 'looks good=👍']);
    assert.equal(r.status, 1);
    assert.match(r.err, /: aliases\.looks good: /);
    assert.equal(readConfig(), 'trigger = "::"\n');
  });

  it('The file is replaced by a rename', () => {
    writeConfig('trigger = "::"\n');
    const before = statSync(box.config).ino;
    const r = runSettings(box, ['--set-trigger', ';;']);
    assert.equal(r.status, 0, r.err);
    assert.notEqual(statSync(box.config).ino, before);
    assert.match(readConfig(), /^trigger = ";;"$/m);
    assert.deepEqual(readdirSync(dirname(box.config)), ['config.toml']);
  });

  it('A reader never sees part of a file', async () => {
    writeConfig('[aliases]\nstart = "🚀"\n');
    const done = join(box.dir, 'done');
    const saves = [];
    for (let i = 0; i < 20; i++) saves.push(`--set-alias`, `a${i}=${'👍'.repeat((i % 16) + 1)}`);
    // One process per save, in a row, while this test reads the file again and again.
    const script = Array.from({ length: 20 }, (_, i) => `"$S" ${saves[2 * i]} '${saves[2 * i + 1]}' >/dev/null || exit 1`).join('\n');
    const child = spawnLoop(box, script, done);
    const seen = new Set();
    const deadline = Date.now() + 20000;
    while (!existsSync(done) && Date.now() < deadline) seen.add(readConfig());
    await new Promise((resolve) => (child.exitCode !== null ? resolve() : child.once('exit', resolve)));
    assert.equal(child.exitCode, 0);
    assert.ok(seen.size > 1, 'the reader saw the file change');
    const probe = join(box.dir, 'probe.toml');
    for (const text of seen) {
      writeFileSync(probe, text);
      const r = runSettings(box, ['--config', probe, '--check']);
      assert.equal(r.status, 0, `a read was not a whole good file:\n${text}\n${r.err}`);
    }
    assert.match(readConfig(), /a19 = /);
  });

  it('A comment and an unknown key survive', () => {
    writeConfig('# my settings\nskin_tone = "medium"\n\n[aliases]\nlgtm = "👍"\n');
    const r = runSettings(box, ['--set-alias', 'ship=🚀']);
    assert.equal(r.status, 0, r.err);
    const text = readConfig();
    assert.match(text, /^# my settings$/m);
    assert.match(text, /^skin_tone = "medium"$/m);
    assert.match(text, /^lgtm = "👍"$/m);
    assert.match(text, /^ship = "🚀"$/m);
  });

  it('The first save', () => {
    assert.equal(existsSync(box.xdg), false);
    const r = runSettings(box, ['--set-alias', 'lgtm=👍']);
    assert.equal(r.status, 0, r.err);
    assert.match(readConfig(), /^lgtm = "👍"$/m);
  });

  it('Check a good file', () => {
    writeConfig('trigger = ";;"\n\n[aliases]\nlgtm = "👍"\n');
    const r = runSettings(box, ['--check']);
    assert.equal(r.status, 0, r.err);
  });

  it('Check a bad file', () => {
    writeConfig('# one\n[aliases\n');
    const r = runSettings(box, ['--check']);
    assert.equal(r.status, 1);
    assert.ok(r.err.startsWith(`${box.config}:2: `), r.err);
  });

  it('Remove an alias', () => {
    writeConfig('[aliases]\nlgtm = "👍"\nship = "🚀"\n');
    const r = runSettings(box, ['--remove-alias', 'ship']);
    assert.equal(r.status, 0, r.err);
    assert.match(readConfig(), /^lgtm = /m);
    assert.doesNotMatch(readConfig(), /ship/);
  });

  it('Another config file', () => {
    const other = join(box.dir, 'other.toml');
    writeFileSync(other, 'trigger = ";;"\n');
    const r = runSettings(box, ['--config', 'other.toml', '--set-trigger', ':']);
    assert.equal(r.status, 0, r.err);
    assert.equal(readFileSync(other, 'utf8'), 'trigger = ":"\n');
    assert.equal(existsSync(box.xdg), false);
  });
});

/** Run the saves of `script` in one background shell, which touches `done` when it ends. */
function spawnLoop(box, script, done) {
  rmSync(done, { force: true });
  return spawn('sh', ['-c', `run() {\n${script}\n}\nrun\nstatus=$?\ntouch '${done}'\nexit $status`], {
    env: { ...settingsEnv(box), S: SETTINGS },
    cwd: box.dir,
    stdio: 'ignore',
  });
}
