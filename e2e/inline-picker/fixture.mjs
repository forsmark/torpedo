/**
 * One private session per test file, and a new input context per test. The
 * engine starts each test with nothing held, because a new context takes
 * the focus. Each test also starts with no usage: the fixture deletes the
 * usage file, and the engine reads that on the focus-in of the new context.
 */
import assert from 'node:assert/strict';
import { after, before, beforeEach } from 'node:test';
import { openSession } from '../harness/session.mjs';

export function usePicker(options = {}) {
  const t = {};
  before(async () => {
    t.session = await openSession(options);
  });
  after(async () => {
    await t.session?.close();
  });
  beforeEach(async () => {
    t.session.removeUsage();
    t.app = await t.session.context();
  });
  return t;
}

/** In the last events, the commit of `text` came before the reply. */
export function commitBeforeReply(app, text) {
  const types = app.last.map((e) => (e.type === 'commit' ? `commit:${e.text}` : e.type));
  const commit = types.indexOf(`commit:${text}`);
  const reply = types.indexOf('reply');
  assert.ok(commit >= 0 && reply > commit, `expected the commit of ${text} before the reply: ${types}`);
}

/** The emoji of a candidate text such as "🔥 fire". */
export function emojiOf(candidate) {
  return candidate.slice(0, candidate.indexOf(' '));
}
