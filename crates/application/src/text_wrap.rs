//! 値をペインの幅で折り返す。

use alloc::string::String;
use alloc::vec::Vec;

use nene_ledger_core::display_width::{char_columns, display_columns, extends_previous};

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

/// `text` を [`wrap`] と同じく折り返したときの、`index` 字目（0 から・改行も 1 字）の前の位置：折り返した何行目か
/// と、その行の左からの桁（半角 1・全角 2）。詳細ペインで直すときのカーソルの棒の位置（GitHub Issue #458）。
#[must_use]
pub fn caret_place(text: &str, max_columns: u32, index: usize) -> (usize, u32) {
    let (row, head) = caret_piece(text, max_columns, index);
    (row, display_columns(&head))
}

/// [`caret_place`] と同じ位置を、折り返した何行目かと、その行の左からカーソルまでの見せる字（表の印に替えた後）で
/// 言う。画素の位置は UI が測った送り幅でこの字を測る（GitHub Issue #462）。
#[must_use]
pub fn caret_piece(text: &str, max_columns: u32, index: usize) -> (usize, String) {
    let mut row = 0_usize;
    let mut before = 0_usize;
    for logical in text.split('\n') {
        let length = logical.chars().count();
        // `\r\n` の `\r` は改行の一部（`wrap` と同じく 1 つの改行に数える）。
        let pieces = wrap_line(&single_line(logical.trim_end_matches('\r')), max_columns);
        if index <= before.saturating_add(length) {
            let (offset, head) = place_in_pieces(&pieces, index.saturating_sub(before));
            return (row.saturating_add(offset), head);
        }
        row = row.saturating_add(pieces.len());
        before = before.saturating_add(length).saturating_add(1);
    }
    (row.saturating_sub(1), String::new())
}

/// [`caret_piece`] の逆。`text` を幅 `max_columns` で折り返したときの、`row` 行目の左から `x` の所に近い字の
/// 境目が何字目か（0 から・改行も 1 字）。`x` は字の送り幅 `advance` と同じ単位。行の右より右は行の末尾、最後の行より
/// 下は値の末尾。詳細ペインを押した所にカーソルの棒を置く（GitHub Issue #460・#462）。
#[must_use]
pub fn index_at<F: Fn(char) -> u32>(
    text: &str,
    max_columns: u32,
    (row, x): (usize, u32),
    advance: F,
) -> usize {
    let mut first_row = 0_usize;
    let mut before = 0_usize;
    for logical in text.split('\n') {
        let length = logical.chars().count();
        let pieces = wrap_line(&single_line(logical.trim_end_matches('\r')), max_columns);
        let rows = pieces.len();
        if let Some(piece) = row
            .checked_sub(first_row)
            .and_then(|offset| pieces.get(offset))
        {
            let skipped: usize = pieces
                .iter()
                .take(row.saturating_sub(first_row))
                .map(|earlier| earlier.chars().count())
                .sum();
            return before
                .saturating_add(skipped)
                .saturating_add(index_in_piece(piece, x, &advance));
        }
        first_row = first_row.saturating_add(rows);
        before = before.saturating_add(length).saturating_add(1);
    }
    text.chars().count()
}

/// 折り返した切れ端 `piece` の、左から `x` の所に近い字の境目（字の真ん中より右なら後ろ）。
fn index_in_piece<F: Fn(char) -> u32>(piece: &str, x: u32, advance: F) -> usize {
    let mut width = 0_u32;
    for (index, character) in piece.chars().enumerate() {
        let wide = advance(character);
        // 字の真ん中（2 倍で比べる）より左なら、その字の前。
        if x.saturating_mul(2) < width.saturating_mul(2).saturating_add(wide) {
            return index;
        }
        width = width.saturating_add(wide);
    }
    piece.chars().count()
}

/// 折り返した 1 つの論理の行 `pieces` の中の、`left` 字目の前の位置（何番目の切れ端か・その中の桁）。
fn place_in_pieces(pieces: &[String], mut left: usize) -> (usize, String) {
    for (offset, piece) in pieces.iter().enumerate() {
        let count = piece.chars().count();
        let last = offset.saturating_add(1) == pieces.len();
        if left < count || (left == count && last) {
            return (offset, piece.chars().take(left).collect());
        }
        left = left.saturating_sub(count);
    }
    (pieces.len().saturating_sub(1), String::new())
}

