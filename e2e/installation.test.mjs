/**
 * Scenarios of the capability installation.
 *
 * Every run puts HOME in a temp folder, and stubs for ibus, gsettings, dconf
 * and sudo first on the PATH. Each stub records its call. The sudo stub runs
 * its command only when a test allows it. A real run always sets PREFIX,
 * COMPONENT_DIR and SUDO inside the temp folder. Only a --dry-run may leave
 * them at their defaults, so no test ever writes to /usr or the real ~/.local.
 */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { chmodSync, existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { after, before, describe, it } from 'node:test';
import { ROOT } from './harness/session.mjs';

const STUBS = ['ibus', 'gsettings', 'dconf', 'sudo'];

/** A temp folder with a HOME and recording stubs. */
function sandbox() {
  const dir = mkdtempSync('/tmp/tpd-install-');
  const bin = join(dir, 'stubs');
  mkdirSync(bin);
  mkdirSync(join(dir, 'home'));
  const log = join(dir, 'calls.log');
  for (const name of STUBS) {
    const run = name === 'sudo' ? '[ "${TORPEDO_TEST_SUDO_RUNS:-}" = 1 ] && exec "$@"\nexit 0\n' : 'exit 1\n';
    writeFileSync(join(bin, name), `#!/bin/sh\necho "${name} $*" >> "${log}"\n${run}`);
    chmodSync(join(bin, name), 0o755);
  }
  return { dir, bin, log, home: join(dir, 'home') };
}

function calls(box) {
  return existsSync(box.log) ? readFileSync(box.log, 'utf8').trim().split('\n') : [];
}

/** Run a script in the sandbox. Without --dry-run, all three overrides must point into it. */
function runScript(box, script, { args = [], env = {} } = {}) {
  if (!args.includes('--dry-run')) {
    for (const k of ['PREFIX', 'COMPONENT_DIR', 'SUDO']) {
      assert.ok(env[k]?.startsWith(box.dir), `SAFETY: ${k} must point into ${box.dir}`);
    }
  }
  const real = homedir();
  const full = {
    PATH: `${box.bin}:${join(real, '.cargo', 'bin')}:${process.env.PATH}`,
    HOME: box.home,
    CARGO_HOME: process.env.CARGO_HOME ?? join(real, '.cargo'),
    RUSTUP_HOME: process.env.RUSTUP_HOME ?? join(real, '.rustup'),
    ...env,
  };
  const result = spawnSync('sh', [join(ROOT, script), ...args], { cwd: box.dir, env: full, encoding: 'utf8' });
  assert.equal(result.status, 0, `${script} failed:\n${result.stdout}\n${result.stderr}`);
  return result.stdout;
}

function overrides(box) {
  return {
    PREFIX: join(box.dir, 'prefix'),
    COMPONENT_DIR: join(box.dir, 'component'),
    SUDO: join(box.bin, 'sudo'),
    TORPEDO_TEST_SUDO_RUNS: '1',
  };
}

describe('installation', () => {
  let box;
  let env;
  let installOut;
  let uninstallOut;
  let installCalls;
  let installed;
  let xml;

  before(() => {
    box = sandbox();
    env = overrides(box);
    installOut = runScript(box, 'install.sh', { env });
    installCalls = calls(box);
    const binary = join(env.PREFIX, 'lib', 'torpedo', 'ibus-engine-torpedo');
    installed = { binary, exists: existsSync(binary), mode: existsSync(binary) ? statSync(binary).mode : 0 };
    xml = readFileSync(join(env.COMPONENT_DIR, 'torpedo.xml'), 'utf8');
    rmSync(box.log, { force: true });
    uninstallOut = runScript(box, 'uninstall.sh', { env });
  });
  after(() => rmSync(box.dir, { recursive: true, force: true }));

  it('The binary is installed', () => {
    assert.ok(installed.exists);
    assert.ok(installed.mode & 0o100, 'executable');
  });

  it('The component file', () => {
    assert.match(xml, new RegExp(`<exec>${installed.binary} --ibus</exec>`));
    assert.ok(installed.binary.startsWith('/'));
    assert.match(xml, /<engine>\s*<name>torpedo<\/name>/);
    assert.match(xml, /<layout>dk<\/layout>/);
    assert.equal(xml.match(/<engine>/g).length, 1);
  });

  it('The component file goes through sudo', () => {
    const sudo = installCalls.filter((c) => c.startsWith('sudo '));
    assert.equal(sudo.length, 1);
    assert.match(sudo[0], new RegExp(`^sudo install -D -m 644 \\S+ ${env.COMPONENT_DIR}/torpedo.xml$`));
  });

  it('Install runs no ibus command', () => {
    assert.deepEqual(installCalls.filter((c) => c.startsWith('ibus')), []);
  });

  it('Install changes no setting', () => {
    assert.deepEqual(installCalls.filter((c) => /^(gsettings|dconf)/.test(c)), []);
  });

  it('The next steps after install', () => {
    const tail = installOut.trim().split('\n').slice(-6).join('\n');
    assert.match(tail, /ibus restart/);
    assert.match(tail, /GNOME Settings.*add Torpedo/);
    assert.match(tail, /Keep Danish .* fallback/);
    assert.ok(tail.indexOf('ibus restart') < tail.indexOf('GNOME Settings'));
  });

  it('Uninstall runs no ibus command', () => {
    assert.deepEqual(calls(box).filter((c) => c.startsWith('ibus')), []);
  });

  it('Uninstall changes no setting', () => {
    assert.deepEqual(calls(box).filter((c) => /^(gsettings|dconf)/.test(c)), []);
  });

  it('Uninstall after install', () => {
    assert.equal(existsSync(join(env.PREFIX, 'lib', 'torpedo')), false);
    assert.equal(existsSync(join(env.COMPONENT_DIR, 'torpedo.xml')), false);
    assert.ok(calls(box).some((c) => c.startsWith('sudo rm -f ')), 'the component file goes through sudo');
  });

  it('The next steps after uninstall', () => {
    assert.match(uninstallOut, /remove Torpedo from the input sources/);
    assert.match(uninstallOut, /ibus restart/);
  });
});

describe('installation defaults', () => {
  let box;
  before(() => {
    box = sandbox();
  });
  after(() => rmSync(box.dir, { recursive: true, force: true }));

  it('The default prefix', () => {
    const out = runScript(box, 'install.sh', { args: ['--dry-run'] });
    assert.match(out, new RegExp(`install -D -m 755 \\S+ ${box.home}/\\.local/lib/torpedo/ibus-engine-torpedo`));
    assert.deepEqual(readdirSync(box.home), []);
  });

  it('The default component folder', () => {
    const existed = existsSync('/usr/share/ibus/component/torpedo.xml');
    const out = runScript(box, 'install.sh', { args: ['--dry-run'] });
    assert.match(out, /^\+ sudo install -D -m 644 .* \/usr\/share\/ibus\/component\/torpedo\.xml$/m);
    assert.deepEqual(readdirSync(box.home), []);
    assert.deepEqual(calls(box), []);
    assert.equal(existsSync('/usr/share/ibus/component/torpedo.xml'), existed);
  });
});
