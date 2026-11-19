/** inline-picker: the configurable trigger, and the trigger rules with another trigger. */
import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { KEY } from '../harness/app.mjs';
import { usePicker } from './fixture.mjs';

describe('inline-picker: the default trigger', () => {
  const t = usePicker();

  it('Two colons open a search', async () => {
    await t.app.type('::');
    assert.equal(t.app.preedit.text, '::');
    assert.equal(t.app.table, null);
    assert.equal(t.app.text, '');
  });
});

describe('inline-picker: the trigger ;;', () => {
  const t = usePicker({ config: 'trigger = ";;"\n' });

  it('Two semicolons open a search', async () => {
    await t.app.type(';;');
    assert.equal(t.app.preedit.text, ';;');
    assert.equal(t.app.table, null);
    assert.equal(t.app.text, '');
  });

  it('Two colons with another trigger', async () => {
    await t.app.type('::fire');
    assert.equal(t.app.text, '::fire');
    assert.equal(t.app.preedit.text, '');
    assert.equal(t.app.table, null);
  });

  it('A colon closes a search of another trigger', async () => {
    await t.app.type(';;fire:');
    assert.equal(t.app.text, '🔥');
  });

  it('Another trigger after a word', async () => {
    await t.app.surrounding('std');
    await t.app.type(';;');
    assert.equal(t.app.text, ';;');
    assert.equal(t.app.preedit.text, '');
  });

  it('The first character of another trigger is held', async () => {
    await t.app.type(';');
    assert.equal(t.app.consumed, true);
    assert.equal(t.app.preedit.text, ';');
    assert.deepEqual(t.app.commits, []);
  });

  it('Backspace after the first character of another trigger', async () => {
    await t.app.type(';');
    await t.app.press(KEY.BackSpace);
    assert.equal(t.app.consumed, true);
    assert.equal(t.app.preedit.text, '');
    assert.equal(t.app.text, '');
    assert.deepEqual(t.app.passed, []);
  });
});

describe('inline-picker: the trigger :', () => {
  const t = usePicker({ config: 'trigger = ":"\n' });

  it('A trigger of one colon', async () => {
    await t.app.type(':fire:');
    assert.equal(t.app.text, '🔥');
  });
});
