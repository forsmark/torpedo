/**
 * settings-window: the input source menu. The harness panel stands in for
 * GNOME Shell. Every settings command here is a stub that records each start,
 * so no window opens.
 */
import assert from 'node:assert/strict';
import { chmodSync, copyFileSync, existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, it } from 'node:test';
import { ENGINE, openSession, recordingStub } from '../harness/session.mjs';

async function waitFor(pred, ms = 3000) {
  const deadline = Date.now() + ms;
  while (!pred() && Date.now() < deadline) await new Promise((r) => setTimeout(r, 20));
  return pred();
}

async function withSession(opts, fn) {
  const session = await openSession(opts);
  try {
    await fn(session, await session.context());
  } finally {
    await session.close();
  }
}

describe('settings-window: the input source menu', () => {
  it('The menu item', () =>
    withSession({}, async (_s, app) => {
      const lists = app.log.filter((e) => e.type === 'properties');
      assert.ok(lists.length >= 1, 'the panel got a property list on focus-in');
      const items = lists.at(-1).items;
      assert.equal(items.length, 1);
      assert.equal(items[0].label, 'Torpedo settings');
      assert.equal(items[0].key, 'torpedo-settings');
      assert.equal(items[0].type, 0);
      assert.equal(items[0].visible, true);
      assert.equal(items[0].sensitive, true);
    }));

  it('The menu item starts the settings command', () =>
    withSession({}, async (session) => {
      assert.equal(session.settingsStarts(), 0);
      await session.propertyActivate('torpedo-settings');
      assert.ok(await waitFor(() => session.settingsStarts() === 1));
      await new Promise((r) => setTimeout(r, 200));
      assert.equal(session.settingsStarts(), 1);
    }));

  it('The default settings command', async () => {
    const dir = mkdtempSync('/tmp/tpd-engine-');
    try {
      const engine = join(dir, 'ibus-engine-torpedo');
      copyFileSync(ENGINE, engine);
      const log = join(dir, 'starts');
      recordingStub(join(dir, 'torpedo-settings'), log);
      await withSession({ engine, env: { TORPEDO_SETTINGS_COMMAND: null } }, async (session) => {
        await session.propertyActivate('torpedo-settings');
        assert.ok(await waitFor(() => existsSync(log)), 'torpedo-settings next to the engine started');
        await new Promise((r) => setTimeout(r, 200));
        assert.equal(readFileSync(log, 'utf8').trim().split('\n').length, 1);
        assert.equal(session.settingsStarts(), 0);
      });
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it('Typing goes on while the settings command runs', async () => {
    const dir = mkdtempSync('/tmp/tpd-slow-');
    try {
      const stub = join(dir, 'slow');
      writeFileSync(stub, `#!/bin/sh\ntouch '${join(dir, 'started')}'\nexec sleep 3\n`);
      chmodSync(stub, 0o755);
      await withSession({ env: { TORPEDO_SETTINGS_COMMAND: stub } }, async (session, app) => {
        await session.propertyActivate('torpedo-settings');
        assert.ok(await waitFor(() => existsSync(join(dir, 'started'))));
        await app.type('::fire:');
        assert.equal(app.text, '🔥');
      });
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });
});
