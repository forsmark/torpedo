/** inline-picker: held text is never lost. */
import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { KEY } from '../harness/app.mjs';
import { usePicker } from './fixture.mjs';

/** The commits that reached the app since `from` in its log. */
function commitsSince(app, from) {
  return app.log.slice(from).filter((e) => e.type === 'commit').map((e) => e.text);
}

describe('inline-picker: held text', () => {
  const t = usePicker();

  it('Focus moves away during a search', async () => {
    await t.app.type('::fi');
    const from = t.app.log.length;
    await t.app.focusOut();
    assert.deepEqual(commitsSince(t.app, from), ['::fi']);
    assert.equal(t.app.text, '::fi');
  });

  it('The app resets the input context during a search', async () => {
    await t.app.type('::fi');
    const from = t.app.log.length;
    await t.app.reset();
    assert.deepEqual(commitsSince(t.app, from), ['::fi']);
    assert.equal(t.app.text, '::fi');
  });

  it('The engine is switched off during a search', async () => {
    await t.app.type('::fi');
    const from = t.app.log.length;
    await t.app.switchEngine('xkb:dk::dan');
    assert.deepEqual(commitsSince(t.app, from), ['::fi']);
    assert.equal(t.app.text, '::fi');
  });

  it('Focus moves away with a held dead key', async () => {
    await t.app.press(KEY.dead_acute);
    const from = t.app.log.length;
    await t.app.focusOut();
    assert.deepEqual(commitsSince(t.app, from), ['´']);
  });

  it('Typing after a reset', async () => {
    await t.app.type('::fi');
    await t.app.reset();
    await t.app.type('a');
    assert.equal(t.app.consumed, false);
    assert.equal(t.app.preedit.text, '');
    assert.equal(t.app.text, '::fia');
  });
});
