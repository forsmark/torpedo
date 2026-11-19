//! The skin tone: which emoji take it, and how.
//!
//! The engine applies the tone only to gemoji entries with `skin_tones`. For
//! such an emoji, [`apply`] puts the tone's modifier right after each
//! Emoji_Modifier_Base character and drops a VS16 (U+FE0F) right after one. A
//! modifier base between two ZWJs (U+200D) stays bare: in 🧑‍🤝‍🧑 only the two
//! people take the tone, not the handshake.

use std::fmt;

/// The skin tone of the config file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkinTone {
    #[default]
    None,
    Light,
    MediumLight,
    Medium,
    MediumDark,
    Dark,
}

/// The key of the skin tone in the config file.
pub const SKIN_TONE_KEY: &str = "skin_tone";

const ZWJ: char = '\u{200D}';
const VS16: char = '\u{FE0F}';

/// The Emoji_Modifier_Base code points, from Unicode 16.0
/// `https://www.unicode.org/Public/16.0.0/ucd/emoji/emoji-data.txt`
/// (2024-05-01). 134 code points in 50 ranges.
const MODIFIER_BASES: &[(u32, u32)] = &[
    (0x261D, 0x261D),
    (0x26F9, 0x26F9),
    (0x270A, 0x270C),
    (0x270D, 0x270D),
    (0x1F385, 0x1F385),
    (0x1F3C2, 0x1F3C4),
    (0x1F3C7, 0x1F3C7),
    (0x1F3CA, 0x1F3CA),
    (0x1F3CB, 0x1F3CC),
    (0x1F442, 0x1F443),
    (0x1F446, 0x1F450),
    (0x1F466, 0x1F46B),
    (0x1F46C, 0x1F46D),
    (0x1F46E, 0x1F478),
    (0x1F47C, 0x1F47C),
    (0x1F481, 0x1F483),
    (0x1F485, 0x1F487),
    (0x1F48F, 0x1F48F),
    (0x1F491, 0x1F491),
    (0x1F4AA, 0x1F4AA),
    (0x1F574, 0x1F575),
    (0x1F57A, 0x1F57A),
    (0x1F590, 0x1F590),
    (0x1F595, 0x1F596),
    (0x1F645, 0x1F647),
    (0x1F64B, 0x1F64F),
    (0x1F6A3, 0x1F6A3),
    (0x1F6B4, 0x1F6B5),
    (0x1F6B6, 0x1F6B6),
    (0x1F6C0, 0x1F6C0),
    (0x1F6CC, 0x1F6CC),
    (0x1F90C, 0x1F90C),
    (0x1F90F, 0x1F90F),
    (0x1F918, 0x1F918),
    (0x1F919, 0x1F91E),
    (0x1F91F, 0x1F91F),
    (0x1F926, 0x1F926),
    (0x1F930, 0x1F930),
    (0x1F931, 0x1F932),
    (0x1F933, 0x1F939),
    (0x1F93C, 0x1F93E),
    (0x1F977, 0x1F977),
    (0x1F9B5, 0x1F9B6),
    (0x1F9B8, 0x1F9B9),
    (0x1F9BB, 0x1F9BB),
    (0x1F9CD, 0x1F9CF),
    (0x1F9D1, 0x1F9DD),
    (0x1FAC3, 0x1FAC5),
    (0x1FAF0, 0x1FAF6),
    (0x1FAF7, 0x1FAF8),
];

/// Whether `c` is an Emoji_Modifier_Base character.
pub fn is_modifier_base(c: char) -> bool {
    let c = c as u32;
    MODIFIER_BASES.iter().any(|&(a, b)| (a..=b).contains(&c))
}

impl SkinTone {
    /// Every skin tone, in the order the settings window shows them.
    pub const ALL: [SkinTone; 6] = [
        SkinTone::None,
        SkinTone::Light,
        SkinTone::MediumLight,
        SkinTone::Medium,
        SkinTone::MediumDark,
        SkinTone::Dark,
    ];

    /// The value of the config file, such as `medium-light`.
    pub fn as_str(self) -> &'static str {
        match self {
            SkinTone::None => "none",
            SkinTone::Light => "light",
            SkinTone::MediumLight => "medium-light",
            SkinTone::Medium => "medium",
            SkinTone::MediumDark => "medium-dark",
            SkinTone::Dark => "dark",
        }
    }

    /// The skin tone of a config value. Case counts.
    pub fn from_config(value: &str) -> Option<SkinTone> {
        SkinTone::ALL.into_iter().find(|t| t.as_str() == value)
    }

    /// The Fitzpatrick modifier, U+1F3FB to U+1F3FF. None for `none`.
    pub fn modifier(self) -> Option<char> {
        let offset = match self {
            SkinTone::None => return None,
            SkinTone::Light => 0,
            SkinTone::MediumLight => 1,
            SkinTone::Medium => 2,
            SkinTone::MediumDark => 3,
            SkinTone::Dark => 4,
        };
        char::from_u32(0x1F3FB + offset)
    }
}

impl fmt::Display for SkinTone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Check a skin tone value of the config file.
pub fn check_skin_tone(value: &str) -> Result<SkinTone, String> {
    SkinTone::from_config(value).ok_or_else(|| {
        let all: Vec<&str> = SkinTone::ALL.iter().map(|t| t.as_str()).collect();
        format!("the skin tone {value:?} is not one of {}", all.join(", "))
    })
}

