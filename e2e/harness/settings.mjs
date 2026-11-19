/**
 * Run torpedo-settings in its mode without a window.
 *
 * Every run gets a private HOME, XDG_CONFIG_HOME and XDG_STATE_HOME in its sandbox, and no
 * DISPLAY or WAYLAND_DISPLAY, so it can neither touch the owner's config file
 * nor open a window on the owner's display. runSettings refuses an action-less
 * run, which would open the window.
 */
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { ROOT } from './session.mjs';

/** The settings binary that scripts/e2e.mjs builds. TORPEDO_SETTINGS_BINARY overrides it. */
export const SETTINGS = process.env.TORPEDO_SETTINGS_BINARY ?? join(ROOT, 'target', 'release', 'torpedo-settings');

const ACTIONS = [
  '--check',
  '--set-trigger',
  '--set-alias',
  '--remove-alias',
  '--set-skin-tone',
  '--set-remember-usage',
  '--clear-usage',
  '--help',
  '--version',
];

/** A temp folder with a HOME, an XDG_CONFIG_HOME and an XDG_STATE_HOME. */
export function sandbox() {
  const dir = mkdtempSync('/tmp/tpd-settings-');
  const home = join(dir, 'home');
  const xdg = join(dir, 'config');
  const state = join(dir, 'state');
  mkdirSync(home);
  return {
    dir,
    home,
    xdg,
    state,
    config: join(xdg, 'torpedo', 'config.toml'),
    usage: join(state, 'torpedo', 'usage.toml'),
    remove: () => rmSync(dir, { recursive: true, force: true }),
  };
}

/** The environment of a run: private paths, no display. `extra` null removes a variable. */
export function settingsEnv(box, extra = {}) {
  const env = {
    PATH: process.env.PATH,
    HOME: box.home,
    XDG_CONFIG_HOME: box.xdg,
    XDG_STATE_HOME: box.state,
    LANG: 'C.UTF-8',
  };
  for (const [k, v] of Object.entries(extra)) {
    if (v === null) delete env[k];
    else env[k] = v;
  }
  assert.ok(!env.DISPLAY && !env.WAYLAND_DISPLAY, 'SAFETY: torpedo-settings runs with no display');
  assert.ok(env.HOME.startsWith(box.dir), 'SAFETY: HOME is private');
  assert.ok((env.XDG_STATE_HOME ?? env.HOME).startsWith(box.dir), 'SAFETY: the usage file is private');
  return env;
}

function guard(args) {
  assert.ok(args.some((a) => ACTIONS.includes(a)), `SAFETY: torpedo-settings without an action opens a window: ${args}`);
}

/** Run torpedo-settings and return its status and output. */
export function runSettings(box, args, { env = {}, cwd } = {}) {
  guard(args);
  const r = spawnSync(SETTINGS, args, { env: settingsEnv(box, env), cwd: cwd ?? box.dir, encoding: 'utf8' });
  if (r.error) throw r.error;
  return { status: r.status, out: r.stdout, err: r.stderr };
}
