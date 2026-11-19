import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { withCargoOnPath } from './cargo-path.mjs';

const home = '/home/someone';
const there = () => true;

describe('cargo on the PATH', () => {
  it('adds ~/.cargo/bin last when PATH lacks it', () => {
    const env = withCargoOnPath({ PATH: '/usr/bin' }, { home, exists: there });
    assert.equal(env.PATH, '/usr/bin:/home/someone/.cargo/bin');
  });

  it('uses $CARGO_HOME/bin when CARGO_HOME is set', () => {
    const env = withCargoOnPath({ PATH: '/usr/bin', CARGO_HOME: '/opt/cargo' }, { home, exists: there });
    assert.equal(env.PATH, '/usr/bin:/opt/cargo/bin');
  });

  it('leaves PATH alone when it already holds the folder', () => {
    const PATH = '/usr/bin:/home/someone/.cargo/bin';
    assert.equal(withCargoOnPath({ PATH }, { home, exists: there }).PATH, PATH);
  });

  it('leaves PATH alone when the folder does not exist', () => {
    assert.equal(withCargoOnPath({ PATH: '/usr/bin' }, { home, exists: () => false }).PATH, '/usr/bin');
  });
});
