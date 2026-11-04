//! セルの値を、表の 1 行に収まる表示の文字列にする（見えない文字は印に・VIEW-05）。

use alloc::string::String;

/// 改行の印。🔴 `↵`（U+21B5）は日本語の字体（Noto Sans CJK JP）に無く、Issue #20 の画面で字の無い印になった。
/// JIS X 0208 にもある `¶` を使う。
const LINE_BREAK: char = '¶';
/// タブの印。
const TAB: char = '→';
/// そのほかの制御文字の印。
const CONTROL: char = '·';
/// 全角空白（U+3000）の印（日本語のエディターの慣習）。
const IDEOGRAPHIC_SPACE: char = '□';
/// ノーブレークスペース（U+00A0・U+202F）の印。
const NO_BREAK_SPACE: char = '␣';
/// ゼロ幅の文字（U+200B〜200D・U+2060・U+FEFF）の印。
const ZERO_WIDTH: char = '¦';

/// セルの値（復号した文字列）を、表の 1 行に収まる表示の文字列にする（VIEW-05）。
///
/// 改行（`\r\n`・`\n`・`\r`・U+2028・U+2029）は 1 つの `¶`、タブは `→`、全角空白は `□`、ノーブレークスペースは `␣`、
/// ゼロ幅の文字は `¦`、そのほかの制御文字は `·` にする（Issue #19・#53）。値は変えない（表示だけ）。
#[must_use]
pub fn single_line(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut characters = value.chars().peekable();
    while let Some(character) = characters.next() {
        let shown = match character {
            '\r' => {
                characters.next_if_eq(&'\n');
                LINE_BREAK
            }
            '\n' | '\u{2028}' | '\u{2029}' => LINE_BREAK,
            '\t' => TAB,
            '\u{3000}' => IDEOGRAPHIC_SPACE,
            '\u{a0}' | '\u{202f}' => NO_BREAK_SPACE,
            '\u{200b}'..='\u{200d}' | '\u{2060}' | '\u{feff}' => ZERO_WIDTH,
            other if other.is_control() => CONTROL,
            other => other,
        };
        out.push(shown);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_breaks_tabs_and_controls_become_marks() {
        assert_eq!(single_line("東京都\r\n千代田区"), "東京都¶千代田区");
        assert_eq!(single_line("a\nb\rc"), "a¶b¶c");
        assert_eq!(single_line("a\tb\u{0}c"), "a→b·c");
        assert_eq!(single_line(" 00123 "), " 00123 ");
        assert_eq!(single_line(""), "");
    }

    #[test]
    fn invisible_spaces_and_zero_width_characters_become_marks() {
        assert_eq!(single_line("山田\u{3000}太郎"), "山田□太郎");
        assert_eq!(single_line("10\u{a0}000\u{202f}円"), "10␣000␣円");
        assert_eq!(
            single_line("ab\u{200b}c\u{200d}\u{2060}\u{feff}"),
            "ab¦c¦¦¦"
        );
        assert_eq!(single_line("一\u{2028}二\u{2029}三"), "一¶二¶三");
        // 普通の空白は変えない。
        assert_eq!(single_line("a b"), "a b");
    }
}
