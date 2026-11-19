/** inline-picker: used emoji come first within a rank. */
import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { usePicker } from './fixture.mjs';

describe('inline-picker: usage in the ranking', () => {
  const t = usePicker();

  it('Two commits move an emoji to the front of its rank', async () => {
    await t.app.type('::thumbsdown: ::thumbsdown: ::thumbs');
    assert.deepEqual(t.app.table.items, ['👎 thumbsdown', '👍 thumbsup']);
  });

  it('A higher usage score comes first', async () => {
    t.session.writeUsage([
      { text: '👍', score: 1 },
      { text: '👎', score: 3 },
    ]);
    await t.app.type('::thumbs');
    assert.equal(t.app.table.items[0], '👎 thumbsdown');
  });

  it('An unused exact match beats a used prefix match', async () => {
    await t.app.type('::fire_extinguisher: ::fire_extinguisher: ::fire');
    assert.equal(t.app.table.items[0], '🔥 fire');
    assert.equal(t.app.table.items[1], '🧯 fire_extinguisher');
  });

});

describe('inline-picker: usage of an alias text', () => {
  const t = usePicker({ config: '[aliases]\nthumbsboth = "👍👎"\nthumbsall = "🙌"\n' });

  it('A used alias text comes first among aliases', async () => {
    await t.app.type('::thumbsboth: ::thumbs');
    assert.deepEqual(t.app.table.items, ['👍👎 thumbsboth', '🙌 thumbsall', '👍 thumbsup', '👎 thumbsdown']);
  });
});
