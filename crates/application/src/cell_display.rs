//! セルの値を、表の 1 行に収まる表示の文字列にする（見えない文字は印に・VIEW-05）。

use alloc::string::String;

use nene_ledger_core::display_width::char_columns;

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

/// 省略の印（U+2026）。
const ELLIPSIS: char = '…';

/// 表示の文字列 `text` を幅 `columns`（半角 1・全角 2）に収める。収まらなければ、収まる頭の部分の後ろに `…` を付ける
/// （GitHub Issue #7・VIEW-17）。値そのものは変えない（コピーと詳細ペインは全文）。
///
/// 🔴 数えるのは頭の部分だけで、値の長さに依らない（1 MiB の値でもセルの幅に比例する手間）。幅は近似（半角 1・全角 2）
/// なので、描く側の矩形での切り取りは残す。幅が `…` より狭ければ `…` だけにする。
#[must_use]
pub fn fitted(text: &str, columns: u32) -> String {
    let room = columns.saturating_sub(char_columns(ELLIPSIS));
    let mut head = String::new();
    let mut used = 0_u32;
    let mut cut = false;
    let mut characters = text.chars();
    for character in characters.by_ref() {
        let next = used.saturating_add(char_columns(character));
        if next > columns {
            cut = true;
            break;
        }
        if next <= room {
            head.push(character);
        }
        used = next;
    }
    if !cut {
        return String::from(text);
    }
    // 頭の部分は `…` の分を空けて収まる所まで（上の輪で `room` を超えた字からは足していない）。
    head.push(ELLIPSIS);
    head
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 収まる値はそのまま、収まらない値は頭の部分 ＋ `…`（`…` は幅 2）。半角・全角・混在（GitHub Issue #7）。
    #[test]
    fn values_that_do_not_fit_end_with_an_ellipsis() {
        assert_eq!(fitted("abcdef", 6), "abcdef");
        assert_eq!(fitted("abcdefg", 6), "abcd…");
        assert_eq!(fitted("東京都千代田区", 14), "東京都千代田区");
        assert_eq!(fitted("東京都千代田区", 13), "東京都千代…");
        assert_eq!(fitted("ab東京", 5), "ab…");
        assert_eq!(fitted("", 0), "");
        assert_eq!(fitted("abc", 1), "…");
    }

    /// 長い値でも頭の部分だけを見る（1 MiB の値で、結果はセルの幅の分だけ）。
    #[test]
    fn a_huge_value_is_cut_to_the_cell() {
        let huge = "x".repeat(1 << 20_u32);
        let shown = fitted(&huge, 40);
        assert_eq!(shown.chars().count(), 39);
        assert!(shown.ends_with('…'));
    }

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
