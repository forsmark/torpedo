/**
 * usage: what the engine records, the usage score, the usage file and how the
 * engine reads it. The picker fixture deletes the usage file before each
 * test, and a usage file a test writes is read when the next search opens.
 */
import assert from 'node:assert/strict';
import { readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { describe, it } from 'node:test';
import { KEY } from '../harness/app.mjs';
import { readTomlFile } from '../harness/session.mjs';
import { usePicker } from '../inline-picker/fixture.mjs';

const DAY = 24 * 60 * 60;
const BROKEN = '[[used]\n';
const near = (a, b) => Math.abs(a - b) <= 0.01;
const nowSecs = () => Date.now() / 1000;

describe('usage: recording', () => {
  const t = usePicker({ config: '[aliases]\nlgtm = "👍"\n' });

  it('The usage file under XDG_STATE_HOME', async () => {
    await t.app.type('::fire:');
    assert.equal(t.session.usagePath, join(t.session.run, 'state', 'torpedo', 'usage.toml'));
    assert.ok(t.session.usageOf('🔥'));
  });

  it('Enter records the emoji', async () => {
    await t.app.type('::fire');
    await t.app.press(KEY.Return);
    assert.equal(t.app.text, '🔥');
    assert.equal(t.session.usageOf('🔥').score, 1);
  });

  it('The closing colon records an alias text', async () => {
    await t.app.type('::lgtm:');
    assert.equal(t.session.usageOf('👍').score, 1);
  });

  it('Two commits of one emoji', async () => {
    await t.app.type('::fire: ::fire:');
    assert.equal(t.app.text, '🔥 🔥');
    assert.ok(near(t.session.usageOf('🔥').score, 2), String(t.session.usageOf('🔥').score));
  });

  it('Typing a query writes nothing', async () => {
    await t.app.type('::fir');
    assert.equal(t.session.readUsage(), null);
  });

  it('Escape records nothing', async () => {
    await t.app.type('::fi');
    await t.app.press(KEY.Escape);
    assert.equal(t.app.text, '::fi');
    assert.equal(t.session.readUsage(), null);
  });

  it('A query with no match records nothing', async () => {
    await t.app.type('::fireq');
    assert.equal(t.app.text, '::fireq');
    assert.equal(t.session.readUsage(), null);
  });
});

describe('usage: the usage score', () => {
  const t = usePicker();

  it('A recent use beats an old one', async () => {
    t.session.writeUsage([
      { text: '👍', score: 4, age: 60 * DAY },
      { text: '👎', score: 1, age: 0 },
    ]);
    await t.app.type('::thumbs');
    assert.equal(t.app.table.items[0], '👎 thumbsdown');
  });

  it('A commit adds to the faded score', async () => {
    t.session.writeUsage([{ text: '🔥', score: 2, age: 14 * DAY }]);
    await t.app.type('::fire:');
    const e = t.session.usageOf('🔥');
    assert.ok(near(e.score, 2), String(e.score));
    assert.ok(Math.abs(e.last - nowSecs()) < 30);
  });
});

describe('usage: the usage file', () => {
  const t = usePicker();

  it('A full usage file drops its lowest entry', async () => {
    const entries = Array.from({ length: 199 }, (_, i) => ({ text: `t${i}`, score: 2 }));
    t.session.writeUsage([...entries, { text: '😀', score: 0.5 }]);
    await t.app.type('::fire:');
    const after = t.session.readUsage();
    assert.equal(after.length, 200);
    assert.ok(after.some((e) => e.text === '🔥'));
    assert.ok(!after.some((e) => e.text === '😀'));
  });

  it('A usage file below the limit keeps every entry', async () => {
    t.session.writeUsage(Array.from({ length: 199 }, (_, i) => ({ text: `t${i}`, score: 2 })));
    await t.app.type('::fire:');
    assert.equal(t.session.readUsage().length, 200);
  });

  it('The usage file is replaced by a rename', async () => {
    t.session.writeUsage([{ text: '👍' }]);
    const before = statSync(t.session.usagePath).ino;
    await t.app.type('::fire:');
    assert.notEqual(statSync(t.session.usagePath).ino, before);
    assert.ok(t.session.usageOf('🔥'));
    assert.deepEqual(readdirSync(dirname(t.session.usagePath)), ['usage.toml']);
  });

  it('A reader never sees part of a usage file', async () => {
    const seen = new Set();
    let reading = true;
    const reader = (async () => {
      while (reading) {
        try {
          seen.add(readFileSync(t.session.usagePath, 'utf8'));
        } catch (e) {
          if (e.code !== 'ENOENT') throw e;
        }
        await new Promise((r) => setImmediate(r));
      }
    })();
    for (let i = 0; i < 20; i++) await t.app.type(' ::fire:');
    reading = false;
    await reader;
    assert.ok(seen.size >= 2, `the reader saw ${seen.size} versions`);
    const dir = t.session.run;
    for (const [i, text] of [...seen].entries()) {
      const f = join(dir, `read-${i}.toml`);
      writeFileSync(f, text);
      const doc = readTomlFile(f);
      assert.ok(Array.isArray(doc.used) && doc.used.length === 1, text);
    }
    assert.ok(near(t.session.usageOf('🔥').score, 20));
  });
});

describe('usage: a missing or broken usage file', () => {
  const t = usePicker();

  it('A broken usage file at start', async () => {
    t.session.writeUsage(BROKEN);
    await t.app.type('::thumbs');
    assert.equal(t.app.table.items[0], '👍 thumbsup');
    await t.app.press(KEY.Escape);
    await t.app.type(' ::fire:');
    assert.equal(t.app.text, '::thumbs 🔥');
  });

  it('A usage file with a bad entry', async () => {
    t.session.writeUsage('[[used]]\ntext = "👎"\nscore = 5.0\nlast = 1\n\n[[used]]\nscore = 1.0\nlast = 1\n');
    await t.app.type('::thumbs');
    assert.equal(t.app.table.items[0], '👍 thumbsup');
  });

  it('A commit replaces a broken usage file', async () => {
    t.session.writeUsage(BROKEN);
    await t.app.type('::fire:');
    assert.deepEqual(t.session.readUsage().map((e) => e.text), ['🔥']);
  });

  it('A broken usage file is kept aside', async () => {
    const aside = `${t.session.usagePath}.broken`;
    t.session.writeUsage(BROKEN);
    writeFileSync(aside, 'older\n');
    await t.app.type('::fire:');
    assert.equal(readFileSync(aside, 'utf8'), BROKEN);
    assert.deepEqual(t.session.readUsage().map((e) => e.text), ['🔥']);
  });
});

describe('usage: turned off', () => {
  const t = usePicker({ config: 'remember_usage = false\n' });

  it('Usage off writes nothing', async () => {
    await t.app.type('::fire:');
    assert.equal(t.app.text, '🔥');
    assert.equal(t.session.readUsage(), null);
  });

  it('Usage off ignores the usage file', async () => {
    t.session.writeUsage([{ text: '👎', score: 5 }]);
    await t.app.type('::thumbs');
    assert.equal(t.app.table.items[0], '👍 thumbsup');
  });

  it('Usage off keeps the usage file', async () => {
    t.session.writeUsage([{ text: '👎', score: 1 }]);
    const before = readFileSync(t.session.usagePath, 'utf8');
    const ino = statSync(t.session.usagePath).ino;
    await t.app.type('::fire:');
    assert.equal(readFileSync(t.session.usagePath, 'utf8'), before);
    assert.equal(statSync(t.session.usagePath).ino, ino);
  });
});

describe('usage: a changed usage file', () => {
  const t = usePicker();

  it('A new usage file when a search opens', async () => {
    await t.app.type('a ');
    t.session.writeUsage([{ text: '👎', score: 5 }]);
    await t.app.type('::thumbs');
    assert.equal(t.app.table.items[0], '👎 thumbsdown');
  });

  it('A deleted usage file after focus-in', async () => {
    await t.app.type('::thumbsdown: ::thumbsdown: ');
    await t.app.type('::thumbs');
    assert.equal(t.app.table.items[0], '👎 thumbsdown', 'the commits ranked it first');
    await t.app.press(KEY.Escape);
    t.session.removeUsage();
    await t.app.focusOut();
    await t.app.focusIn();
    await t.app.type('::thumbs');
    assert.equal(t.app.table.items[0], '👍 thumbsup');
  });

  it('A commit after the usage file is deleted', async () => {
    await t.app.type('::thumbsdown: ::thumbsdown:');
    t.session.removeUsage();
    await t.app.focusOut();
    await t.app.focusIn();
    await t.app.type('::fire:');
    assert.deepEqual(t.session.readUsage().map((e) => e.text), ['🔥']);
  });
});
