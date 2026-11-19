//! Where the trigger counts.
//!
//! A boundary is the start of the text, or a place right after whitespace, an
//! emoji, an opening bracket or an opening quote. The engine reads the
//! character before the cursor from the surrounding text when the app sent
//! it. Otherwise it uses the last character it passed through or committed.

/// The opening brackets and quotes after which the trigger counts.
pub const OPENERS: [char; 8] = ['(', '[', '{', '"', '\'', '«', '„', '“'];

/// Whether `c` can end an emoji. A pragmatic check, not the full
/// Extended_Pictographic property: a unit test runs it over the last
/// character of every emoji in the gemoji data.
pub fn ends_emoji(c: char) -> bool {
    matches!(c as u32,
        0x1f000..=0x1faff
        | 0x2190..=0x21ff
        | 0x2300..=0x23ff
        | 0x25a0..=0x25ff
        | 0x2900..=0x297f
        | 0x2600..=0x27bf
        | 0x2b00..=0x2bff
        | 0x3030 | 0x303d | 0x3297 | 0x3299
        | 0xa9 | 0xae | 0x203c | 0x2049 | 0x2122 | 0x2139
        | 0xfe0f | 0x20e3 | 0xe007f)
}

/// Whether the trigger counts right after `c`.
pub fn is_boundary_after(c: char) -> bool {
    c.is_whitespace() || OPENERS.contains(&c) || ends_emoji(c)
}

/// What the engine knows about the text before the cursor.
#[derive(Debug, Default)]
pub struct Context {
    /// The character before the cursor in the last surrounding text. `Some(None)`
    /// is the start of the text.
    surrounding: Option<Option<char>>,
    /// Whether keys passed or text committed since the surrounding text came.
    stale: bool,
    /// The last character the engine passed through or committed.
    last: Option<char>,
}

impl Context {
    /// Forget everything, as at focus-in.
    pub fn reset(&mut self) {
        *self = Context::default();
    }

    /// The app sent its surrounding text. `cursor` and `anchor` count characters.
    pub fn set_surrounding(&mut self, text: &str, cursor: u32, anchor: u32) {
        let at = cursor.min(anchor) as usize;
        self.surrounding = Some(if at == 0 {
            None
        } else {
            text.chars().nth(at - 1)
        });
        self.stale = false;
    }

    /// The engine passed `c` through or committed it.
    pub fn saw_char(&mut self, c: char) {
        self.last = Some(c);
        self.stale = true;
    }

    /// The engine committed `text`.
    pub fn saw_text(&mut self, text: &str) {
        if let Some(c) = text.chars().last() {
            self.saw_char(c);
        }
    }

    /// The engine passed a key without a character, such as Left or Ctrl+v.
    /// The text before the cursor is now unknown.
    pub fn saw_unknown(&mut self) {
        self.last = None;
        self.stale = true;
    }

    /// Whether the cursor is at a boundary.
    pub fn at_boundary(&self) -> bool {
        match (self.surrounding, self.stale) {
            (Some(before), false) => before.is_none_or(is_boundary_after),
            _ => self.last.is_none_or(is_boundary_after),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emoji::EmojiIndex;

    fn with(text: &str) -> Context {
        let mut c = Context::default();
        let n = text.chars().count() as u32;
        c.set_surrounding(text, n, n);
        c
    }

    #[test]
    fn the_start_of_the_text_is_a_boundary() {
        assert!(with("").at_boundary());
    }

    #[test]
    fn after_a_space_is_a_boundary() {
        assert!(with("hello ").at_boundary());
    }

    #[test]
    fn after_a_letter_is_not() {
        assert!(!with("std").at_boundary());
    }

    #[test]
    fn after_openers_is_a_boundary() {
        assert!(with("(").at_boundary());
        assert!(with("„").at_boundary());
        assert!(with("x «").at_boundary());
    }

    #[test]
    fn after_an_emoji_is_a_boundary() {
        assert!(with("ok 🔥").at_boundary());
        assert!(with("❤️").at_boundary());
    }

    #[test]
    fn a_selection_uses_its_start() {
        let mut c = Context::default();
        c.set_surrounding("ab cd", 5, 3);
        assert!(c.at_boundary());
        c.set_surrounding("ab cd", 3, 5);
        assert!(c.at_boundary());
        c.set_surrounding("ab cd", 2, 5);
        assert!(!c.at_boundary());
    }

    #[test]
    fn stale_surrounding_text_gives_way_to_the_last_character() {
        let mut c = with("hello ");
        c.saw_char('x');
        assert!(!c.at_boundary());
        c.set_surrounding("hello x ", 8, 8);
        assert!(c.at_boundary());
    }

    #[test]
    fn nothing_known_is_a_boundary() {
        assert!(Context::default().at_boundary());
    }

    #[test]
    fn an_unknown_last_character_is_a_boundary() {
        let mut c = Context::default();
        c.saw_char('d');
        assert!(!c.at_boundary());
        c.saw_unknown();
        assert!(c.at_boundary());
    }

    #[test]
    fn a_committed_emoji_is_a_boundary() {
        let mut c = Context::default();
        c.saw_text("🔥");
        assert!(c.at_boundary());
    }

    #[test]
    fn every_gemoji_emoji_ends_at_a_boundary() {
        for e in EmojiIndex::gemoji().entries() {
            let last = e.emoji.chars().last().unwrap();
            assert!(ends_emoji(last), "{} ends with {:#x}", e.emoji, last as u32);
        }
    }
}
