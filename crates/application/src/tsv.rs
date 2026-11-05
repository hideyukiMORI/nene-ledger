//! 表をタブ区切りの文字列にする（Excel に貼る形）。Excel からの貼り付けの文字列を表に戻す。

use alloc::string::String;
use alloc::vec::Vec;
use core::ops::RangeInclusive;

/// 行 `rows` の、列 `columns` だけをタブ区切りにする。行の区切りは CRLF（Excel の形）で、最後の行にも付ける。
/// 欠けたセルは空。値は変えない（型を推し量らない・KEEP-01）。
#[must_use]
pub fn tsv(rows: &[&[String]], columns: &[RangeInclusive<usize>]) -> String {
    let mut text = String::new();
    for row in rows {
        for (index, column) in columns.iter().cloned().flatten().enumerate() {
            if index > 0 {
                text.push('\t');
            }
            if let Some(cell) = row.get(column) {
                push_field(&mut text, cell);
            }
        }
        text.push_str("\r\n");
    }
    text
}

/// 1 つのセル。タブ・改行・`"` を含めば `"` で囲み、中の `"` は二重にする。
fn push_field(text: &mut String, cell: &str) {
    if !cell.contains(['\t', '\n', '\r', '"']) {
        text.push_str(cell);
        return;
    }
    text.push('"');
    for character in cell.chars() {
        if character == '"' {
            text.push('"');
        }
        text.push(character);
    }
    text.push('"');
}

/// クリップボードのタブ区切りの文字列 `text` を、行とセルに分ける（[`tsv`] の逆・Excel の形）。
///
/// - 行の区切りは CRLF・LF・CR。最後の行の後ろの改行は空の行にしない（Excel は最後の行にも付ける）
/// - `"` で始まるセルは、`"` で閉じるまでをそのまま読む（中のタブと改行も値・`""` は `"`）。閉じた後ろの字は値に続ける。
///   閉じないまま終われば、`"` から後ろを字のまま値にする（勝手に引用を解かない）
/// - `"` で始まらないセルの中の `"` は字のまま
/// - 値は変えない（型を推し量らない・KEEP-01・CLIP-01）。空の文字列は行が無い
#[must_use]
pub fn parse_tsv(text: &str) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        let (cell, after) = field(rest);
        row.push(cell);
        match after.chars().next() {
            Some('\t') => rest = after.get(1..).unwrap_or_default(),
            Some(_) => {
                rows.push(core::mem::take(&mut row));
                rest = after_line_break(after);
            }
            None => {
                rows.push(core::mem::take(&mut row));
                rest = after;
            }
        }
        // 区切りのタブで終われば、最後のセルは空。
        if rest.is_empty() && after.starts_with('\t') {
            row.push(String::new());
            rows.push(core::mem::take(&mut row));
        }
    }
    rows
}

/// `text` の頭の 1 つのセルと、その後ろ（タブか改行か空）。
fn field(text: &str) -> (String, &str) {
    if let Some(quoted) = text.strip_prefix('"')
        && let Some((value, after)) = closed_quote(quoted)
    {
        let end = after.find(['\t', '\r', '\n']).unwrap_or(after.len());
        let (tail, rest) = after.split_at(end);
        return (value + tail, rest);
    }
    let end = text.find(['\t', '\r', '\n']).unwrap_or(text.len());
    let (value, rest) = text.split_at(end);
    (String::from(value), rest)
}

/// 引用の中 `quoted`（頭の `"` の後ろ）を閉じる `"` まで読み、値と閉じた後ろを返す。閉じなければ `None`。
fn closed_quote(quoted: &str) -> Option<(String, &str)> {
    let mut value = String::new();
    let mut characters = quoted.char_indices().peekable();
    while let Some((index, character)) = characters.next() {
        if character != '"' {
            value.push(character);
            continue;
        }
        if characters.peek().is_some_and(|&(_, next)| next == '"') {
            let _doubled = characters.next();
            value.push('"');
            continue;
        }
        return Some((
            value,
            quoted.get(index.saturating_add(1)..).unwrap_or_default(),
        ));
    }
    None
}

