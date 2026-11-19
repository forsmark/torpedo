/**
 * config-file: the location and the value rules, through
 * `torpedo-settings --check`, which uses the rules of torpedo-core that the
 * engine uses.
 */
import assert from 'node:assert/strict';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { after, before, describe, it } from 'node:test';
import { runSettings, sandbox } from '../harness/settings.mjs';

describe('config-file: rules', () => {
  let box;
  before(() => {
    box = sandbox();
  });
  after(() => box.remove());

  function check(text) {
    mkdirSync(dirname(box.config), { recursive: true });
    writeFileSync(box.config, text);
    return runSettings(box, ['--check']);
  }

  function good(text) {
    const r = check(text);
    assert.equal(r.status, 0, r.err);
    assert.match(r.out, /: ok\n$/);
  }

  function bad(text, key) {
    const r = check(text);
    assert.equal(r.status, 1, r.out);
    assert.ok(r.err.includes(`: ${key}: `), `expected the problem to name ${key}: ${r.err}`);
  }

  it('The default location', () => {
    const r = runSettings(box, ['--check'], { env: { XDG_CONFIG_HOME: null } });
    assert.equal(r.status, 0, r.err);
    assert.ok(r.out.startsWith(`${join(box.home, '.config', 'torpedo', 'config.toml')}: `), r.out);
  });

  it('Two semicolons', () => good('trigger = ";;"\n'));
  it('One colon', () => good('trigger = ":"\n'));
  it('A trigger with a letter', () => bad('trigger = ":a"\n', 'trigger'));
  it('A trigger with a query sign', () => bad('trigger = "+"\n', 'trigger'));
  it('A trigger with a space', () => bad('trigger = ": "\n', 'trigger'));
  it('A trigger of four characters', () => bad('trigger = "::::"\n', 'trigger'));
  it('An empty trigger', () => bad('trigger = ""\n', 'trigger'));
  it('A trigger that is not a string', () => bad('trigger = 2\n', 'trigger'));
  it('A known skin tone', () => good('skin_tone = "medium-dark"\n'));
  it('A skin tone in upper case', () => bad('skin_tone = "Medium"\n', 'skin_tone'));
  it('A skin tone that is not a string', () => bad('skin_tone = 3\n', 'skin_tone'));
  it('Usage turned off', () => good('remember_usage = false\n'));
  it('remember_usage as a string', () => bad('remember_usage = "no"\n', 'remember_usage'));

  it('A name with a plus sign', () => good('[aliases]\n"lgtm+" = "👍"\n'));
  it('A name with a space', () => bad('[aliases]\n"looks good" = "👍"\n', 'aliases.looks good'));
  it('Two names that differ in case', () => bad('[aliases]\nlgtm = "👍"\nLGTM = "✅"\n', 'aliases.LGTM'));

  it('A text emoji', () => good("[aliases]\nshrug = '¯\\_(ツ)_/¯'\n"));
  it('An empty text', () => bad('[aliases]\nlgtm = ""\n', 'aliases.lgtm'));
  it('A text of 17 code points', () => bad('[aliases]\nlong = "abcdefghijklmnopq"\n', 'aliases.long'));
  it('A text with a line break', () => bad('[aliases]\ntwo = "a\\nb"\n', 'aliases.two'));
  it('A text that is not a string', () => bad('[aliases]\none = 1\n', 'aliases.one'));
});
