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
 */
import { spawn } from 'node:child_process';
import { existsSync, mkdtempSync, readdirSync, rmSync, statSync } from 'node:fs';
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

/** The environment of the private session: IBus and XDG paths in `run`. */
export function privateEnv(run, env = process.env) {
  const out = { ...env };
  delete out.IBUS_ADDRESS;
  return {
    ...out,
    TORPEDO_E2E_RUN: run,
    TORPEDO_E2E_OUTER_BUS: env.DBUS_SESSION_BUS_ADDRESS ?? '',
    TORPEDO_ENGINE: ENGINE,
    IBUS_ADDRESS_FILE: join(run, 'addr'),
    XDG_CONFIG_HOME: join(run, 'config'),
    XDG_CACHE_HOME: join(run, 'cache'),
    XDG_DATA_HOME: join(run, 'data'),
    GSETTINGS_BACKEND: 'memory',
    GIO_USE_VFS: 'local',
  };
}

export class Session {
  constructor(child, run, lines, before) {
    this.child = child;
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
export async function openSession() {
  if (!existsSync(ENGINE)) throw new Error(`no engine binary at ${ENGINE}. Run node scripts/e2e.mjs.`);
  const before = liveBusListing();
  const run = mkdtempSync('/tmp/tpd-');
  const child = spawn(
    'dbus-run-session',
    ['--config-file', join(HERE, 'session.conf'), '--', 'python3', '-I', join(HERE, 'driver.py')],
    { env: privateEnv(run), stdio: ['pipe', 'pipe', 'pipe'], detached: true },
  );
  const lines = createInterface({ input: child.stdout });
  const session = new Session(child, run, lines, before);
  const ready = await session.next();
  if (!ready.ready) throw new Error(`driver did not start: ${session.stderr}`);
  session.address = ready.address;
  return session;
}