/// 行の頭の改行（CRLF・LF・CR）の後ろ。
fn after_line_break(text: &str) -> &str {
    text.strip_prefix("\r\n")
        .or_else(|| text.strip_prefix('\n'))
        .or_else(|| text.strip_prefix('\r'))
        .unwrap_or(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(cells: &[&[&str]]) -> Vec<Vec<String>> {
        cells
            .iter()
            .map(|row| row.iter().map(|&cell| String::from(cell)).collect())
            .collect()
    }

    fn slices(rows: &[Vec<String>]) -> Vec<&[String]> {
        rows.iter().map(Vec::as_slice).collect()
    }

    #[test]
    fn rows_become_tab_separated_lines_with_crlf() {
        let rows = table(&[&["a", "b", "c"], &["1", "2", "3"]]);
        assert_eq!(
            tsv(&slices(&rows), &[RangeInclusive::new(0, 2)]),
            "a\tb\tc\r\n1\t2\t3\r\n"
        );
        assert_eq!(
            tsv(&slices(&rows), &[RangeInclusive::new(1, 2)]),
            "b\tc\r\n2\t3\r\n"
        );
        assert_eq!(
            tsv(&slices(&rows), &[RangeInclusive::new(1, 1)]),
            "b\r\n2\r\n"
        );
    }

    #[test]
    fn values_are_kept_and_special_cells_are_quoted() {
        let rows = table(&[&["007", "1.20E+03", "東京\n都", "a\tb", "say \"hi\"", "x\ry"]]);
        assert_eq!(
            tsv(&slices(&rows), &[RangeInclusive::new(0, 5)]),
            "007\t1.20E+03\t\"東京\n都\"\t\"a\tb\"\t\"say \"\"hi\"\"\"\t\"x\ry\"\r\n"
        );
    }

    #[test]
    fn missing_cells_are_empty() {
        let rows = table(&[&["a"], &["1", "2"]]);
        assert_eq!(
            tsv(&slices(&rows), &[RangeInclusive::new(0, 1)]),
            "a\t\r\n1\t2\r\n"
        );
        assert_eq!(tsv(&[], &[RangeInclusive::new(0, 1)]), "");
    }

    fn owned(cells: &[&[&str]]) -> Vec<Vec<String>> {
        table(cells)
    }

    /// Excel の形のタブ区切りを行とセルに戻す。最後の改行は行にしない。値は変えない（CLIP-01・KEEP-01）。
    #[test]
    fn excel_text_is_split_into_rows_and_cells() {
        assert_eq!(parse_tsv(""), Vec::<Vec<String>>::new());
        assert_eq!(parse_tsv("0012"), owned(&[&["0012"]]));
        assert_eq!(parse_tsv("0012\r\n"), owned(&[&["0012"]]));
        assert_eq!(
            parse_tsv("a\tb\r\n1\t2\r\n"),
            owned(&[&["a", "b"], &["1", "2"]])
        );
        assert_eq!(parse_tsv("a\nb\rc"), owned(&[&["a"], &["b"], &["c"]]));
        assert_eq!(parse_tsv("a\t\tc"), owned(&[&["a", "", "c"]]));
        assert_eq!(parse_tsv("a\t"), owned(&[&["a", ""]]));
        assert_eq!(parse_tsv("\r\n"), owned(&[&[""]]));
        assert_eq!(
            parse_tsv("1.20E+03\t2026-11-06"),
            owned(&[&["1.20E+03", "2026-11-06"]])
        );
    }

    /// 引用は Excel と同じに解く。閉じない引用と、頭に無い `"` は字のまま。
    #[test]
    fn quoted_cells_keep_tabs_newlines_and_quotes() {
        assert_eq!(
            parse_tsv("\"a\tb\"\t\"line1\nline2\"\r\n\"say \"\"hi\"\"\"\r\n"),
            owned(&[&["a\tb", "line1\nline2"], &["say \"hi\""]])
        );
        assert_eq!(parse_tsv("\"abc"), owned(&[&["\"abc"]]));
        assert_eq!(parse_tsv("ab\"c\td"), owned(&[&["ab\"c", "d"]]));
        assert_eq!(parse_tsv("\"a\"b\tc"), owned(&[&["ab", "c"]]));
    }

    /// 書いた文字列を戻すと元の行になる（欠けたセルは空）。
    #[test]
    fn copied_text_round_trips() {
        let rows = table(&[
            &["社員番号", "氏名", "備考"],
            &["0012", "Smith, John", "月末\t締め"],
            &["", "\"二重\"", "東京\r\n丸の内"],
        ]);
        let text = tsv(&slices(&rows), core::slice::from_ref(&(0..=2)));
        assert_eq!(parse_tsv(&text), rows);
    }

    /// 無作為の表（タブ・CR・LF・引用符・空のセル・日本語）を書いて戻すと元の行になる（貼り付けの読み方の往復・
    /// 1 列の空のセルの行も、空の行から空のセル 1 つに戻る）。
    #[test]
    fn random_tables_round_trip() {
        const PIECES: [&str; 10] = ["a", "\t", "\r", "\n", "\r\n", "\"", "", "東", "0012", " "];
        let mut state = 0x2545_F491_4F6C_DD1D_u64;
        let mut next = |bound: usize| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            usize::try_from(state % u64::try_from(bound).unwrap()).unwrap()
        };
        for _ in 0..2_000_u32 {
            let columns = 1 + next(4);
            let rows: Vec<Vec<String>> = (0..1 + next(4))
                .map(|_| {
                    (0..columns)
                        .map(|_| (0..next(4)).map(|_| PIECES[next(PIECES.len())]).collect())
                        .collect()
                })
                .collect();
            let text = tsv(&slices(&rows), core::slice::from_ref(&(0..=columns - 1)));
            assert_eq!(parse_tsv(&text), rows, "{text:?}");
        }
    }
}
