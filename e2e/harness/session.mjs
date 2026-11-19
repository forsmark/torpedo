/**
 * A private IBus session for the e2e tests.
 *
 *   const session = await openSession();
 *   const ic = await session.context();
 *   await ic.type('::fire');
 *   await session.close();
 *
 * openSession starts `dbus-run-session` with e2e/harness/session.conf and runs
 * driver.py inside it. The driver starts a private ibus-daemon and talks to it
 * as an IBus client. Every IBus and XDG path points into a fresh folder under
 * /tmp, and IBUS_ADDRESS is removed, so nothing reaches the live session.
 *
 * The session runs in its own process group, and Node never signals it. close()
 * only ends the driver's stdin. The driver then stops the daemon with SIGTERM
 * and waits for it, before dbus-run-session ends the bus. If Node dies, the
 * driver sees the end of stdin and does the same.
 *
 * close() also fails when ~/.config/ibus/bus/ changed while the session ran.
 *
 * Usage goes to a private XDG_STATE_HOME in the run folder. privateEnv refuses
 * an environment that would let the engine write the owner's usage file.
 *
 * openSession({ config, env, engine }) takes the text of a private config
 * file, more variables for the daemon and the engine (null removes one), and
 * another engine binary. Every session removes DISPLAY and WAYLAND_DISPLAY,
 * and points TORPEDO_SETTINGS_COMMAND at a stub that records each start in
 * <run>/settings-starts, so no test can open a window.
 */
import { spawn, spawnSync } from 'node:child_process';
import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  renameSync,
  rmSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import { homedir } from 'node:os';
import { dirname, join } from 'node:path';
import { createInterface } from 'node:readline';
import { fileURLToPath } from 'node:url';
import { App } from './app.mjs';

const HERE = dirname(fileURLToPath(import.meta.url));
export const ROOT = join(HERE, '..', '..');
/** The engine binary that scripts/e2e.mjs builds. TORPEDO_ENGINE overrides it. */
export const ENGINE = process.env.TORPEDO_ENGINE ?? join(ROOT, 'target', 'release', 'torpedo-engine');
const LIVE_BUS_DIR = join(homedir(), '.config', 'ibus', 'bus');

/** A listing of the live address folder: names, sizes and change times. */
export function liveBusListing(dir = LIVE_BUS_DIR) {
  if (!existsSync(dir)) return '';
  return readdirSync(dir)
    .sort()
    .map((name) => {
      const s = statSync(join(dir, name));
      return `${name} ${s.size} ${s.mtimeMs}`;
    })
    .join('\n');
}

/** The stub settings command of a run, and the file it records starts in. */
export const settingsStub = (run) => join(run, 'settings-stub');
export const settingsStartsFile = (run) => join(run, 'settings-starts');

/** The config file of a run. */
export const configFile = (run) => join(run, 'config', 'torpedo', 'config.toml');

/** The usage file that the engine of an environment uses. */
export function usageFileOf(env) {
  const state = env.XDG_STATE_HOME?.startsWith('/') ? env.XDG_STATE_HOME : join(env.HOME, '.local', 'state');
  return join(state, 'torpedo', 'usage.toml');
}

/** The text of a usage file. Each entry is { text, score, age }, with age in seconds before now. */
export function usageText(entries, now = Math.floor(Date.now() / 1000)) {
  return entries
    .map(({ text, score = 1, age = 0 }) => `[[used]]\ntext = ${JSON.stringify(text)}\nscore = ${Number(score).toFixed(6)}\nlast = ${now - age}\n`)
    .join('\n');
}

/** Parse a usage file with Python's tomllib. Null when it does not exist. */
export function readTomlFile(path) {
  if (!existsSync(path)) return null;
  const r = spawnSync(
    'python3',
    ['-I', '-c', 'import json,sys,tomllib; print(json.dumps(tomllib.load(open(sys.argv[1], "rb"))))', path],
    { encoding: 'utf8' },
  );
  if (r.status !== 0) throw new Error(`${path} does not parse: ${r.stderr}`);
  return JSON.parse(r.stdout);
}

/**
 * The environment of the private session: IBus and XDG paths in `run`, no
 * display, and a stub settings command. `extra` adds variables, and a null
 * value removes one.
 */
export function privateEnv(run, env = process.env, extra = {}) {
  const out = { ...env };
  delete out.IBUS_ADDRESS;
  delete out.DISPLAY;
  delete out.WAYLAND_DISPLAY;
  const all = {
    ...out,
    TORPEDO_E2E_RUN: run,
    TORPEDO_E2E_OUTER_BUS: env.DBUS_SESSION_BUS_ADDRESS ?? '',
    TORPEDO_ENGINE: ENGINE,
    TORPEDO_SETTINGS_COMMAND: settingsStub(run),
    IBUS_ADDRESS_FILE: join(run, 'addr'),
    XDG_CONFIG_HOME: join(run, 'config'),
    XDG_STATE_HOME: join(run, 'state'),
    XDG_CACHE_HOME: join(run, 'cache'),
    XDG_DATA_HOME: join(run, 'data'),
    GSETTINGS_BACKEND: 'memory',
    GIO_USE_VFS: 'local',
  };
  for (const [k, v] of Object.entries(extra)) {
    if (v === null) delete all[k];
    else all[k] = v;
  }
  const inRun = (p) => typeof p === 'string' && p.startsWith(`${run}/`);
  if (all.XDG_STATE_HOME !== undefined ? !inRun(all.XDG_STATE_HOME) : !inRun(all.HOME)) {
    throw new Error(`SAFETY: the usage file ${usageFileOf(all)} is outside the run folder ${run}`);
  }
  return all;
}

