/**
 * The wire order of a commit and a key the engine lets through.
 *
 * For a key without a character, the engine commits the held text and
 * returns false. An async client gets the CommitText signal and the
 * ProcessKeyEvent reply as two messages. This test sends `:` and then Ctrl+a
 * or Left many times with process_key_event_async, and checks that the commit
 * arrives before the reply every time. docs/ibus-protocol.md records the
 * result.
 */
import assert from 'node:assert/strict';
import { after, before, describe, it } from 'node:test';
import { CONTROL, KEY } from './app.mjs';
import { openSession } from './session.mjs';

const RUNS = 50;

describe('the order of a commit and a passed key', () => {
  let session;
  let app;
  before(async () => {
    session = await openSession();
    app = await session.context();
  });
  after(async () => {
    await session?.close();
  });

  for (const [name, keyval, state] of [
    ['Ctrl+a', 0x61, CONTROL],
    ['Left', KEY.Left, 0],
  ]) {
    it(`the commit arrives before the reply for ${name}, ${RUNS} times`, async () => {
      let inOrder = 0;
      for (let i = 0; i < RUNS; i++) {
        await app.type(':');
        await app.press(keyval, state);
        const types = app.last.map((e) => e.type);
        const commit = types.indexOf('commit');
        const reply = types.indexOf('reply');
        assert.ok(commit >= 0 && reply >= 0, `run ${i}: ${types}`);
        if (commit < reply) inOrder += 1;
      }
      console.log(`order: ${name}: commit before reply in ${inOrder} of ${RUNS} runs`);
      assert.equal(inOrder, RUNS);
    });
  }
});
