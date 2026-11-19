/**
 * The harness itself: a private session starts, selects torpedo, and stops
 * without touching the live IBus session.
 */
import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { existsSync, readFileSync } from 'node:fs';
import { liveBusListing, openSession, privateEnv, usageFileOf } from './session.mjs';
import { sandbox, settingsEnv } from './settings.mjs';

describe('the private IBus session', () => {
  it('keeps every IBus and XDG path in its run folder', () => {
    const env = privateEnv('/tmp/tpd-x', { IBUS_ADDRESS: 'unix:path=/live', HOME: '/home/someone' });
    assert.equal(env.IBUS_ADDRESS, undefined);
    for (const k of ['IBUS_ADDRESS_FILE', 'XDG_CONFIG_HOME', 'XDG_CACHE_HOME', 'XDG_DATA_HOME', 'XDG_STATE_HOME']) {
      assert.ok(env[k].startsWith('/tmp/tpd-x/'), k);
    }
  });

  it('refuses a usage file outside the run folder', () => {
    assert.throws(() => privateEnv('/tmp/tpd-x', { HOME: '/home/someone' }, { XDG_STATE_HOME: '/home/someone/.local/state' }), /SAFETY/);
    assert.throws(() => privateEnv('/tmp/tpd-x', { HOME: '/home/someone' }, { XDG_STATE_HOME: null }), /SAFETY/);
    assert.throws(() => privateEnv('/tmp/tpd-x', { HOME: '/home/someone' }, { XDG_STATE_HOME: 'relative' }), /SAFETY/);
    const env = privateEnv('/tmp/tpd-x', { HOME: '/home/someone' }, { XDG_STATE_HOME: null, HOME: '/tmp/tpd-x/home' });
    assert.equal(usageFileOf(env), '/tmp/tpd-x/home/.local/state/torpedo/usage.toml');
    assert.equal(usageFileOf(privateEnv('/tmp/tpd-x', {})), '/tmp/tpd-x/state/torpedo/usage.toml');
  });

  it('gives each settings run a private state folder', () => {
    const box = sandbox();
    try {
      assert.equal(settingsEnv(box).XDG_STATE_HOME, box.state);
      assert.throws(() => settingsEnv(box, { XDG_STATE_HOME: '/home/someone/.local/state' }), /SAFETY/);
    } finally {
      box.remove();
    }
  });

  it('writes, reads and removes a private usage file', async () => {
    const session = await openSession();
    try {
      assert.ok(session.usagePath.startsWith(`${session.run}/`));
      assert.equal(session.readUsage(), null);
      session.writeUsage([{ text: '👍', score: 2, age: 60 }, { text: '¯\\_(ツ)_/¯' }]);
      const entries = session.readUsage();
      assert.equal(entries.length, 2);
      assert.equal(session.usageOf('👍').score, 2);
      assert.ok(Math.abs(session.usageOf('👍').last - (Date.now() / 1000 - 60)) < 5);
      assert.equal(entries[1].text, '¯\\_(ツ)_/¯');
      session.removeUsage();
      assert.equal(session.readUsage(), null);
    } finally {
      await session.close();
    }
  });

  it('removes the display and stubs the settings command', () => {
    const env = privateEnv('/tmp/tpd-x', { DISPLAY: ':0', WAYLAND_DISPLAY: 'wayland-0', TORPEDO_SETTINGS_COMMAND: '/real' });
    assert.equal(env.DISPLAY, undefined);
    assert.equal(env.WAYLAND_DISPLAY, undefined);
    assert.equal(env.TORPEDO_SETTINGS_COMMAND, '/tmp/tpd-x/settings-stub');
    const extra = privateEnv('/tmp/tpd-x', {}, { TORPEDO_SETTINGS_COMMAND: null, TORPEDO_DEBUG: '1' });
    assert.equal('TORPEDO_SETTINGS_COMMAND' in extra, false);
    assert.equal(extra.TORPEDO_DEBUG, '1');
  });

  it('writes a private config file, and records properties and settings starts', async () => {
    const session = await openSession({ config: 'trigger = ";;"\n', env: { TORPEDO_DEBUG: '1' } });
    try {
      assert.ok(session.configPath.startsWith(session.run));
      assert.equal(readFileSync(session.configPath, 'utf8'), 'trigger = ";;"\n');
      const app = await session.context();
      assert.ok(app.log.some((e) => e.type === 'properties'));
      session.writeConfig('trigger = ":"\n');
      assert.equal(readFileSync(session.configPath, 'utf8'), 'trigger = ":"\n');
      session.removeConfig();
      assert.equal(existsSync(session.configPath), false);
      assert.equal(session.settingsStarts(), 0);
      await session.propertyActivate('torpedo-settings');
      const deadline = Date.now() + 3000;
      while (session.settingsStarts() === 0 && Date.now() < deadline) await new Promise((r) => setTimeout(r, 20));
      assert.equal(session.settingsStarts(), 1);
      assert.match(session.engineLog(), /\[torpedo\] config /);
    } finally {
      await session.close();
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