/// The emoji in this skin tone. Call it only for an emoji that gemoji marks
/// with `skin_tones`. With `SkinTone::None` it returns the emoji as it is.
pub fn apply(emoji: &str, tone: SkinTone) -> String {
    let Some(modifier) = tone.modifier() else {
        return emoji.to_string();
    };
    let chars: Vec<char> = emoji.chars().collect();
    let mut out = String::with_capacity(emoji.len() + 8);
    let mut toned_before = false;
    for (i, &c) in chars.iter().enumerate() {
        if c == VS16 && toned_before {
            toned_before = false;
            continue;
        }
        out.push(c);
        toned_before = false;
        let between_zwjs = i > 0 && chars[i - 1] == ZWJ && chars.get(i + 1) == Some(&ZWJ);
        if is_modifier_base(c) && !between_zwjs {
            out.push(modifier);
            toned_before = true;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emoji::EmojiIndex;
    use std::collections::HashSet;
    use std::process::Command;

    fn hex(s: &str) -> String {
        let parts: Vec<String> = s.chars().map(|c| format!("{:X}", c as u32)).collect();
        parts.join(" ")
    }

    #[test]
    fn the_config_values() {
        assert_eq!(
            SkinTone::from_config("medium-light"),
            Some(SkinTone::MediumLight)
        );
        assert_eq!(SkinTone::from_config("Medium"), None);
        assert_eq!(SkinTone::from_config(""), None);
        for t in SkinTone::ALL {
            assert_eq!(SkinTone::from_config(t.as_str()), Some(t));
        }
        assert!(
            check_skin_tone("purple")
                .unwrap_err()
                .contains("medium-dark")
        );
        assert_eq!(SkinTone::default(), SkinTone::None);
    }

    #[test]
    fn the_modifiers() {
        assert_eq!(SkinTone::None.modifier(), None);
        assert_eq!(SkinTone::Light.modifier(), Some('\u{1F3FB}'));
        assert_eq!(SkinTone::Dark.modifier(), Some('\u{1F3FF}'));
    }

    #[test]
    fn none_returns_the_emoji() {
        assert_eq!(apply("👋", SkinTone::None), "👋");
        assert_eq!(apply("✌\u{FE0F}", SkinTone::None), "✌\u{FE0F}");
    }

    #[test]
    fn a_hand_in_the_light_tone() {
        assert_eq!(hex(&apply("🤚", SkinTone::Light)), "1F91A 1F3FB");
    }

    #[test]
    fn the_vs16_after_a_modifier_base_goes() {
        assert_eq!(hex(&apply("✌\u{FE0F}", SkinTone::Medium)), "270C 1F3FD");
    }

    #[test]
    fn a_person_with_a_sign() {
        let golfing_woman = "\u{1F3CC}\u{FE0F}\u{200D}\u{2640}\u{FE0F}";
        assert_eq!(
            hex(&apply(golfing_woman, SkinTone::MediumDark)),
            "1F3CC 1F3FE 200D 2640 FE0F"
        );
    }

    #[test]
    fn two_people_get_the_same_tone() {
        let couple = "\u{1F469}\u{200D}\u{2764}\u{FE0F}\u{200D}\u{1F468}";
        assert_eq!(
            hex(&apply(couple, SkinTone::Light)),
            "1F469 1F3FB 200D 2764 FE0F 200D 1F468 1F3FB"
        );
    }

    #[test]
    fn the_handshake_between_two_people_stays_bare() {
        let holding = "\u{1F9D1}\u{200D}\u{1F91D}\u{200D}\u{1F9D1}";
        assert_eq!(
            hex(&apply(holding, SkinTone::MediumLight)),
            "1F9D1 1F3FC 200D 1F91D 200D 1F9D1 1F3FC"
        );
        assert_eq!(hex(&apply("🤝", SkinTone::MediumLight)), "1F91D 1F3FC");
    }

    #[test]
    fn the_table_has_134_code_points() {
        let n: u32 = MODIFIER_BASES.iter().map(|(a, b)| b - a + 1).sum();
        assert_eq!(n, 134);
    }

    const IBUS_DICT: &str = "/usr/share/ibus/dicts/emoji-en.dict";
    const PRINT_DICT: &str = r#"
import sys, gi
gi.require_version('IBus', '1.0')
from gi.repository import IBus
for d in IBus.EmojiData.load(sys.argv[1]):
    print(' '.join('%X' % ord(c) for c in d.get_emoji()))
"#;

    /// The sequences of the IBus emoji dictionary, as hex code points.
    fn ibus_sequences() -> HashSet<String> {
        let out = Command::new("python3")
            .args(["-I", "-c", PRINT_DICT, IBUS_DICT])
            .output()
            .expect(
                "python3 runs. The skin tone oracle needs python3, gir1.2-ibus-1.0 and ibus-data",
            );
        assert!(
            out.status.success(),
            "the skin tone oracle needs python3, gir1.2-ibus-1.0 and ibus-data ({IBUS_DICT}): {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let seqs: HashSet<String> = String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(str::to_string)
            .collect();
        assert!(
            seqs.len() > 1000,
            "the IBus dictionary holds {} sequences",
            seqs.len()
        );
        seqs
    }

    #[test]
    fn every_toned_form_is_in_the_ibus_dictionary() {
        let seqs = ibus_sequences();
        let mut missing = Vec::new();
        let mut checked = 0;
        for e in EmojiIndex::gemoji()
            .entries()
            .iter()
            .filter(|e| e.skin_tones)
        {
            for tone in &SkinTone::ALL[1..] {
                checked += 1;
                let toned = apply(&e.emoji, *tone);
                if !seqs.contains(&hex(&toned)) {
                    missing.push(format!("{} {tone}: {}", e.shortcodes[0], hex(&toned)));
                }
            }
        }
        assert_eq!(checked, 305 * 5);
        assert!(
            missing.is_empty(),
            "not in {IBUS_DICT}:\n{}",
            missing.join("\n")
        );
    }
}
