/** inline-picker: the preedit, the query and the candidate list. */
import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { usePicker } from './fixture.mjs';

describe('inline-picker: search', () => {
  const t = usePicker();

  it('The preedit of a search', async () => {
    await t.app.type('::fi');
    assert.equal(t.app.preedit.text, '::fi');
    assert.deepEqual(t.app.preedit.attrs, [[1, 1, 0, 4]]);
  });

  it('A shortcode with a plus sign', async () => {
    await t.app.type('::+1');
    assert.equal(t.app.table.items[0], '👍 +1');
  });

  it('A shortcode with digits and underscores', async () => {
    await t.app.type('::1st_place');
    assert.equal(t.app.table.items[0], '🥇 1st_place_medal');
  });

  it('Upper case letters', async () => {
    await t.app.type('::FIRE');
    assert.equal(t.app.table.items[0], '🔥 fire');
  });

  it('No candidate list for an empty query', async () => {
    await t.app.type('::');
    assert.equal(t.app.table, null);
  });

  it('The candidate list for a query', async () => {
    await t.app.type('::fi');
    assert.equal(t.app.table.visible, true);
    assert.equal(t.app.table.page_size, 9);
    assert.equal(t.app.table.cursor, 0);
  });

  it('Candidates have no labels', async () => {
    await t.app.type('::fi');
    assert.deepEqual(t.app.table.labels, []);
  });

  it('A candidate shows its emoji and shortcode', async () => {
    await t.app.type('::fire');
    assert.equal(t.app.table.items[0], '🔥 fire');
  });

  it('An emoji with two matching shortcodes', async () => {
    await t.app.type('::fist');
    assert.deepEqual(
      t.app.table.items.filter((c) => c.startsWith('✊ ')),
      ['✊ fist'],
    );
  });

  it('Exact before prefix before word prefix', async () => {
    await t.app.type('::fire');
    assert.deepEqual(t.app.table.items, [
      '🔥 fire',
      '🧑‍🚒 firefighter',
      '🚒 fire_engine',
      '🎆 fireworks',
      '🧨 firecracker',
      '🧯 fire_extinguisher',
      '❤️‍🔥 heart_on_fire',
      '👨‍🚒 man_firefighter',
      '👩‍🚒 woman_firefighter',
    ]);
  });

  it('Prefix before substring', async () => {
    await t.app.type('::ire');
    const items = t.app.table.items;
    assert.equal(items[0], '🇮🇪 ireland');
    assert.ok(items.indexOf('🇮🇪 ireland') < items.indexOf('😫 tired_face'));
  });

  it("Ties keep gemoji's order", async () => {
    await t.app.type('::thumbs');
    assert.deepEqual(t.app.table.items, ['👍 thumbsup', '👎 thumbsdown']);
  });
});
