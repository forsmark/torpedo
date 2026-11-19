/**
 * config-file: what the engine does with a missing, broken, bad or changed
 * config file. Each test opens its own private session, because the engine
 * reads the file when it starts.
 */
import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { KEY } from '../harness/app.mjs';
import { openSession } from '../harness/session.mjs';

/** Run `fn(session, app)` in a private session with this config file. */
async function withSession(opts, fn) {
  const session = await openSession(opts);
  try {
    const app = await session.context();
    await fn(session, app);
  } finally {
    await session.close();
  }
}

describe('config-file: the engine', () => {
  it('The config file under XDG_CONFIG_HOME', () =>
    withSession({ config: 'trigger = ";;"\n' }, async (_s, app) => {
      await app.type(';;fire:');
      assert.equal(app.text, '🔥');
    }));

  it('No config file', () =>
    withSession({}, async (_s, app) => {
      await app.type('::fire:');
      assert.equal(app.text, '🔥');
    }));

  it('A broken file at start', () =>
    withSession({ config: 'trigger = ";;\n' }, async (_s, app) => {
      await app.type('::fire:');
      assert.equal(app.text, '🔥');
    }));

  it('A bad value at start', () =>
    withSession({ config: 'trigger = ";;"\n[aliases]\nlgtm = ""\n' }, async (_s, app) => {
      await app.type('::fire:');
      assert.equal(app.text, '🔥');
    }));

  it('A broken file after a good one', () =>
    withSession({ config: '[aliases]\nlgtm = "👍"\n' }, async (session, app) => {
      session.writeConfig('trigger = "\n');
      await app.type('::lgtm:');
      assert.equal(app.text, '👍');
    }));

  it('The log of a bad trigger', () =>
    withSession({ config: 'trigger = "ab"\n', env: { TORPEDO_DEBUG: '1' } }, async (session) => {
      const log = session.engineLog();
      const line = log.split('\n').find((l) => l.includes(`config ${session.configPath}:`) && l.includes('trigger: '));
      assert.ok(line, `no log line names the config file and trigger:\n${log}`);
    }));

  it('A key from a later version', () =>
    withSession({ config: 'theme = "dark"\ntrigger = ";;"\n\n[later]\nlast = "🔥"\n' }, async (_s, app) => {
      await app.type(';;fire:');
      assert.equal(app.text, '🔥');
    }));

  it('No skin_tone key', () =>
    withSession({ config: 'trigger = "::"\n' }, async (_s, app) => {
      await app.type('::wave:');
      assert.equal(app.text, '👋');
    }));

  it('No remember_usage key', () =>
    withSession({ config: 'trigger = "::"\n' }, async (session, app) => {
      assert.equal(session.readUsage(), null);
      await app.type('::fire:');
      assert.ok(session.usageOf('🔥'));
    }));

  it('A new alias when a search opens', () =>
    withSession({ config: 'trigger = "::"\n' }, async (session, app) => {
      session.writeConfig('trigger = "::"\n\n[aliases]\nlgtm = "👍"\n');
      await app.type('::lgtm:');
      assert.equal(app.text, '👍');
    }));

  it('A new trigger after focus-in', () =>
    withSession({ config: 'trigger = "::"\n' }, async (session, app) => {
      session.writeConfig('trigger = ";;"\n');
      await app.focusOut();
      await app.focusIn();
      await app.type(';;fire:');
      assert.equal(app.text, '🔥');
    }));

  it('A deleted config file', () =>
    withSession({ config: 'trigger = ";;"\n' }, async (session, app) => {
      await app.type(';;fire:');
      assert.equal(app.text, '🔥');
      session.removeConfig();
      await app.focusOut();
      await app.focusIn();
      await app.type(' ::fire:');
      assert.equal(app.text, '🔥 🔥');
    }));

  it('The trigger changes as a search opens', () =>
    withSession({ config: 'trigger = "::"\n' }, async (session, app) => {
      session.writeConfig('trigger = ";;"\n');
      await app.type('::');
      assert.equal(app.preedit.text, '::');
      await app.press(KEY.Escape);
      assert.equal(app.text, '::');
    }));

  it('The new trigger after the search', () =>
    withSession({ config: 'trigger = "::"\n' }, async (session, app) => {
      session.writeConfig('trigger = ";;"\n');
      await app.type('::');
      await app.press(KEY.Escape);
      await app.type(' ;;fire:');
      assert.equal(app.text, ':: 🔥');
    }));
});
