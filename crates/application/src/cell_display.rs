//! セルの値を、表の 1 行に収まる表示の文字列にする（見えない文字は印に・VIEW-05）。

use alloc::string::String;
use alloc::vec::Vec;
use core::ops::Range;

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
    push_single_line(value, &mut out);
    out
}

/// `single_line(value)` を `out` の後ろに足す（セルごとに `String` を作らずに 1 本の文字列へ書く・GitHub Issue #22）。
pub fn push_single_line(value: &str, out: &mut String) {
    out.extend(shown_characters(value).map(|(_, shown)| shown));
}

/// `value` を前から 1 字ずつ表示の字にした並び（元の字の始まりのバイトと、表示の字）。`\r\n` は 1 つの `¶` になる。
/// 字ごとの印は [`mark_of`] の表で決める（`single_line`・`is_single_line`・当たった字の位置の写しが同じ表を使う）。
fn shown_characters(value: &str) -> impl Iterator<Item = (usize, char)> + '_ {
    let mut characters = value.char_indices().peekable();
    core::iter::from_fn(move || {
        let (at, character) = characters.next()?;
        if character == '\r' {
            characters.next_if(|&(_, next)| next == '\n');
        }
        Some((at, mark_of(character).unwrap_or(character)))
    })
}

/// 字 `character` を表示で替える印（替えなければ `None`）。`single_line` と `is_single_line` の正本（2 つが食い違わない
/// ように表を 1 つにする）。`\r\n` の `\n` は `single_line` が `\r` と合わせて 1 つの印にする。
const fn mark_of(character: char) -> Option<char> {
    match character {
        '\r' | '\n' | '\u{2028}' | '\u{2029}' => Some(LINE_BREAK),
        '\t' => Some(TAB),
        '\u{3000}' => Some(IDEOGRAPHIC_SPACE),
        '\u{a0}' | '\u{202f}' => Some(NO_BREAK_SPACE),
        '\u{200b}'..='\u{200d}' | '\u{2060}' | '\u{feff}' => Some(ZERO_WIDTH),
        other if other.is_control() => Some(CONTROL),
        _ => None,
    }
}

/// `value` が表示の 1 行そのものか（`single_line(value) == value`・印に替える字が無い）。写しを作らずに確かめる
/// （窓の行は、表示の文字列が元の値と同じなら写しを持たない・GitHub Issue #22）。
#[must_use]
pub fn is_single_line(value: &str) -> bool {
    value.chars().all(|character| mark_of(character).is_none())
}

/// `single_line(value)` の中で、見えない字・改行・タブを替えた印のバイトの範囲（左から・重ならない）。元の値にあった
/// 本物の `¶`・`→` などは入らない（印かどうかは元の字で決める・VIEW-05・GitHub Issue #261）。
#[must_use]
pub fn mark_spans(value: &str) -> Vec<Range<usize>> {
    let mut spans = Vec::new();
    let mut shown = 0_usize;
    for (at, character) in shown_characters(value) {
        let end = shown.saturating_add(character.len_utf8());
        let original = value.get(at..).and_then(|rest| rest.chars().next());
        if original.and_then(mark_of).is_some() {
            spans.push(shown..end);
        }
        shown = end;
    }
    spans
}

/// 元の値 `value` の中の左から並んだ重ならないバイトの範囲 `spans` を、`single_line(value)` の中のバイトの範囲に写す
/// （検索で当たった字の印・GitHub Issue #189）。`\r\n` の間で切れる範囲は `¶` の後ろに寄せる。長さ 0 になった範囲は落とす。
#[must_use]
pub fn shown_spans(value: &str, spans: &[Range<usize>]) -> Vec<Range<usize>> {
    let points: Vec<usize> = spans
        .iter()
        .flat_map(|span| [span.start, span.end])
        .collect();
    let mut mapped = Vec::with_capacity(points.len());
    let mut shown = 0_usize;
    for (at, character) in shown_characters(value) {
        while points.get(mapped.len()).is_some_and(|&point| point <= at) {
            mapped.push(shown);
        }
        shown = shown.saturating_add(character.len_utf8());
    }
    mapped.resize(points.len(), shown);
    mapped
        .as_chunks::<2>()
        .0
        .iter()
        .filter(|&&[start, end]| start < end)
        .map(|&[start, end]| start..end)
        .collect()
}

