//! What a key is: its character, whether it is a dead key or a modifier, and
//! whether its modifier state makes it pass through.
//!
//! Keyvals are X keysyms, as IBus sends them. The compositor has already
//! applied the layout, so `:` on the Danish layout arrives as keyval 0x3a with
//! the Shift bit set.

use unicode_normalization::UnicodeNormalization;

pub const SHIFT_MASK: u32 = 1 << 0;
pub const LOCK_MASK: u32 = 1 << 1;
pub const CONTROL_MASK: u32 = 1 << 2;
pub const MOD1_MASK: u32 = 1 << 3;
pub const MOD2_MASK: u32 = 1 << 4;
pub const MOD4_MASK: u32 = 1 << 6;
pub const MOD5_MASK: u32 = 1 << 7;
pub const SUPER_MASK: u32 = 1 << 26;
pub const HYPER_MASK: u32 = 1 << 27;
pub const META_MASK: u32 = 1 << 28;
pub const RELEASE_MASK: u32 = 1 << 30;

/// Ctrl, Alt, Super, Hyper and Meta. A key pressed with one of them passes
/// through. Shift, Caps Lock, Num Lock and AltGr are not in it.
pub const PASS_THROUGH_MASK: u32 =
    CONTROL_MASK | MOD1_MASK | MOD4_MASK | SUPER_MASK | HYPER_MASK | META_MASK;

pub const BACKSPACE: u32 = 0xff08;
pub const TAB: u32 = 0xff09;
pub const ISO_LEFT_TAB: u32 = 0xfe20;
pub const RETURN: u32 = 0xff0d;
pub const KP_ENTER: u32 = 0xff8d;
pub const ESCAPE: u32 = 0xff1b;
pub const LEFT: u32 = 0xff51;
pub const UP: u32 = 0xff52;
pub const DOWN: u32 = 0xff54;
pub const PAGE_UP: u32 = 0xff55;
pub const PAGE_DOWN: u32 = 0xff56;
pub const SHIFT_L: u32 = 0xffe1;
pub const ISO_LEVEL3_SHIFT: u32 = 0xfe03;
pub const DEAD_GRAVE: u32 = 0xfe50;
pub const DEAD_ACUTE: u32 = 0xfe51;
pub const DEAD_CIRCUMFLEX: u32 = 0xfe52;
pub const DEAD_TILDE: u32 = 0xfe53;
pub const DEAD_DIAERESIS: u32 = 0xfe57;
pub const DEAD_ABOVERING: u32 = 0xfe58;
pub const DEAD_BELOWDOT: u32 = 0xfe60;

/// Whether the key is a release.
pub fn is_release(state: u32) -> bool {
    state & RELEASE_MASK != 0
}

/// Whether Ctrl, Alt, Super, Hyper or Meta is held.
pub fn has_pass_through_modifier(state: u32) -> bool {
    state & PASS_THROUGH_MASK != 0
}

/// Whether the key is a modifier key alone, such as Shift or AltGr.
pub fn is_modifier_key(keyval: u32) -> bool {
    (0xffe1..=0xffee).contains(&keyval)
        || (0xfe01..=0xfe0f).contains(&keyval)
        || keyval == 0xff7e // Mode_switch
        || keyval == 0xff7f // Num_Lock
}

/// Whether the key is a dead key, known or not.
pub fn is_dead_key(keyval: u32) -> bool {
    (0xfe50..=0xfe8f).contains(&keyval)
}

/// The character a key types, if any.
pub fn char_of(keyval: u32) -> Option<char> {
    match keyval {
        0x20..=0x7e | 0xa0..=0xff => char::from_u32(keyval),
        0x0100_0000..=0x0110_ffff => char::from_u32(keyval - 0x0100_0000),
        0xff80 => Some(' '),
        0xffaa => Some('*'),
        0xffab => Some('+'),
        0xffac => Some(','),
        0xffad => Some('-'),
        0xffae => Some('.'),
        0xffaf => Some('/'),
        0xffb0..=0xffb9 => char::from_u32(u32::from(b'0') + keyval - 0xffb0),
        0xffbd => Some('='),
        _ => None,
    }
}

/// The accent of a dead key that torpedo composes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Accent {
    /// The dead key's keyval.
    pub keyval: u32,
    /// The combining mark it puts on the next character.
    pub combining: char,
    /// The accent on its own, as a space or a second press gives it.
    pub spacing: char,
}

