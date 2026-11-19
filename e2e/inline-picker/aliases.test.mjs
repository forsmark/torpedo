/** inline-picker: aliases in the search, the ranking and the closing colon. */
import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { KEY } from '../harness/app.mjs';
import { usePicker } from './fixture.mjs';

const SHRUG = '¯\\_(ツ)_/¯';

describe('inline-picker: aliases', () => {
  const t = usePicker({
    config: [
      '[aliases]',
      'lgtm = "👍"',
      `shrug = '${SHRUG}'`,
      'fireball = "🔥"',
      'fi = "🇫🇮"',
      'thumbsboth = "👍👎"',
      'thumbsall = "🙌"',
      'zzbot = "🤖"',
      '',
    ].join('\n'),
  });

  it('A colon after an alias', async () => {
    await t.app.type('::lgtm:');
    assert.equal(t.app.text, '👍');
  });

  it('A candidate shows its alias', async () => {
    await t.app.type('::lgtm');
    assert.equal(t.app.table.items[0], '👍 lgtm');
  });

  it('An emoji with a matching shortcode and alias', async () => {
    await t.app.type('::fire');
    assert.deepEqual(
      t.app.table.items.filter((c) => c.startsWith('🔥 ')),
      ['🔥 fire'],
    );
  });

  it('An exact alias before prefix matches', async () => {
    await t.app.type('::fi');
    assert.equal(t.app.table.items[0], '🇫🇮 fi');
  });

  it('Aliases first among equal ranks', async () => {
    await t.app.type('::thumbs');
    assert.deepEqual(t.app.table.items, ['🙌 thumbsall', '👍👎 thumbsboth', '👍 thumbsup', '👎 thumbsdown']);
  });

  it('Only an alias matches', async () => {
    await t.app.type('::zzb');
    assert.equal(t.app.preedit.text, '::zzb');
    assert.equal(t.app.text, '');
    assert.equal(t.app.table.items[0], '🤖 zzbot');
  });

  it('An alias by its prefix', async () => {
    await t.app.type('::lg');
    assert.ok(t.app.table.items.includes('👍 lgtm'), t.app.table.items.join(', '));
  });

  it('An alias in upper case', async () => {
    await t.app.type('::LGTM');
    assert.equal(t.app.table.items[0], '👍 lgtm');
  });

  it('An alias for a text emoji', async () => {
    await t.app.type('::shrug');
    await t.app.press(KEY.Return);
    assert.equal(t.app.text, SHRUG);
  });
});

describe('inline-picker: an alias named like a shortcode', () => {
  const t = usePicker({ config: '[aliases]\nfire = "🚒"\n' });

  it('An alias named fire', async () => {
    await t.app.type('::fire:');
    assert.equal(t.app.text, '🚒');
  });

  it('The shortcode it hides', async () => {
    await t.app.type('::fire');
    assert.equal(t.app.table.items[0], '🚒 fire');
    assert.ok(!t.app.table.items.includes('🔥 fire'));
  });
});
