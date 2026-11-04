//! 表をタブ区切りの文字列にする（Excel に貼る形）。

use alloc::string::String;
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

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
}
