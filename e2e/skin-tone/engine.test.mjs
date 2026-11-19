/**
 * skin-tone: the engine shows and commits emoji in the skin tone of the
 * config file, never tones an alias text, and records the bare emoji.
 */
import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { KEY } from '../harness/app.mjs';
import { usePicker } from '../inline-picker/fixture.mjs';

const hex = (s) => [...s].map((c) => c.codePointAt(0).toString(16).toUpperCase()).join(' ');

describe('skin-tone: medium', () => {
  const t = usePicker({
    config: 'skin_tone = "medium"\n[aliases]\nhi = "👋"\nwave_hi = "👋"\n',
  });

  it('A candidate in a tone', async () => {
    await t.app.type('::wave');
    assert.equal(t.app.table.items[0], '👋🏽 wave');
  });

  it('Enter commits the tone', async () => {
    await t.app.type('::wave');
    await t.app.press(KEY.Return);
    assert.equal(hex(t.app.text), '1F44B 1F3FD');
  });

  it('An emoji with no toned form', async () => {
    await t.app.type('::fire:');
    assert.equal(t.app.text, '🔥');
  });

  it('The VS16 after a modifier base goes', async () => {
    await t.app.type('::v:');
    assert.equal(hex(t.app.text), '270C 1F3FD');
  });

  it('An alias of a hand', async () => {
    await t.app.type('::hi:');
    assert.equal(hex(t.app.text), '1F44B');
  });

  it('An alias and a shortcode of the same emoji', async () => {
    await t.app.type('::wave');
    assert.equal(t.app.table.items[0], '👋🏽 wave');
    assert.ok(!t.app.table.items.includes('👋 wave_hi'), t.app.table.items.join(', '));
  });

  it('The alias name as the best match', async () => {
    await t.app.type('::wave_h');
    assert.equal(t.app.table.items[0], '👋 wave_hi');
  });

  it('A toned commit records the bare emoji', async () => {
    await t.app.type('::wave:');
    assert.equal(hex(t.app.text), '1F44B 1F3FD');
    assert.ok(t.session.usageOf('👋'));
    assert.equal(t.session.usageOf('👋🏽'), undefined);
  });
});

describe('skin-tone: other tones', () => {
  const t = usePicker({ config: 'skin_tone = "dark"\n' });

  it('The closing colon commits the tone', async () => {
    await t.app.type('::+1:');
    assert.equal(hex(t.app.text), '1F44D 1F3FF');
  });

  it('A new skin tone keeps the ranking', async () => {
    t.session.writeUsage([{ text: '👎', score: 3 }]);
    await t.app.type('::thumbs');
    assert.deepEqual(t.app.table.items, ['👎🏿 thumbsdown', '👍🏿 thumbsup']);
  });

  it('A hand in the light tone', async () => {
    t.session.writeConfig('skin_tone = "light"\n');
    await t.app.type('::raised_back_of_hand:');
    assert.equal(hex(t.app.text), '1F91A 1F3FB');
  });

  it('Two people get the same tone', async () => {
    t.session.writeConfig('skin_tone = "light"\n');
    await t.app.type('::couple_with_heart_woman_man:');
    assert.equal(hex(t.app.text), '1F469 1F3FB 200D 2764 FE0F 200D 1F468 1F3FB');
  });

  it('A person with a sign', async () => {
    t.session.writeConfig('skin_tone = "medium-dark"\n');
    await t.app.type('::golfing_woman:');
    assert.equal(hex(t.app.text), '1F3CC 1F3FE 200D 2640 FE0F');
  });

  it('The handshake between two people stays bare', async () => {
    t.session.writeConfig('skin_tone = "medium-light"\n');
    await t.app.type('::people_holding_hands:');
    assert.equal(hex(t.app.text), '1F9D1 1F3FC 200D 1F91D 200D 1F9D1 1F3FC');
  });

  it('No skin tone', async () => {
    t.session.writeConfig('skin_tone = "none"\n');
    await t.app.type('::wave:');
    assert.equal(hex(t.app.text), '1F44B');
  });
});
