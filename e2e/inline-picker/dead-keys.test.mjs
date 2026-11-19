/** inline-picker: dead keys compose in the engine. */
import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { KEY, SHIFT } from '../harness/app.mjs';
import { commitBeforeReply, usePicker } from './fixture.mjs';

describe('inline-picker: dead keys', () => {
  const t = usePicker();

  it('The dead acute key', async () => {
    await t.app.press(KEY.dead_acute);
    assert.equal(t.app.consumed, true);
    assert.equal(t.app.preedit.text, '´');
    assert.equal(t.app.text, '');
  });

  it('The accent is underlined', async () => {
    await t.app.press(KEY.dead_diaeresis);
    assert.equal(t.app.preedit.text, '¨');
    assert.deepEqual(t.app.preedit.attrs, [[1, 1, 0, 1]]);
  });

  it('An unknown dead key', async () => {
    await t.app.press(KEY.dead_belowdot);
    assert.equal(t.app.consumed, false);
    assert.equal(t.app.preedit.text, '');
  });

  it('Acute and e', async () => {
    await t.app.press(KEY.dead_acute);
    await t.app.type('e');
    assert.equal(t.app.text, 'é');
    assert.equal(t.app.preedit.text, '');
  });

  it('Diaeresis and u', async () => {
    await t.app.press(KEY.dead_diaeresis);
    await t.app.type('u');
    assert.equal(t.app.text, 'ü');
  });

  it('Tilde and n', async () => {
    await t.app.press(KEY.dead_tilde);
    await t.app.type('n');
    assert.equal(t.app.text, 'ñ');
  });

  it('Acute and upper case E', async () => {
    await t.app.press(KEY.dead_acute);
    await t.app.press(0x45, SHIFT);
    assert.equal(t.app.text, 'É');
  });

  it('Dead key and space', async () => {
    await t.app.press(KEY.dead_acute);
    await t.app.type(' ');
    assert.equal(t.app.text, '´');
    assert.equal(t.app.consumed, true);
  });

  it('The same dead key twice', async () => {
    await t.app.press(KEY.dead_circumflex);
    await t.app.press(KEY.dead_circumflex);
    assert.equal(t.app.text, '^');
    assert.equal(t.app.preedit.text, '');
  });

  it('Acute and q', async () => {
    await t.app.press(KEY.dead_acute);
    await t.app.type('q');
    assert.equal(t.app.text, '´q');
  });

  it('Dead key and Left', async () => {
    await t.app.press(KEY.dead_acute);
    await t.app.press(KEY.Left);
    assert.equal(t.app.text, '´');
    assert.deepEqual(t.app.passed, [{ keyval: KEY.Left, state: 0 }]);
    commitBeforeReply(t.app, '´');
  });

  it('Two different dead keys', async () => {
    await t.app.press(KEY.dead_acute);
    await t.app.press(KEY.dead_diaeresis);
    assert.equal(t.app.text, '´');
    assert.equal(t.app.preedit.text, '¨');
  });

  it('Backspace after a dead key', async () => {
    await t.app.press(KEY.dead_acute);
    await t.app.press(KEY.BackSpace);
    assert.equal(t.app.preedit.text, '');
    assert.equal(t.app.text, '');
    assert.deepEqual(t.app.passed, []);
  });

  it('Shift after a dead key', async () => {
    await t.app.press(KEY.dead_acute);
    await t.app.press(KEY.Shift_L);
    await t.app.press(0x45, SHIFT);
    assert.equal(t.app.text, 'É');
  });

  it('A dead key while the first colon is held', async () => {
    await t.app.type(':');
    await t.app.press(KEY.dead_acute);
    assert.equal(t.app.text, ':');
    assert.equal(t.app.preedit.text, '´');
  });

  it('A dead key during a search', async () => {
    await t.app.type('::fi');
    await t.app.press(KEY.dead_acute);
    assert.equal(t.app.text, '::fi');
    assert.equal(t.app.preedit.text, '´');
    assert.equal(t.app.table, null);
  });
});
