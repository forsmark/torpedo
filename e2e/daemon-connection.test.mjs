/**
 * Scenarios of the capability daemon-connection: how the engine finds the
 * daemon, when it connects, and when it stops.
 *
 * These tests start the engine by hand through the driver, with the
 * environment each scenario needs. The daemon itself always runs with
 * IBUS_ADDRESS_FILE and private XDG folders.
 */
import assert from 'node:assert/strict';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { after, before, describe, it } from 'node:test';
import { openSession } from './harness/session.mjs';

const NAME = 'org.freedesktop.IBus.Torpedo';
const ALL = ['IBUS_ADDRESS', 'IBUS_ADDRESS_FILE'];

function machineId() {
  const file = ['/var/lib/dbus/machine-id', '/etc/machine-id'].find((p) => existsSync(p));
  return readFileSync(file, 'utf8').trim();
}

describe('daemon-connection', () => {
  let session;
  before(async () => {
    session = await openSession();
  });
  after(async () => {
    await session?.close();
  });

  /** Start the engine with `args` and `env`, and report whether it registered. */
  async function start(args, env, unset) {
    const { index } = await session.send({ cmd: 'spawn_engine', args, env, unset });
    const { owned } = await session.send({ cmd: 'has_owner', name: NAME, timeout: 5 });
    const stopped = await session.send({ cmd: 'stop_spawned', index });
    assert.ok(stopped.released, 'the engine released its name');
    return owned;
  }

  it('Started with --ibus', async () => {
    assert.equal(await start(['--ibus'], { IBUS_ADDRESS: session.address }, []), true);
  });

  it('Started without --ibus', async () => {
    const { index } = await session.send({
      cmd: 'spawn_engine',
      args: [],
      env: { IBUS_ADDRESS: session.address },
      unset: [],
    });
    const { exit } = await session.send({ cmd: 'wait_spawned', index, timeout: 5 });
    assert.equal(exit, 0);
    const { owned } = await session.send({ cmd: 'has_owner', name: NAME, timeout: 0 });
    assert.equal(owned, false);
  });

  it('The address in IBUS_ADDRESS', async () => {
    const wrong = join(session.run, 'wrong-addr');
    writeFileSync(wrong, 'IBUS_ADDRESS=unix:path=/tmp/tpd-no-such-daemon/bus,guid=0\n');
    const env = { IBUS_ADDRESS: session.address, IBUS_ADDRESS_FILE: wrong };
    assert.equal(await start(['--ibus'], env, []), true);
  });

  it('The address file in IBUS_ADDRESS_FILE', async () => {
    const env = { IBUS_ADDRESS_FILE: join(session.run, 'addr') };
    assert.equal(await start(['--ibus'], env, ['IBUS_ADDRESS']), true);
  });

  it('The default address file', async () => {
    const config = join(session.run, 'default-config');
    const display = 'torpedo-e2e';
    const dir = join(config, 'ibus', 'bus');
    mkdirSync(dir, { recursive: true });
    writeFileSync(join(dir, `${machineId()}-unix-${display}`), readFileSync(join(session.run, 'addr')));
    const env = { XDG_CONFIG_HOME: config, WAYLAND_DISPLAY: display };
    assert.ok(env.XDG_CONFIG_HOME.startsWith(session.run), 'the engine reads only a private folder');
    assert.equal(await start(['--ibus'], env, ALL), true);
  });
});

describe('daemon-connection when the daemon stops', () => {
  it('The daemon stops', async () => {
    const session = await openSession();
    try {
      await session.context();
      const { pids } = await session.send({ cmd: 'engine_pids' });
      assert.equal(pids.length, 1, 'the daemon started one engine');
      const { alive } = await session.send({ cmd: 'stop_daemon', timeout: 5 });
      assert.deepEqual(alive, []);
    } finally {
      await session.close();
    }
  });
});
