/**
 * Typing faster than the engine answers. A real client sends the next key
 * before the reply to the previous one. This test sends a whole burst of keys
 * with process_key_event_async and waits only at the end. The app's text must
 * come out exactly as typed, so the engine must handle the keys one at a time
 * and in the order they arrive.
 *
 * No scenario names this. It guards the order that every inline-picker
 * scenario takes for granted.
 */
import assert from 'node:assert/strict';
import { after, before, beforeEach, describe, it } from 'node:test';
import { App, KEY } from './app.mjs';
import { openSession } from './session.mjs';

const BURSTS = 5;

/** About 26 keys: plain text, a search with Enter, and a dead key compose. */
function burstKeys() {
  return [
    ...App.keysOf('hej med dig '),
    ...App.keysOf('::fire'),
    { keyval: KEY.Return, state: 0 },
    ...App.keysOf(' '),
    { keyval: KEY.dead_acute, state: 0 },
    ...App.keysOf('e ok'),
  ];
}

describe('a burst of keys', () => {
  let session;
  let app;
  before(async () => {
    session = await openSession();
  });
  after(async () => {
    await session?.close();
  });
  beforeEach(async () => {
    app = await session.context();
  });

  for (let i = 1; i <= BURSTS; i++) {
    it(`comes out exactly as typed, burst ${i}`, async () => {
      const keys = burstKeys();
      assert.ok(keys.length >= 25);
      await app.burst(keys);
      assert.equal(app.text, 'hej med dig 🔥 é ok');
      assert.equal(app.preedit.text, '');
      assert.equal(app.table, null);
    });
  }
});
