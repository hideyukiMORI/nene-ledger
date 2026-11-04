//! 値をペインの幅で折り返す。

use alloc::string::String;
use alloc::vec::Vec;

use nene_ledger_core::display_width::char_columns;

use crate::cell_display::single_line;

/// 折り返した値。出す行と、出しきれなかった行の数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrappedText {
    lines: Vec<String>,
    hidden: usize,
}

impl WrappedText {
    /// 出す行。
    #[must_use]
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// 出しきれなかった行の数。
    #[must_use]
    pub fn hidden(&self) -> usize {
        self.hidden
    }
}

/// `text` を改行（`\r\n`・`\n`・`\r`）で分け、各行を `max_columns`（半角の字数）で折り返して、最初の
/// `max_lines` 行を出す。タブと制御文字は表と同じ印にする（値は変えない・表示だけ）。幅が 0 でも 1 字ずつは出す。
#[must_use]
pub fn wrap(text: &str, max_columns: u32, max_lines: usize) -> WrappedText {
    let mut lines = Vec::new();
    let mut total = 0_usize;
    for logical in text.split("\r\n").flat_map(|part| part.split(['\n', '\r'])) {
        for line in wrap_line(&single_line(logical), max_columns) {
            if lines.len() < max_lines {
                lines.push(line);
            }
            total = total.saturating_add(1);
        }
    }
    WrappedText {
        hidden: total.saturating_sub(lines.len()),
        lines,
    }
}

/// 改行を含まない 1 行を、`max_columns` で折り返す。空の行も 1 行と数える。
fn wrap_line(line: &str, max_columns: u32) -> Vec<String> {
    let mut wrapped = Vec::new();
    let mut current = String::new();
    let mut width = 0_u32;
    for character in line.chars() {
        let columns = char_columns(character);
        if !current.is_empty() && width.saturating_add(columns) > max_columns {
            wrapped.push(core::mem::take(&mut current));
            width = 0;
        }
        current.push(character);
        width = width.saturating_add(columns);
    }
    wrapped.push(current);
    wrapped
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn lines(wrapped: &WrappedText) -> Vec<&str> {
        wrapped.lines().iter().map(String::as_str).collect()
    }

    #[test]
    fn long_lines_wrap_by_display_width() {
        let wrapped = wrap("abcdefgh", 3, 10);
        assert_eq!(lines(&wrapped), vec!["abc", "def", "gh"]);
        assert_eq!(wrapped.hidden(), 0);
        // 全角は 2 と数える。幅 5 に「東京都」は 2 字まで。
        assert_eq!(
            lines(&wrap("東京都千代田区", 5, 10)),
            vec!["東京", "都千", "代田", "区"]
        );
    }

    #[test]
    fn every_kind_of_line_break_starts_a_new_line() {
        let wrapped = wrap("a\r\nb\nc\rd\n\ne", 10, 10);
        assert_eq!(lines(&wrapped), vec!["a", "b", "c", "d", "", "e"]);
        assert_eq!(lines(&wrap("x\ty\u{0}", 10, 10)), vec!["x→y·"]);
    }

    #[test]
    fn lines_beyond_the_limit_are_counted() {
        let wrapped = wrap("1\n2\n3\n4\n5", 10, 2);
        assert_eq!(lines(&wrapped), vec!["1", "2"]);
        assert_eq!(wrapped.hidden(), 3);
        assert_eq!(wrap("", 10, 2).lines(), &[String::new()]);
    }

    #[test]
    fn a_zero_width_still_shows_one_character_per_line() {
        assert_eq!(lines(&wrap("ab", 0, 10)), vec!["a", "b"]);
    }
}
