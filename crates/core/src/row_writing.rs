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
/// # Errors
///
/// セルが無い（[`RowWriteFailure::NoSuchCell`]）・番号が昇順でない（[`RowWriteFailure::Unordered`]）。
pub fn edited_row(
    record: &Record,
    edits: Vec<(usize, String)>,
) -> Result<Vec<RowPiece>, RowWriteFailure> {
    let mut pieces = Vec::with_capacity(edits.len().saturating_mul(2).saturating_add(1));
    let mut cursor = record.raw().start();
    let mut previous: Option<usize> = None;
    for (column, text) in edits {
        if previous.is_some_and(|before| before >= column) {
            return Err(RowWriteFailure::Unordered(column));
        }
        previous = Some(column);
        let field = record
            .fields()
            .get(column)
            .ok_or(RowWriteFailure::NoSuchCell(column))?;
        push_source(&mut pieces, cursor, field.raw().start());
        pieces.push(RowPiece::Text(text));
        cursor = field.raw().end();
    }
    let end = record.raw().end().saturating_add(record.terminator().len());
    push_source(&mut pieces, cursor, end);
    Ok(pieces)
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
    use crate::dialect::Dialect;
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
        let pieces = edited_row(&record, vec![(1, String::from("y"))]).unwrap();
        assert_eq!(assembled(input, &pieces), b"\"0012\",y,\"a\"\"b\"\r\n");
        let none = edited_row(&record, Vec::new()).unwrap();
        assert_eq!(assembled(input, &none), b"\"0012\", x ,\"a\"\"b\"\r\n");
    }

    #[test]
    fn the_first_and_last_cells_can_be_replaced() {
        let input = b"a,b,c";
        let record = first_record(input);
        let edits = vec![(0, String::from("\"A\"")), (2, String::from("C"))];
        let pieces = edited_row(&record, edits).unwrap();
        assert_eq!(assembled(input, &pieces), b"\"A\",b,C");
    }

    #[test]
    fn missing_or_unordered_cells_are_refused() {
        let record = first_record(b"a,b\n");
        assert_eq!(
            edited_row(&record, vec![(2, String::new())]),
            Err(RowWriteFailure::NoSuchCell(2))
        );
        let twice = vec![(1, String::new()), (1, String::new())];
        assert_eq!(
            edited_row(&record, twice),
            Err(RowWriteFailure::Unordered(1))
        );
    }
}