/// 改行を含まない 1 行を、`max_columns` で折り返す。空の行も 1 行と数える。前の字に重なる字（結合文字・異体字の
/// 選択子など）は、前の字から離さない（GitHub Issue #42）。ゼロ幅の結合子などの見えない字は、`single_line` が見える印に
/// 替えてあるので、ふつうの字として折り返す。
fn wrap_line(line: &str, max_columns: u32) -> Vec<String> {
    let mut wrapped = Vec::new();
    let mut current = String::new();
    let mut width = 0_u32;
    for character in line.chars() {
        let columns = char_columns(character);
        if !extends_previous(character)
            && !current.is_empty()
            && width.saturating_add(columns) > max_columns
        {
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

    /// 結合文字は前の字と同じ行に残す（GitHub Issue #42。前は `["ab", "\u{301}"]` に分けた）。
    #[test]
    fn combining_characters_stay_with_their_base() {
        assert_eq!(lines(&wrap("ab\u{301}", 2, 10)), vec!["ab\u{301}"]);
        assert_eq!(
            lines(&wrap("aか\u{3099}b", 3, 10)),
            vec!["aか\u{3099}", "b"]
        );
        assert_eq!(lines(&wrap("a👍\u{FE0F}", 3, 10)), vec!["a👍\u{FE0F}"]);
    }

    #[test]
    fn a_zero_width_still_shows_one_character_per_line() {
        assert_eq!(lines(&wrap("ab", 0, 10)), vec!["a", "b"]);
    }

    /// カーソルの棒の位置は、折り返しと改行をまたいでも折り返した行と桁で言う（GitHub Issue #458）。
    #[test]
    fn the_caret_is_placed_on_the_wrapped_line() {
        // 幅 4 で「abcdef」は「abcd」「ef」。改行の後の「gh」は 3 行目。
        let text = "abcdef\ngh";
        assert_eq!(caret_place(text, 4, 0), (0, 0));
        assert_eq!(caret_place(text, 4, 3), (0, 3));
        assert_eq!(caret_place(text, 4, 4), (1, 0));
        assert_eq!(caret_place(text, 4, 6), (1, 2));
        assert_eq!(caret_place(text, 4, 7), (2, 0));
        assert_eq!(caret_place(text, 4, 9), (2, 2));
    }

    /// 押した行と桁から字の番目へ。字の真ん中より右は後ろ・行の右は行の末尾・下は値の末尾（GitHub Issue #460）。
    #[test]
    fn the_index_is_found_from_the_wrapped_row_and_column() {
        // （値・幅・行・桁・字の番目）。幅 4 で「abcdef\ngh」は「abcd」「ef」「gh」。全角は 2 桁で、1 桁目（字の
        // 真ん中）より右なら後ろ。
        let cases: [(&str, u32, usize, u32, usize); 12] = [
            ("abcdef\ngh", 4, 0, 0, 0),
            ("abcdef\ngh", 4, 0, 3, 3),
            ("abcdef\ngh", 4, 1, 1, 5),
            ("abcdef\ngh", 4, 1, 9, 6),
            ("abcdef\ngh", 4, 2, 0, 7),
            ("abcdef\ngh", 4, 2, 9, 9),
            ("abcdef\ngh", 4, 7, 0, 9),
            ("あい", 10, 0, 0, 0),
            ("あい", 10, 0, 1, 1),
            ("あい", 10, 0, 2, 1),
            ("ab\r\ncd", 10, 1, 1, 5),
            ("", 10, 3, 3, 0),
        ];
        for (text, width, row, column, expected) in cases {
            assert_eq!(
                index_at(text, width, (row, column), char_columns),
                expected,
                "{text:?} {row} {column}"
            );
        }
    }

    /// 字の番目から行と桁へ、行と桁から字の番目へ戻すと、同じ行と桁に戻る（GitHub Issue #460）。
    #[test]
    fn the_index_and_the_caret_place_round_trip() {
        let text = "abcdef\ngh";
        for index in 0..=9 {
            let (row, column) = caret_place(text, 4, index);
            let back = index_at(text, 4, (row, column), char_columns);
            assert_eq!(caret_place(text, 4, back), (row, column));
        }
    }

    /// 全角は 2 桁・空の値は先頭・`\r\n` は 1 つの改行（GitHub Issue #458）。
    #[test]
    fn the_caret_counts_wide_characters_and_line_breaks() {
        assert_eq!(caret_place("あいう", 10, 2), (0, 4));
        assert_eq!(caret_place("", 10, 0), (0, 0));
        assert_eq!(caret_place("ab\n", 10, 3), (1, 0));
        assert_eq!(caret_place("ab\r\ncd", 10, 5), (1, 1));
    }
}
