/**
 * An app with one input context, as the e2e tests see it.
 *
 * App folds the driver's events into what the app would show: `text` gets each
 * commit, and each key the engine lets through, in the order they arrive. A key
 * the engine lets through adds its character, unless Ctrl, Alt or Super is held.
 * `passed` lists the keys that reached the app. So a test that asserts on `text`
 * also checks the order of commits and passed keys.
 */

export const SHIFT = 1 << 0;
export const LOCK = 1 << 1;
export const CONTROL = 1 << 2;
export const MOD1 = 1 << 3;
export const MOD2 = 1 << 4;
export const MOD4 = 1 << 6;
export const MOD5 = 1 << 7;
export const SUPER = 1 << 26;
export const RELEASE = 1 << 30;
const PASS_THROUGH = CONTROL | MOD1 | MOD4 | SUPER | (1 << 27) | (1 << 28);

export const KEY = {
  BackSpace: 0xff08,
  Tab: 0xff09,
  Return: 0xff0d,
  Escape: 0xff1b,
  Left: 0xff51,
  Up: 0xff52,
  Down: 0xff54,
  Page_Up: 0xff55,
  Page_Down: 0xff56,
  Shift_L: 0xffe1,
  ISO_Level3_Shift: 0xfe03,
  dead_grave: 0xfe50,
  dead_acute: 0xfe51,
  dead_circumflex: 0xfe52,
  dead_tilde: 0xfe53,
  dead_diaeresis: 0xfe57,
  dead_belowdot: 0xfe60,
  space: 0x20,
};

/** The keyval of a character. */
export function keyvalOf(ch) {
  const cp = ch.codePointAt(0);
  return cp < 0x100 ? cp : 0x0100_0000 + cp;
}

/** The character of a keyval, or '' when it has none. */
export function charOf(keyval) {
  if ((keyval >= 0x20 && keyval <= 0x7e) || (keyval >= 0xa0 && keyval <= 0xff)) return String.fromCodePoint(keyval);
  if (keyval >= 0x0100_0000) return String.fromCodePoint(keyval - 0x0100_0000);
  return '';
}

/** The modifier state the Danish layout gives a character. */
export function stateOf(ch) {
  if (/[A-Z:_;>*?!"#%&/()=]/.test(ch)) return SHIFT;
  if (/[@£${}[\]|~\\€]/.test(ch)) return MOD5;
  return 0;
}

export class App {
  constructor(session, reply) {
    this.session = session;
    this.text = '';
    this.passed = [];
    this.preedit = { text: '', attrs: [], visible: false };
    this.table = null;
    this.log = [];
    this.last = [];
    this.needsSurroundingText = reply.needs_surrounding_text;
    this.fold(reply.events);
  }

  fold(events, key) {
    this.last = events;
    this.log.push(...events);
    for (const e of events) {
      switch (e.type) {
        case 'commit':
          this.text += e.text;
          break;
        case 'preedit':
          this.preedit = { text: e.visible ? e.text : '', attrs: e.attrs, visible: e.visible };
          break;
        case 'hide-preedit':
          this.preedit = { text: '', attrs: [], visible: false };
          break;
        case 'table':
          this.table = e.visible ? e : null;
          break;
        case 'hide-table':
          this.table = null;
          break;
        case 'reply':
          if (key && e.handled === false) this.letThrough(key);
          break;
        case 'forward':
          this.letThrough({ keyval: e.keyval, state: e.state });
          break;
      }
    }
  }

  letThrough(key) {
    this.passed.push(key);
    if (!(key.state & (PASS_THROUGH | RELEASE))) this.text += charOf(key.keyval);
  }

  /** The verdict of the last key: true when the engine used it. */
  get consumed() {
    const reply = this.last.find((e) => e.type === 'reply');
    return reply?.handled;
  }

  /** The commits of the last command. */
  get commits() {
    return this.last.filter((e) => e.type === 'commit').map((e) => e.text);
  }

  async command(cmd) {
    const reply = await this.session.send(cmd);
    this.fold(reply.events ?? []);
    return reply;
  }

  async press(keyval, state = 0) {
    const key = { keyval, state };
    const reply = await this.session.send({ cmd: 'key', keyval, state });
    this.fold(reply.events, key);
    return this;
  }

  async release(keyval, state = 0) {
    return this.press(keyval, state | RELEASE);
  }

  /** Type each character with the state the Danish layout gives it. */
  async type(text) {
    for (const ch of text) await this.press(keyvalOf(ch), stateOf(ch));
    return this;
  }

  async surrounding(text, cursor, anchor) {
    const n = [...text].length;
    return this.command({ cmd: 'surrounding', text, cursor: cursor ?? n, anchor: anchor ?? cursor ?? n });
  }

  focusIn() {
    return this.command({ cmd: 'focus_in' });
  }

  focusOut() {
    return this.command({ cmd: 'focus_out' });
  }

  reset() {
    return this.command({ cmd: 'reset' });
  }

  click(index) {
    return this.command({ cmd: 'click', index });
  }

  switchEngine(name) {
    return this.command({ cmd: 'switch_engine', name });
  }

  /** Whether a search is open: the preedit shows the trigger. */
  get searching() {
    return this.preedit.text.startsWith('::');
  }
}
