/**
 * Rust is installed per user with rustup, in ~/.cargo/bin. A shell that never
 * ran `source ~/.cargo/env` lacks that folder on its PATH. The scripts that
 * start cargo give their commands this environment, so cargo is found anyway.
 */
import { existsSync } from 'node:fs';
import { homedir } from 'node:os';
import { delimiter, join } from 'node:path';

/** The folder that holds cargo: $CARGO_HOME/bin, or ~/.cargo/bin. */
export function cargoBin(env = process.env, home = homedir()) {
  return join(env.CARGO_HOME ?? join(home, '.cargo'), 'bin');
}

/**
 * A copy of `env` whose PATH ends with the cargo folder, when that folder
 * exists and PATH lacks it. Otherwise a plain copy of `env`. The folder goes
 * last, so a cargo that PATH already names still wins.
 */
export function withCargoOnPath(env = process.env, { home = homedir(), exists = existsSync } = {}) {
  const bin = cargoBin(env, home);
  const path = env.PATH ?? '';
  if (!exists(bin) || path.split(delimiter).includes(bin)) return { ...env };
  return { ...env, PATH: path ? `${path}${delimiter}${bin}` : bin };
}