/** A script that appends a line to `log` each time it runs, and exits. */
export function recordingStub(path, log) {
  writeFileSync(path, `#!/bin/sh\necho started >> '${log}'\n`);
  chmodSync(path, 0o755);
}

/** Write a file with a temp file and a rename, as the settings window does. */
export function writeAtomically(path, text) {
  mkdirSync(dirname(path), { recursive: true });
  const tmp = `${path}.e2e-tmp`;
  writeFileSync(tmp, text);
  renameSync(tmp, path);
}

export class Session {
  constructor(child, run, lines, before, env = {}) {
    this.child = child;
    this.env = env;
    this.run = run;
    this.lines = lines;
    this.before = before;
    this.waiting = [];
    this.buffered = [];
    this.stderr = '';
    lines.on('line', (line) => {
      const next = this.waiting.shift();
      if (next) next.resolve(JSON.parse(line));
      else this.buffered.push(JSON.parse(line));
    });
    child.stderr.on('data', (d) => {
      this.stderr += d;
    });
    child.on('exit', () => {
      for (const w of this.waiting.splice(0)) w.reject(new Error(`driver exited: ${this.stderr}`));
    });
  }

  /** The next line from the driver. */
  next() {
    if (this.buffered.length) return Promise.resolve(this.buffered.shift());
    return new Promise((resolve, reject) => this.waiting.push({ resolve, reject }));
  }

  /** Send one command and return the driver's reply. */
  async send(cmd) {
    this.child.stdin.write(`${JSON.stringify(cmd)}\n`);
    const reply = await this.next();
    if (reply.ok === false) throw new Error(`driver: ${reply.error}`);
    return reply;
  }

  /** A new input context, focused and switched to torpedo. */
  async context(name) {
    const reply = await this.send({ cmd: 'context', name });
    if (!reply.ok) throw new Error('the input context did not get the torpedo engine');
    return new App(this, reply);
  }

  /** The private config file of this session. */
  get configPath() {
    return configFile(this.run);
  }

  /** Replace the config file in one step. */
  writeConfig(text) {
    writeAtomically(this.configPath, text);
  }

  /** Delete the config file. */
  removeConfig() {
    rmSync(this.configPath, { force: true });
  }

  /** The private usage file of this session. */
  get usagePath() {
    return usageFileOf(this.env);
  }

  /** Replace the usage file in one step: entries as usageText takes them, or a raw text. */
  writeUsage(entries) {
    writeAtomically(this.usagePath, typeof entries === 'string' ? entries : usageText(entries));
  }

  /** The entries of the usage file, as { text, score, last }, or null when it does not exist. */
  readUsage() {
    const doc = readTomlFile(this.usagePath);
    return doc === null ? null : (doc.used ?? []);
  }

  /** The entry of `text` in the usage file, or undefined. */
  usageOf(text) {
    return (this.readUsage() ?? []).find((e) => e.text === text);
  }

  /** Delete the usage file. */
  removeUsage() {
    rmSync(this.usagePath, { force: true });
  }

  /** How often the stub settings command started. */
  settingsStarts() {
    const f = settingsStartsFile(this.run);
    return existsSync(f) ? readFileSync(f, 'utf8').trim().split('\n').length : 0;
  }

  /** The daemon's log, which holds the engine's stderr. */
  engineLog() {
    const f = join(this.run, 'daemon.log');
    return existsSync(f) ? readFileSync(f, 'utf8') : '';
  }

  /** The panel activates a property, as a click in the input source menu. */
  propertyActivate(name, state = 0) {
    return this.send({ cmd: 'property_activate', name, state });
  }

  /** Stop the driver, wait for it, and check the live address folder. */
  async close() {
    if (this.child.exitCode === null) {
      const exited = new Promise((resolve) => this.child.once('exit', resolve));
      this.child.stdin.end(`${JSON.stringify({ cmd: 'stop' })}\n`);
      await exited;
    }
    const after = liveBusListing();
    rmSync(this.run, { recursive: true, force: true });
    if (after !== this.before) {
      throw new Error(`SAFETY: ${LIVE_BUS_DIR} changed during the session:\n${this.before}\n---\n${after}`);
    }
  }
}

/** Start a private session. */
export async function openSession({ config, env = {}, engine } = {}) {
  const binary = engine ?? ENGINE;
  if (!existsSync(binary)) throw new Error(`no engine binary at ${binary}. Run node scripts/e2e.mjs.`);
  const before = liveBusListing();
  const run = mkdtempSync('/tmp/tpd-');
  recordingStub(settingsStub(run), settingsStartsFile(run));
  if (config !== undefined) writeAtomically(configFile(run), config);
  const extra = engine ? { TORPEDO_ENGINE: engine, ...env } : env;
  let sessionEnv;
  try {
    sessionEnv = privateEnv(run, process.env, extra);
  } catch (e) {
    rmSync(run, { recursive: true, force: true });
    throw e;
  }
  const child = spawn(
    'dbus-run-session',
    ['--config-file', join(HERE, 'session.conf'), '--', 'python3', '-I', join(HERE, 'driver.py')],
    { env: sessionEnv, stdio: ['pipe', 'pipe', 'pipe'], detached: true },
  );
  const lines = createInterface({ input: child.stdout });
  const session = new Session(child, run, lines, before, sessionEnv);
  const ready = await session.next();
  if (!ready.ready) throw new Error(`driver did not start: ${session.stderr}`);
  session.address = ready.address;
  return session;
}
