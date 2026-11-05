//! 書き換えた行を、原本のバイト列の区間と、書き換えたセルの文字列とで組み立てる規則（ADR 0024 の決定 4）。

use alloc::string::String;
use alloc::vec::Vec;

use crate::byte_range::ByteRange;
use crate::record::Record;
use crate::row_piece::RowPiece;
use crate::row_write_failure::RowWriteFailure;

/// レコード `record` のうち、`edits`（セルの番号の昇順・書いた文字列）のセルだけを差し替えた行。
///
/// 差し替えないセル・区切り・終わり方は、レコードのバイト列（`record` の区間が指す入力）の区間として写す。
/// 区間はレコードの頭からの位置で、終わりは終わり方（改行）を含む。引用符はセルの区間に含まれるので、
/// 差し替えたセルの引用は `edits` の文字列が持つ（KEEP-03）。
///
/// 原本の行に無いセル（列の数が足りない行・OPEN-14）は、行の終わり（終わり方の前）まで写してから、
/// 区切り `delimiter` と空のセルで間の列を埋めて足す（ADR 0024 の施主の判断 4・5）。
///
/// # Errors
///
/// 番号が昇順でない（[`RowWriteFailure::Unordered`]）。
pub fn edited_row(
    record: &Record,
    edits: Vec<(usize, String)>,
    delimiter: char,
) -> Result<Vec<RowPiece>, RowWriteFailure> {
    let mut pieces = Vec::with_capacity(edits.len().saturating_mul(2).saturating_add(1));
    let mut cursor = record.raw().start();
    let mut previous: Option<usize> = None;
    // 行の今のセルの数（足したセルを含む）。
    let mut width = record.fields().len();
    for (column, text) in edits {
        if previous.is_some_and(|before| before >= column) {
            return Err(RowWriteFailure::Unordered(column));
        }
        previous = Some(column);
        if let Some(field) = record.fields().get(column) {
            push_source(&mut pieces, cursor, field.raw().start());
            pieces.push(RowPiece::Text(text));
            cursor = field.raw().end();
            continue;
        }
        push_source(&mut pieces, cursor, record.raw().end());
        cursor = record.raw().end();
        pieces.push(RowPiece::Text(appended((width, column), &text, delimiter)));
        width = column.saturating_add(1);
    }
    let end = record.raw().end().saturating_add(record.terminator().len());
    push_source(&mut pieces, cursor, end);
    Ok(pieces)
}

/// セルが `width` 個の行の後ろに足す文字列。`column` 番目に `text` を置き、間の列は空のセル。
/// 先頭のセル（0 番目）の前には区切りを書かない。
fn appended((width, column): (usize, usize), text: &str, delimiter: char) -> String {
    let mut out = String::new();
    for index in width..=column {
        if index > 0 {
            out.push(delimiter);
        }
        if index == column {
            out.push_str(text);
        }
    }
    out
}

/// `start..end` が空でなければ、写す区間として足す。
fn push_source(pieces: &mut Vec<RowPiece>, start: usize, end: usize) {
    if let Some(range) = ByteRange::new(start, end).filter(|range| !range.is_empty()) {
        pieces.push(RowPiece::Source(range));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delimiter::Delimiter;
    use crate::dialect::Dialect;
    use crate::quoting::Quoting;
    use crate::records::Records;
    use alloc::vec;

    fn first_record(input: &[u8]) -> Record {
        Records::new(input, Dialect::rfc4180()).next().unwrap()
    }

    fn assembled(input: &[u8], pieces: &[RowPiece]) -> Vec<u8> {
        let mut out = Vec::new();
        for piece in pieces {
            match piece {
                RowPiece::Source(range) => out.extend_from_slice(range.slice(input).unwrap()),
                RowPiece::Text(text) => out.extend_from_slice(text.as_bytes()),
            }
        }
        out
    }

    #[test]
    fn untouched_cells_and_delimiters_are_copied_byte_for_byte() {
        let input = b"\"0012\", x ,\"a\"\"b\"\r\nnext\n";
        let record = first_record(input);
        let pieces = edited_row(&record, vec![(1, String::from("y"))], ',').unwrap();
        assert_eq!(assembled(input, &pieces), b"\"0012\",y,\"a\"\"b\"\r\n");
        let none = edited_row(&record, Vec::new(), ',').unwrap();
        assert_eq!(assembled(input, &none), b"\"0012\", x ,\"a\"\"b\"\r\n");
    }

    #[test]
    fn the_first_and_last_cells_can_be_replaced() {
        let input = b"a,b,c";
        let record = first_record(input);
        let edits = vec![(0, String::from("\"A\"")), (2, String::from("C"))];
        let pieces = edited_row(&record, edits, ',').unwrap();
        assert_eq!(assembled(input, &pieces), b"\"A\",b,C");
    }

    #[test]
    fn unordered_cells_are_refused() {
        let record = first_record(b"a,b\n");
        let twice = vec![(1, String::new()), (1, String::new())];
        assert_eq!(
            edited_row(&record, twice, ','),
            Err(RowWriteFailure::Unordered(1))
        );
    }

    #[test]
    fn missing_cells_are_appended_after_empty_cells_and_before_the_line_end() {
        let input = b"a,\"b\"\r\nnext\n";
        let record = first_record(input);
        let pieces = edited_row(&record, vec![(3, String::from("x"))], ',').unwrap();
        assert_eq!(assembled(input, &pieces), b"a,\"b\",,x\r\n");
        // すぐ後ろの列は空のセルを挟まない。足したセルの後ろへさらに足すときも同じ。
        let edits = vec![
            (0, String::from("A")),
            (2, String::from("c")),
            (4, String::from("e")),
        ];
        let spread = edited_row(&record, edits, ',').unwrap();
        assert_eq!(assembled(input, &spread), b"A,\"b\",c,,e\r\n");
        // 終わり方の無い最後の行・方言の区切り。
        let tab = b"a\tb";
        let dialect = Dialect::new(Delimiter::new(b'\t').unwrap(), Quoting::Doubled, b'"').unwrap();
        let tab_record = Records::new(tab, dialect).next().unwrap();
        let tabbed = edited_row(&tab_record, vec![(2, String::from("\"q,\""))], '\t').unwrap();
        assert_eq!(assembled(tab, &tabbed), b"a\tb\t\"q,\"");
    }
}
