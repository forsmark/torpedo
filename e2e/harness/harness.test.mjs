/**
 * The harness itself: a private session starts, selects torpedo, and stops
 * without touching the live IBus session.
 */
import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { liveBusListing, openSession, privateEnv } from './session.mjs';

describe('the private IBus session', () => {
  it('keeps every IBus and XDG path in its run folder', () => {
    const env = privateEnv('/tmp/tpd-x', { IBUS_ADDRESS: 'unix:path=/live', HOME: '/home/someone' });
    assert.equal(env.IBUS_ADDRESS, undefined);
    for (const k of ['IBUS_ADDRESS_FILE', 'XDG_CONFIG_HOME', 'XDG_CACHE_HOME', 'XDG_DATA_HOME']) {
      assert.ok(env[k].startsWith('/tmp/tpd-x/'), k);
    }
  });

  it('selects torpedo and stops, leaving the live address folder alone', async () => {
    const before = liveBusListing();
    const session = await openSession();
    try {
      assert.ok(session.address.includes('/tmp/tpd-'));
      const app = await session.context();
      assert.ok(app.needsSurroundingText);
    } finally {
      await session.close();
    }
    assert.equal(liveBusListing(), before);
  });
});
