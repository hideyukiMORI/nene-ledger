//! 書き換えた行を、原本のバイト列の区間と、書き換えたセルの文字列とで組み立てる規則（ADR 0024 の決定 4）。

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::byte_range::ByteRange;
use crate::column_id::ColumnId;
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

/// 列の構造が変わった行（列を足し消しした表・ADR 0027 の決定 5）。`columns` は表の列の順の同一性、`values` は
/// 書いた文字列（書き換えたセルと足した列のセル。`cell_writing::written_cell` で書いたもの）。
///
/// 原本のセルは原本のバイト列（引用符を含む）のまま写し、`values` にあるセルはその文字列、足した列で書いていない
/// セルは空のセルにする。区切りは `delimiter`、終わり方は原本のまま。書くのは、原本にあるセル・足した列・書いた
/// セルのうち最後の列まで（原本の行より後ろの、何も無い原本の列は書かない）。空の行になるとき（1 つのセルだけで
/// 空・セルが 1 つも残らない）は、`lone_empty`（引用の方言なら `""`）と書く（空の行にしない・ADR 0024 の決定 4）。
#[must_use]
pub fn restructured_row(
    record: &Record,
    columns: &[ColumnId],
    values: &BTreeMap<ColumnId, String>,
    (delimiter, lone_empty): (char, &str),
) -> Vec<RowPiece> {
    let present = |column: &ColumnId| match column {
        ColumnId::Source(source) => *source < record.fields().len() || values.contains_key(column),
        ColumnId::Added(_) => true,
    };
    let last = columns.iter().rposition(present);
    let mut pieces = Vec::new();
    let mut empty = true;
    for (index, column) in columns
        .iter()
        .enumerate()
        .take(last.map_or(0, |last| last + 1))
    {
        if index > 0 {
            pieces.push(RowPiece::Text(String::from(delimiter)));
        }
        let field = match column {
            ColumnId::Source(source) => record.fields().get(*source),
            ColumnId::Added(_) => None,
        };
        match (values.get(column), field) {
            (Some(text), _) => {
                empty &= text.is_empty();
                pieces.push(RowPiece::Text(text.clone()));
            }
            (None, Some(field)) => {
                empty &= field.raw().is_empty();
                push_source(&mut pieces, field.raw().start(), field.raw().end());
            }
            (None, None) => {}
        }
    }
    // セルが 1 つも残らない行も、空の行にしない（行が消えたように読まれる）。
    if empty && last.is_none_or(|last| last == 0) {
        pieces.clear();
        pieces.push(RowPiece::Text(String::from(lone_empty)));
    }
    let end = record.raw().end();
    push_source(
        &mut pieces,
        end,
        end.saturating_add(record.terminator().len()),
    );
    pieces
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
    fn restructured_rows_copy_source_cells_and_fill_added_columns() {
        let input = b"a,\"b,c\",d\r\nnext";
        let record = first_record(input);
        // 原本の列 0・足した列・原本の列 2（原本の列 1 は消した）。
        let columns = [ColumnId::Source(0), ColumnId::Added(0), ColumnId::Source(2)];
        let none = BTreeMap::new();
        let blank = restructured_row(&record, &columns, &none, (',', "\"\""));
        assert_eq!(assembled(input, &blank), b"a,,d\r\n");
        let mut values = BTreeMap::new();
        values.insert(ColumnId::Added(0), String::from("\"x,y\""));
        let filled = restructured_row(&record, &columns, &values, (',', "\"\""));
        assert_eq!(assembled(input, &filled), b"a,\"x,y\",d\r\n");
        // 原本の行より後ろの原本の列は書かない。足した列は書く。
        let short = first_record(b"1\n");
        let wide = [ColumnId::Source(0), ColumnId::Source(1), ColumnId::Added(3)];
        let widened = restructured_row(&short, &wide, &none, (',', "\"\""));
        assert_eq!(assembled(b"1\n", &widened), b"1,,\n");
        let narrow = [ColumnId::Source(0), ColumnId::Source(1)];
        let kept = restructured_row(&short, &narrow, &none, (',', "\"\""));
        assert_eq!(assembled(b"1\n", &kept), b"1\n");
        // 1 つのセルだけの空の行にはしない。
        let lone = [ColumnId::Added(0)];
        let quoted = restructured_row(&short, &lone, &none, (',', "\"\""));
        assert_eq!(assembled(b"1\n", &quoted), b"\"\"\n");
        let gone = [ColumnId::Source(1)];
        let emptied = restructured_row(&short, &gone, &none, (',', "\"\""));
        assert_eq!(assembled(b"1\n", &emptied), b"\"\"\n");
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
