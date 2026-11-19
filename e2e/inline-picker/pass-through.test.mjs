/** inline-picker: keys that pass through, and held text before them. */
import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { CONTROL, KEY, MOD1, MOD2, MOD4, MOD5, SHIFT, SUPER } from '../harness/app.mjs';
import { commitBeforeReply, usePicker } from './fixture.mjs';

describe('inline-picker: pass-through', () => {
  const t = usePicker();

  it('A letter with nothing held passes through', async () => {
    await t.app.type('a');
    assert.equal(t.app.consumed, false);
    assert.equal(t.app.text, 'a');
    assert.deepEqual(t.app.commits, []);
    assert.equal(t.app.preedit.text, '');
  });

  it('A key release passes through', async () => {
    await t.app.type('::fi');
    await t.app.release(0x69);
    assert.equal(t.app.consumed, false);
    assert.equal(t.app.preedit.text, '::fi');
  });

  it('Ctrl with a letter passes through', async () => {
    await t.app.press(0x63, CONTROL);
    assert.equal(t.app.consumed, false);
  });

  it('Alt with a letter passes through', async () => {
    await t.app.press(0x66, MOD1);
    assert.equal(t.app.consumed, false);
  });

  it('Super with a letter passes through', async () => {
    await t.app.press(0x61, MOD4 | SUPER);
    assert.equal(t.app.consumed, false);
  });

  it('Shift gives the colon of the trigger', async () => {
    await t.app.press(0x3a, SHIFT);
    await t.app.press(0x3a, SHIFT);
    assert.equal(t.app.preedit.text, '::');
    assert.equal(t.app.text, '');
  });

  it('AltGr gives a character', async () => {
    await t.app.type('::fi');
    await t.app.press(0x7b, MOD5);
    assert.equal(t.app.text, '::fi{');
    assert.equal(t.app.preedit.text, '');
    assert.equal(t.app.table, null);
  });

  it('Num Lock does not stop the trigger', async () => {
    await t.app.press(0x3a, SHIFT | MOD2);
    await t.app.press(0x3a, SHIFT | MOD2);
    assert.equal(t.app.preedit.text, '::');
  });

  it('Shift pressed again between the colons', async () => {
    await t.app.type(':');
    await t.app.press(KEY.Shift_L);
    await t.app.release(KEY.Shift_L, SHIFT);
    await t.app.type(':');
    assert.equal(t.app.preedit.text, '::');
    assert.equal(t.app.text, '');
  });

  it('Ctrl with a letter during a search', async () => {
    await t.app.type('::fi');
    await t.app.press(0x61, CONTROL);
    assert.equal(t.app.text, '::fi');
    assert.deepEqual(t.app.passed.at(-1), { keyval: 0x61, state: CONTROL });
    commitBeforeReply(t.app, '::fi');
    assert.equal(t.app.preedit.text, '');
    assert.equal(t.app.table, null);
  });

  it('An arrow key during a search', async () => {
    await t.app.type('::fi');
    await t.app.press(KEY.Left);
    assert.equal(t.app.text, '::fi');
    assert.deepEqual(t.app.passed.at(-1), { keyval: KEY.Left, state: 0 });
    commitBeforeReply(t.app, '::fi');
    assert.equal(t.app.preedit.text, '');
  });
});
