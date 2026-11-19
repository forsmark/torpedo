/** inline-picker: where the trigger counts, and the held trigger prefix. */
import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { KEY } from '../harness/app.mjs';
import { usePicker } from './fixture.mjs';

describe('inline-picker: trigger and boundary', () => {
  const t = usePicker();

  it('A trigger after a word', async () => {
    await t.app.surrounding('std');
    await t.app.type('::');
    assert.equal(t.app.text, '::');
    assert.equal(t.app.preedit.text, '');
  });

  it('A trigger after a space', async () => {
    await t.app.surrounding('hello ');
    await t.app.type('::');
    assert.equal(t.app.preedit.text, '::');
  });

  it('A trigger at the start of the text', async () => {
    await t.app.surrounding('');
    await t.app.type('::');
    assert.equal(t.app.preedit.text, '::');
  });

  it('A trigger after an emoji', async () => {
    await t.app.surrounding('ok 🔥');
    await t.app.type('::');
    assert.equal(t.app.preedit.text, '::');
  });

  it('A trigger after an opening bracket', async () => {
    await t.app.surrounding('(');
    await t.app.type('::smile:');
    assert.equal(t.app.text, '😄');
  });

  it('A trigger after an opening quote', async () => {
    await t.app.surrounding('„');
    await t.app.type('::');
    assert.equal(t.app.preedit.text, '::');
  });

  it('A second input context is asked', async () => {
    const second = await t.session.context('second');
    assert.ok(second.log.some((e) => e.type === 'require-surrounding-text' && e.context === 'second'));
    assert.equal(second.needsSurroundingText, true);
  });

  it('A trigger after a typed letter', async () => {
    await t.app.type('d::');
    assert.equal(t.app.text, 'd::');
    assert.equal(t.app.preedit.text, '');
  });

  it('A trigger after a typed space', async () => {
    await t.app.type('d ::');
    assert.equal(t.app.preedit.text, '::');
  });

  it('A trigger with nothing known', async () => {
    await t.app.type('::');
    assert.equal(t.app.preedit.text, '::');
  });

  it('Two emoji in a row', async () => {
    await t.app.type('::fire:::rocket:');
    assert.equal(t.app.text, '🔥🚀');
  });

  it('Two colons open a search', async () => {
    await t.app.type('::');
    assert.equal(t.app.preedit.text, '::');
    assert.equal(t.app.table, null);
    assert.equal(t.app.text, '');
  });

  it('The first colon is held', async () => {
    await t.app.type(':');
    assert.equal(t.app.consumed, true);
    assert.equal(t.app.preedit.text, ':');
    assert.deepEqual(t.app.commits, []);
  });

  it('A letter after the first colon', async () => {
    await t.app.type(':a');
    assert.equal(t.app.text, ':a');
    assert.equal(t.app.preedit.text, '');
  });

  it('Backspace after the first colon', async () => {
    await t.app.type(':');
    await t.app.press(KEY.BackSpace);
    assert.equal(t.app.consumed, true);
    assert.equal(t.app.preedit.text, '');
    assert.equal(t.app.text, '');
    assert.deepEqual(t.app.passed, []);
  });
});
