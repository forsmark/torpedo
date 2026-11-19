/** inline-picker: the keys during a search. */
import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { KEY } from '../harness/app.mjs';
import { commitBeforeReply, emojiOf, usePicker } from './fixture.mjs';

describe('inline-picker: keys during a search', () => {
  const t = usePicker();

  it('Down selects the next candidate', async () => {
    await t.app.type('::fire');
    const second = emojiOf(t.app.table.items[1]);
    await t.app.press(KEY.Down);
    await t.app.press(KEY.Return);
    assert.equal(t.app.text, second);
  });

  it('Up at the first candidate', async () => {
    await t.app.type('::fire');
    await t.app.press(KEY.Up);
    assert.equal(t.app.consumed, true);
    assert.equal(t.app.table.cursor, 0);
    assert.deepEqual(t.app.passed, []);
  });

  it('Page Down', async () => {
    await t.app.type('::fi');
    assert.ok(t.app.table.items.length > 9);
    await t.app.press(KEY.Page_Down);
    assert.equal(t.app.table.cursor, 9);
  });

  it('Page Up after Page Down', async () => {
    await t.app.type('::fi');
    await t.app.press(KEY.Page_Down);
    await t.app.press(KEY.Page_Up);
    assert.equal(t.app.table.cursor, 0);
  });

  it('Enter commits', async () => {
    await t.app.type('::fire');
    await t.app.press(KEY.Return);
    assert.equal(t.app.text, '🔥');
    assert.equal(t.app.consumed, true);
    assert.equal(t.app.preedit.text, '');
    assert.equal(t.app.table, null);
  });

  it('Tab commits', async () => {
    await t.app.type('::fire');
    await t.app.press(KEY.Tab);
    assert.equal(t.app.text, '🔥');
    assert.equal(t.app.consumed, true);
  });

  it('A click on the first page', async () => {
    await t.app.type('::fire');
    const third = emojiOf(t.app.table.items[2]);
    await t.app.click(2);
    assert.equal(t.app.text, third);
    assert.equal(t.app.preedit.text, '');
  });

  it('A click on the second page', async () => {
    await t.app.type('::fi');
    const tenth = emojiOf(t.app.table.items[9]);
    await t.app.press(KEY.Page_Down);
    await t.app.click(0);
    assert.equal(t.app.text, tenth);
  });

  it('Backspace removes a query character', async () => {
    await t.app.type('::fir');
    await t.app.press(KEY.BackSpace);
    assert.equal(t.app.preedit.text, '::fi');
    assert.equal(t.app.consumed, true);
  });

  it('Backspace with an empty query', async () => {
    await t.app.type('::');
    await t.app.press(KEY.BackSpace);
    assert.equal(t.app.preedit.text, '');
    assert.equal(t.app.text, '');
    assert.deepEqual(t.app.passed, []);
  });

  it('Backspace to an empty query hides the list', async () => {
    await t.app.type('::f');
    await t.app.press(KEY.BackSpace);
    assert.equal(t.app.preedit.text, '::');
    assert.equal(t.app.table, null);
  });

  it('Escape during a search', async () => {
    await t.app.type('::fi');
    await t.app.press(KEY.Escape);
    assert.equal(t.app.text, '::fi');
    assert.equal(t.app.consumed, true);
    assert.deepEqual(t.app.passed, []);
  });

  it('Escape with an empty query', async () => {
    await t.app.type('::');
    await t.app.press(KEY.Escape);
    assert.equal(t.app.text, '::');
    assert.equal(t.app.consumed, true);
  });

  it('Slack style', async () => {
    await t.app.type('::fire:');
    assert.equal(t.app.text, '🔥');
  });

  it('A colon after a partial shortcode', async () => {
    await t.app.type('::fir:');
    assert.equal(t.app.text, '::fir:');
  });

  it('A space closes the search', async () => {
    await t.app.type('::fi ');
    assert.equal(t.app.text, '::fi ');
    assert.equal(t.app.preedit.text, '');
    assert.equal(t.app.table, null);
  });

  it('A Danish letter closes the search', async () => {
    await t.app.type('::fiæ');
    assert.equal(t.app.text, '::fiæ');
  });

  it('Enter with an empty query', async () => {
    await t.app.type('::');
    await t.app.press(KEY.Return);
    assert.equal(t.app.text, '::');
    assert.deepEqual(t.app.passed.at(-1), { keyval: KEY.Return, state: 0 });
    commitBeforeReply(t.app, '::');
  });

  it('No shortcode matches', async () => {
    await t.app.type('::fire');
    await t.app.type('q');
    assert.equal(t.app.text, '::fireq');
    assert.equal(t.app.preedit.text, '');
  });
});
