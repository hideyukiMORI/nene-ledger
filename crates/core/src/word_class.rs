//! 語の切り方。ダブルクリックで選ぶ語を、同じ種類の字の続きとして決める（GitHub Issue #466）。
//!
//! 辞書は使わない。英数字・漢字・ひらがな・カタカナ・空白はそれぞれの続きで 1 語、改行とそれ以外の記号は 1 字で
//! 1 語。位置はすべて文字（`char`）の添字。

/// 語を切るための字の種類。閉じた集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordClass {
    /// 英数字と `_`（全角の英数字・ほかの書記体系の字を含む）。
    Alphanumeric,
    /// 漢字（`々`・`〆` を含む）。
    Kanji,
    /// ひらがな。
    Hiragana,
    /// カタカナ（長音符・半角のカタカナを含む）。
    Katakana,
    /// 改行を除く空白（全角の空白を含む）。続きで 1 語。
    Space,
    /// 改行。1 字で 1 語。
    LineBreak,
    /// それ以外の記号。1 字で 1 語。
    Symbol,
}

impl WordClass {
    /// `character` の種類。
    #[must_use]
    pub fn of(character: char) -> Self {
        match character {
            '\n' => Self::LineBreak,
            '\u{3005}'
            | '\u{3006}'
            | '\u{3400}'..='\u{4DBF}'
            | '\u{4E00}'..='\u{9FFF}'
            | '\u{F900}'..='\u{FAFF}'
            | '\u{20000}'..='\u{3FFFF}' => Self::Kanji,
            '\u{3041}'..='\u{309F}' => Self::Hiragana,
            '\u{30A0}'..='\u{30FA}'
            | '\u{30FC}'..='\u{30FF}'
            | '\u{31F0}'..='\u{31FF}'
            | '\u{FF66}'..='\u{FF9F}' => Self::Katakana,
            '_' => Self::Alphanumeric,
            other if other.is_whitespace() => Self::Space,
            other if other.is_alphanumeric() => Self::Alphanumeric,
            _ => Self::Symbol,
        }
    }

    /// 続けて並べば 1 語になる種類か。改行と記号は 1 字で 1 語。
    #[must_use]
    pub const fn joins(self) -> bool {
        match self {
            Self::Alphanumeric | Self::Kanji | Self::Hiragana | Self::Katakana | Self::Space => {
                true
            }
            Self::LineBreak | Self::Symbol => false,
        }
    }

    /// `text` の `index` 字目を含む語の始まりと、含まない終わり。`index` が末尾（より右）か改行なら、その前の字の語
    /// （行の末より右を押したとき）。前の字も無いか改行なら、空の範囲。
    #[must_use]
    pub fn span_around(text: &str, index: usize) -> (usize, usize) {
        let classes: alloc::vec::Vec<Self> = text.chars().map(Self::of).collect();
        let index = index.min(classes.len());
        let word = |position: usize| {
            classes
                .get(position)
                .is_some_and(|class| *class != Self::LineBreak)
        };
        let target = if word(index) {
            index
        } else {
            match index.checked_sub(1) {
                Some(before) if word(before) => before,
                Some(_) | None => return (index, index),
            }
        };
        let Some(&class) = classes.get(target) else {
            return (index, index);
        };
        if !class.joins() {
            return (target, target.saturating_add(1));
        }
        let start = classes.get(..target).map_or(0, |head| {
            head.iter()
                .rposition(|other| *other != class)
                .map_or(0, |last| last.saturating_add(1))
        });
        let end = classes.get(target..).map_or(target, |tail| {
            tail.iter()
                .position(|other| *other != class)
                .map_or(classes.len(), |first| target.saturating_add(first))
        });
        (start, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_word_is_a_run_of_the_same_kind_of_character() {
        let text = "東京都に住む ABC_12、トーキョー";
        assert_eq!(WordClass::span_around(text, 1), (0, 3));
        assert_eq!(WordClass::span_around(text, 3), (3, 4));
        // 辞書を使わないので、送り仮名は別の語（「住」と「む」）。
        assert_eq!(WordClass::span_around(text, 4), (4, 5));
        assert_eq!(WordClass::span_around(text, 5), (5, 6));
        assert_eq!(WordClass::span_around(text, 6), (6, 7));
        assert_eq!(WordClass::span_around(text, 9), (7, 13));
        assert_eq!(WordClass::span_around(text, 13), (13, 14));
        assert_eq!(WordClass::span_around(text, 16), (14, 19));
    }

    #[test]
    fn symbols_and_line_breaks_are_one_character_each() {
        let text = "a..b\n\nc";
        assert_eq!(WordClass::span_around(text, 1), (1, 2));
        assert_eq!(WordClass::span_around(text, 2), (2, 3));
        // 改行の上（行の末より右）は、その行の最後の字の語。
        assert_eq!(WordClass::span_around(text, 4), (3, 4));
        // 空の行は空の範囲。
        assert_eq!(WordClass::span_around(text, 5), (5, 5));
        // 末尾より右は最後の字の語。
        assert_eq!(WordClass::span_around(text, 7), (6, 7));
        assert_eq!(WordClass::span_around(text, 99), (6, 7));
        assert_eq!(WordClass::span_around("", 0), (0, 0));
    }

    #[test]
    fn full_width_letters_and_half_width_kana_join_their_kind() {
        assert_eq!(WordClass::span_around("ＡＢＣ１２３　ｶﾀｶﾅ", 2), (0, 6));
        assert_eq!(WordClass::span_around("ＡＢＣ１２３　ｶﾀｶﾅ", 8), (7, 11));
        assert_eq!(WordClass::of('ー'), WordClass::Katakana);
        assert_eq!(WordClass::of('・'), WordClass::Symbol);
        assert_eq!(WordClass::of('々'), WordClass::Kanji);
        assert_eq!(WordClass::of('\u{3000}'), WordClass::Space);
    }
}