/// 幅 `columns` に収めた表示の文字列 `text`（[`fitted`]）の中の、印を付ける範囲。`spans` は `text` の中の範囲。`…` で
/// 切った後ろにかかる範囲は、見えている頭の部分までと `…` に印を付ける（切った後ろに当たりがあることが見える）。
#[must_use]
pub fn fitted_spans(text: &str, fitted: &str, spans: &[Range<usize>]) -> Vec<Range<usize>> {
    // 🔴 バイトの長さでは決められない（3 バイトの `…` が 1 バイトの字 3 つと入れ替わると同じ長さになる）。
    let cut = fitted.len() != text.len() || fitted != text;
    let head = if cut {
        fitted.len().saturating_sub(ELLIPSIS.len_utf8())
    } else {
        fitted.len()
    };
    let mut marked: Vec<Range<usize>> = Vec::new();
    for span in spans {
        let start = span.start.min(head);
        let end = if span.end > head && cut {
            fitted.len()
        } else {
            span.end.min(head)
        };
        if start >= end {
            continue;
        }
        match marked.last_mut() {
            Some(last) if last.end >= start => last.end = last.end.max(end),
            Some(_) | None => marked.push(start..end),
        }
    }
    marked
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

    /// 1 つの範囲の並び。
    fn one(range: Range<usize>) -> Vec<Range<usize>> {
        core::iter::once(range).collect()
    }

    /// 元の値の範囲は、表示の 1 行の同じ字の範囲に写る（`\r\n` は 1 つの `¶`・全角空白は `□`）。
    #[test]
    fn spans_in_the_value_map_onto_the_single_line() {
        let value = "東京\r\nab\tc\u{3000}d";
        let shown = single_line(value);
        assert_eq!(shown, "東京¶ab→c□d");
        // 「ab」（元の 8..10）は表示の 8..10、「\r\n」（元の 6..8）は「¶」（表示の 6..8）、「□d」。
        assert_eq!(shown_spans(value, &one(8..10)), one(8..10));
        assert_eq!(shown_spans(value, &one(6..8)), one(6..8));
        assert_eq!(shown_spans(value, &[0..3, 12..16]), [0..3, 14..18]);
        assert_eq!(&shown[14..18], "□d");
        // `\r` だけの範囲は `¶` を含む。`\n` だけの範囲は `¶` の後ろに寄って長さ 0 になる。
        assert_eq!(shown_spans("a\r\nb", &one(1..2)), one(1..3));
        assert!(shown_spans("a\r\nb", &one(2..3)).is_empty());
        assert!(shown_spans("abc", &[]).is_empty());
    }

    /// 切った後ろにかかる当たりは、見えている頭と `…` に印を付ける。重なった印はまとめる。
    #[test]
    fn spans_past_the_cut_mark_the_ellipsis() {
        let text = "abcdefgh";
        let cut = fitted(text, 6);
        assert_eq!(cut, "abcd…");
        assert_eq!(fitted_spans(text, &cut, &one(1..2)), one(1..2));
        assert_eq!(fitted_spans(text, &cut, &one(3..6)), one(3..7));
        assert_eq!(fitted_spans(text, &cut, &one(6..7)), one(4..7));
        assert_eq!(fitted_spans(text, &cut, &[3..5, 6..7]), one(3..7));
        assert_eq!(fitted_spans("ab", "ab", &one(0..2)), one(0..2));
        // 切った後もバイトの長さが同じ（「abcde」→「ab…」はどちらも 5 バイト）でも、切ったと分かる。
        let same_length = fitted("abcde", 4);
        assert_eq!(same_length, "ab…");
        assert_eq!(fitted_spans("abcde", &same_length, &one(3..5)), one(2..5));
    }

    /// 写しを作らない確かめは、写しと比べたときと同じ答え。
    #[test]
    fn single_line_values_are_recognised_without_a_copy() {
        for value in [
            "",
            "abc",
            "東京",
            "a\r\nb",
            "a\rb",
            "a\nb",
            "a\tb",
            "\u{0}",
            "山田\u{3000}太郎",
            "10\u{a0}000",
            "ab\u{200b}c",
            "一\u{2028}二",
            "¶",
            "→",
            "\r",
            "\r\n",
            "x\u{7f}",
            "x\u{85}",
            "\u{2060}\u{feff}",
        ] {
            assert_eq!(
                is_single_line(value),
                single_line(value) == value,
                "{value:?}"
            );
        }
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

    /// 印の範囲は替えた字だけ。元の値にあった本物の `¶`・`→` は入らない。`\r\n` は 1 つの `¶`（VIEW-05・GitHub Issue
    /// #261）。
    #[test]
    fn mark_spans_cover_only_replaced_characters() {
        let value = "a\tb\r\nc¶→d\u{3000}e\u{a0}\u{0}";
        let shown = single_line(value);
        let marked: Vec<&str> = mark_spans(value)
            .iter()
            .map(|span| shown.get(span.clone()).unwrap())
            .collect();
        assert_eq!(marked, ["→", "¶", "□", "␣", "·"]);
        assert_eq!(shown, "a→b¶c¶→d□e␣·");
        assert!(mark_spans("plain ¶→·").is_empty());
        assert!(mark_spans("").is_empty());
    }
}