const ACCENTS: [(u32, char, char); 13] = [
    (DEAD_GRAVE, '\u{300}', '`'),
    (DEAD_ACUTE, '\u{301}', '´'),
    (DEAD_CIRCUMFLEX, '\u{302}', '^'),
    (DEAD_TILDE, '\u{303}', '~'),
    (0xfe54, '\u{304}', '¯'), // dead_macron
    (0xfe55, '\u{306}', '˘'), // dead_breve
    (0xfe56, '\u{307}', '˙'), // dead_abovedot
    (DEAD_DIAERESIS, '\u{308}', '¨'),
    (DEAD_ABOVERING, '\u{30a}', '˚'),
    (0xfe59, '\u{30b}', '˝'), // dead_doubleacute
    (0xfe5a, '\u{30c}', 'ˇ'), // dead_caron
    (0xfe5b, '\u{327}', '¸'), // dead_cedilla
    (0xfe5c, '\u{328}', '˛'), // dead_ogonek
];

/// The accent of a dead key, if torpedo knows it.
pub fn accent(keyval: u32) -> Option<Accent> {
    ACCENTS
        .iter()
        .find(|(k, _, _)| *k == keyval)
        .map(|&(keyval, combining, spacing)| Accent {
            keyval,
            combining,
            spacing,
        })
}

/// The precomposed form of `base` with the accent, if one exists.
pub fn compose(base: char, accent: Accent) -> Option<char> {
    let mut composed = [base, accent.combining].into_iter().nfc();
    match (composed.next(), composed.next()) {
        (Some(c), None) => Some(c),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colon_with_shift_is_a_colon() {
        assert_eq!(char_of(0x3a), Some(':'));
        assert!(!has_pass_through_modifier(SHIFT_MASK));
    }

    #[test]
    fn brace_with_altgr_is_a_brace() {
        assert_eq!(char_of(0x7b), Some('{'));
        assert!(!has_pass_through_modifier(MOD5_MASK));
    }

    #[test]
    fn caps_lock_and_num_lock_do_not_pass_through() {
        assert!(!has_pass_through_modifier(LOCK_MASK | MOD2_MASK));
    }

    #[test]
    fn danish_letters_are_latin1_keyvals() {
        assert_eq!(char_of(0xe6), Some('æ'));
        assert_eq!(char_of(0xf8), Some('ø'));
        assert_eq!(char_of(0xe5), Some('å'));
    }

    #[test]
    fn unicode_keyvals_carry_their_code_point() {
        assert_eq!(char_of(0x0100_20ac), Some('€'));
    }

    #[test]
    fn keypad_digits_are_digits() {
        assert_eq!(char_of(0xffb1), Some('1'));
        assert_eq!(char_of(0xffab), Some('+'));
    }

    #[test]
    fn control_keys_have_no_character() {
        for k in [
            BACKSPACE, TAB, RETURN, ESCAPE, LEFT, UP, DEAD_ACUTE, SHIFT_L,
        ] {
            assert_eq!(char_of(k), None, "{k:#x}");
        }
    }

    #[test]
    fn dead_acute_is_a_known_dead_key() {
        assert!(is_dead_key(DEAD_ACUTE));
        assert_eq!(accent(DEAD_ACUTE).unwrap().spacing, '´');
    }

    #[test]
    fn dead_belowdot_is_an_unknown_dead_key() {
        assert!(is_dead_key(DEAD_BELOWDOT));
        assert_eq!(accent(DEAD_BELOWDOT), None);
    }

    #[test]
    fn shift_and_altgr_are_modifier_keys() {
        assert!(is_modifier_key(SHIFT_L));
        assert!(is_modifier_key(ISO_LEVEL3_SHIFT));
        assert!(!is_modifier_key(0x61));
    }

    #[test]
    fn ctrl_alt_and_super_pass_through() {
        for m in [CONTROL_MASK, MOD1_MASK, MOD4_MASK, SUPER_MASK] {
            assert!(has_pass_through_modifier(m | SHIFT_MASK), "{m:#x}");
        }
    }

    #[test]
    fn compose_gives_precomposed_forms() {
        let a = |k| accent(k).unwrap();
        assert_eq!(compose('e', a(DEAD_ACUTE)), Some('é'));
        assert_eq!(compose('u', a(DEAD_DIAERESIS)), Some('ü'));
        assert_eq!(compose('n', a(DEAD_TILDE)), Some('ñ'));
        assert_eq!(compose('E', a(DEAD_ACUTE)), Some('É'));
        assert_eq!(compose('a', a(DEAD_ABOVERING)), Some('å'));
    }

    #[test]
    fn compose_fails_without_a_precomposed_form() {
        assert_eq!(compose('q', accent(DEAD_ACUTE).unwrap()), None);
    }

    #[test]
    fn every_accent_has_distinct_keyvals() {
        let mut k: Vec<u32> = ACCENTS.iter().map(|a| a.0).collect();
        k.dedup();
        assert_eq!(k.len(), ACCENTS.len());
    }
}
