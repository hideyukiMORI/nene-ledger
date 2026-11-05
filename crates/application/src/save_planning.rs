//! 原本 ＋ セルの書き換えから、保存の計画を作る（ADR 0024 の決定 4・ADR 0025 の決定 1）。

use alloc::string::String;
use alloc::vec::Vec;

use nene_ledger_core::cell_edits::CellEdits;
use nene_ledger_core::cell_value::CellValue;
use nene_ledger_core::cell_writing::{CellWriteFailure, written_cell};
use nene_ledger_core::column_id::ColumnId;
use nene_ledger_core::dialect::Dialect;
use nene_ledger_core::record::Record;
use nene_ledger_core::row_id::RowId;
use nene_ledger_core::row_piece::RowPiece;
use nene_ledger_core::row_write_failure::RowWriteFailure;
use nene_ledger_core::row_writing::edited_row;
use nene_ledger_core::save_plan::SavePlan;

use crate::record_lookup::RecordLookup;
use crate::save_plan_failure::SavePlanFailure;

/// 原本（長さ `source_length`）にセルの書き換え `edits` を重ねた内容を書く、保存の計画。
///
/// 書き換えていない行は原本の区間として写し、書き換えた行は `edited_row` で組み立てる。
/// 🔴 書き換えが無ければ、原本の全体を写す 1 歩になる（KEEP-02）。
///
/// # Errors
///
/// [`SavePlanFailure`] のとおり。
pub fn save_plan<L: RecordLookup>(
    edits: &CellEdits,
    lookup: &mut L,
    source_length: u64,
    dialect: Dialect,
) -> Result<SavePlan, SavePlanFailure<L::Failure>> {
    let mut plan = SavePlan::empty();
    let mut cursor = 0_u64;
    for (row, cells) in edits.rows() {
        let RowId::Source(number) = row else {
            return Err(SavePlanFailure::Unsupported);
        };
        let located = lookup
            .record(number)
            .map_err(SavePlanFailure::Lookup)?
            .ok_or(SavePlanFailure::Missing(number))?;
        if !located.is_whole() {
            return Err(SavePlanFailure::TooLong(number));
        }
        let pieces = row_pieces(number, located.record(), &cells, dialect)?;
        plan.copy(cursor, located.offset().saturating_sub(cursor));
        cursor = located.offset();
        for piece in pieces {
            match piece {
                RowPiece::Source(range) => {
                    let start = located.offset().saturating_add(widen(range.start()));
                    plan.copy(start, widen(range.len()));
                    cursor = start.saturating_add(widen(range.len()));
                }
                RowPiece::Text(text) => plan.text(&text),
            }
        }
        cursor = cursor.max(
            located
                .offset()
                .saturating_add(record_end(located.record())),
        );
    }
    plan.copy(cursor, source_length.saturating_sub(cursor));
    Ok(plan)
}

/// 書き換えた 1 行の部分の列。
fn row_pieces<F>(
    number: u64,
    record: &Record,
    cells: &[(ColumnId, &CellValue)],
    dialect: Dialect,
) -> Result<Vec<RowPiece>, SavePlanFailure<F>> {
    let mut written: Vec<(usize, String)> = Vec::with_capacity(cells.len());
    for &(column, value) in cells {
        let (ColumnId::Source(column), CellValue::Text(text)) = (column, value) else {
            return Err(SavePlanFailure::Unsupported);
        };
        // 原本の行に無いセルは足す（間は空のセル・ADR 0024 の施主の判断 4・5）。引用は要るときだけ。
        let was_quoted = record
            .fields()
            .get(column)
            .is_some_and(|field| field.quoted());
        let lone_empty = text.is_empty() && record.fields().len() == 1;
        let cell = written_cell(
            text,
            was_quoted || lone_empty,
            &dialect,
            record.terminator(),
        )
        .map_err(|failure| match failure {
            CellWriteFailure::NeedsQuoting => SavePlanFailure::Unwritable {
                row: number,
                column,
            },
        })?;
        written.push((column, cell));
    }
    let delimiter = char::from(dialect.delimiter().byte());
    edited_row(record, written, delimiter).map_err(|failure| match failure {
        RowWriteFailure::Unordered(column) => SavePlanFailure::Unordered {
            row: number,
            column,
        },
    })
}

/// レコードの終わり（終わり方を含む）の、レコードの頭からの位置。
fn record_end(record: &Record) -> u64 {
    widen(record.raw().end().saturating_add(record.terminator().len()))
}

/// `usize` を `u64` に広げる（64 ビットより広い `usize` は無いので飽和は起きない）。
fn widen(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::located_record::LocatedRecord;
    use nene_ledger_core::records::Records;
    use nene_ledger_core::save_step::SaveStep;

    /// 入力を記憶に持ち、参照実装で切って引く。
    struct InMemory<'a> {
        input: &'a [u8],
        lookups: usize,
    }

    impl RecordLookup for InMemory<'_> {
        type Failure = ();

        fn record(&mut self, number: u64) -> Result<Option<LocatedRecord>, ()> {
            self.lookups += 1;
            let start = if self.input.starts_with(b"\xEF\xBB\xBF") {
                3
            } else {
                0
            };
            let body = self.input.get(start..).unwrap_or_default();
            let nth = usize::try_from(number).unwrap_or(usize::MAX);
            let Some(record) = Records::new(body, Dialect::rfc4180()).nth(nth) else {
                return Ok(None);
            };
            let bytes = body
                .get(record.raw().start()..)
                .unwrap_or_default()
                .to_vec();
            let Some(again) = Records::new(&bytes, Dialect::rfc4180()).next() else {
                return Ok(None);
            };
            let offset = u64::try_from(start + record.raw().start()).unwrap_or(u64::MAX);
            Ok(Some(LocatedRecord::new(offset, bytes, again, true)))
        }
    }

    fn apply(input: &[u8], plan: &SavePlan) -> Vec<u8> {
        let mut out = Vec::new();
        for step in plan.steps() {
            match step {
                SaveStep::Copy { offset, length } => {
                    let start = usize::try_from(*offset).unwrap();
                    out.extend_from_slice(&input[start..start + usize::try_from(*length).unwrap()]);
                }
                SaveStep::Text(text) => out.extend_from_slice(text.as_bytes()),
            }
        }
        out
    }

    fn saved(input: &[u8], edits: &CellEdits) -> Vec<u8> {
        let mut lookup = InMemory { input, lookups: 0 };
        let length = u64::try_from(input.len()).unwrap();
        apply(
            input,
            &save_plan(edits, &mut lookup, length, Dialect::rfc4180()).unwrap(),
        )
    }

    fn text(value: &str) -> CellValue {
        CellValue::Text(String::from(value))
    }

    #[test]
    fn without_edits_the_whole_source_is_one_copy() {
        let input = b"\xEF\xBB\xBFa,\"b\"\r\n0012,1-2\n";
        let mut lookup = InMemory { input, lookups: 0 };
        let plan = save_plan(&CellEdits::empty(), &mut lookup, 22, Dialect::rfc4180()).unwrap();
        assert_eq!(
            plan.steps(),
            [SaveStep::Copy {
                offset: 0,
                length: 22
            }]
        );
        assert_eq!(lookup.lookups, 0);
    }

    #[test]
    fn one_cell_is_rewritten_and_everything_else_is_kept() {
        let input = b"\xEF\xBB\xBFid,name\r\n0012,\"Sato\"\r\n0013, x \r\nlast";
        let mut edits = CellEdits::empty();
        edits.set(RowId::Source(1), ColumnId::Source(1), text("Su,zuki"));
        assert_eq!(
            saved(input, &edits),
            b"\xEF\xBB\xBFid,name\r\n0012,\"Su,zuki\"\r\n0013, x \r\nlast"
        );
        let mut last = CellEdits::empty();
        last.set(RowId::Source(3), ColumnId::Source(0), text("a\nb"));
        assert_eq!(
            saved(input, &last),
            b"\xEF\xBB\xBFid,name\r\n0012,\"Sato\"\r\n0013, x \r\n\"a\nb\""
        );
    }

    #[test]
    fn several_rows_keep_their_line_endings() {
        let input = b"a,b\nc,d\r\ne,f\n";
        let mut edits = CellEdits::empty();
        edits.set(RowId::Source(0), ColumnId::Source(0), text("A"));
        edits.set(RowId::Source(1), ColumnId::Source(1), text("x\ny"));
        edits.set(RowId::Source(2), ColumnId::Source(1), text(""));
        assert_eq!(saved(input, &edits), b"A,b\nc,\"x\r\ny\"\r\ne,\n");
    }

    #[test]
    fn emptying_a_lone_cell_after_a_carriage_return_keeps_the_row() {
        let input = b"a\rb\nc";
        let mut edits = CellEdits::empty();
        edits.set(RowId::Source(1), ColumnId::Source(0), text(""));
        assert_eq!(saved(input, &edits), b"a\r\"\"\nc");
    }

    #[test]
    fn unsupported_and_missing_edits_are_refused() {
        let input = b"a,b\n";
        let plan = |edits: &CellEdits| {
            let mut lookup = InMemory { input, lookups: 0 };
            save_plan(edits, &mut lookup, 4, Dialect::rfc4180())
        };
        let mut added = CellEdits::empty();
        added.set(RowId::Added(0), ColumnId::Source(0), text("x"));
        assert_eq!(plan(&added), Err(SavePlanFailure::Unsupported));
        let mut absent = CellEdits::empty();
        absent.set(RowId::Source(0), ColumnId::Source(0), CellValue::Absent);
        assert_eq!(plan(&absent), Err(SavePlanFailure::Unsupported));
        let mut missing = CellEdits::empty();
        missing.set(RowId::Source(4), ColumnId::Source(0), text("x"));
        assert_eq!(plan(&missing), Err(SavePlanFailure::Missing(4)));
    }

    #[test]
    fn cells_beyond_a_short_row_are_appended_after_empty_cells() {
        // 1 行目は 2 列、2 行目は 1 列。2 行目の 3 列目に書くと、間の 2 列目は空のセルになる。
        let input = b"a,b,c\r\n\"d\"\r\ne,f,g\r\n";
        let mut edits = CellEdits::empty();
        edits.set(RowId::Source(1), ColumnId::Source(2), text("x,y"));
        assert_eq!(
            saved(input, &edits),
            b"a,b,c\r\n\"d\",,\"x,y\"\r\ne,f,g\r\n"
        );
        // すぐ後ろの列は区切り 1 つ。終わり方の無い最後の行。
        let mut next = CellEdits::empty();
        next.set(RowId::Source(0), ColumnId::Source(1), text("2"));
        assert_eq!(saved(b"1", &next), b"1,2");
    }

    /// 決まった種から作る疑似乱数（テストの決定性）。
    struct Xorshift(u64);

    impl Xorshift {
        fn below(&mut self, bound: usize) -> usize {
            self.0 ^= self.0 << 13_u32;
            self.0 ^= self.0 >> 7_u32;
            self.0 ^= self.0 << 17_u32;
            usize::try_from(self.0 % u64::try_from(bound).unwrap()).unwrap()
        }

        fn pick<'a>(&mut self, choices: &[&'a str]) -> &'a str {
            choices[self.below(choices.len())]
        }
    }

    const CELLS: [&str; 7] = [
        "a",
        "",
        "\"q\"\"x\"",
        "\"a,b\"",
        "0012",
        " 1-2 ",
        "\"l\r\nm\"",
    ];
    const VALUES: [&str; 7] = ["z", "", "a,b", "say \"hi\"", "p\nq", "x\r\ny", "  "];
    const ENDINGS: [&str; 3] = ["\n", "\r\n", "\r"];

    fn random_table(rng: &mut Xorshift) -> Vec<u8> {
        let mut out = String::new();
        let rows = 1 + rng.below(8);
        for row in 0..rows {
            let cells = 1 + rng.below(4);
            let line: Vec<&str> = (0..cells).map(|_| rng.pick(&CELLS)).collect();
            out.push_str(&line.join(","));
            if row + 1 < rows || rng.below(2) == 0 {
                out.push_str(rng.pick(&ENDINGS));
            }
        }
        out.into_bytes()
    }

    fn values(input: &[u8], record: &Record) -> Vec<Vec<u8>> {
        record
            .fields()
            .iter()
            .map(|&field| nene_ledger_core::value::value(input, field, &Dialect::rfc4180()))
            .collect()
    }

    fn expected_value(value: &str, record: &Record) -> Vec<u8> {
        let unified = value.replace("\r\n", "\n").replace('\r', "\n");
        let line_break = match record.terminator().bytes() {
            b"" => "\n",
            other => core::str::from_utf8(other).unwrap(),
        };
        unified.replace('\n', line_break).into_bytes()
    }

    /// 1 行の値が、書き換えたセルだけ変わり、書き換えていない行はバイト列のまま。
    fn assert_row(
        row: usize,
        (old, input): (&Record, &[u8]),
        (new, output): (&Record, &[u8]),
        expected: &[(usize, usize, &str)],
    ) {
        let edited: Vec<(usize, &str)> = expected
            .iter()
            .filter(|&&(r, _, _)| r == row)
            .map(|&(_, column, value)| (column, value))
            .collect();
        let mut want = values(input, old);
        for &(column, value) in &edited {
            // 原本の行に無いセルは足す。間の列は空のセル。
            if want.len() <= column {
                want.resize(column + 1, Vec::new());
            }
            want[column] = expected_value(value, old);
        }
        assert_eq!(values(output, new), want, "{input:?} -> {output:?}");
        assert_eq!(old.terminator(), new.terminator());
        if edited.is_empty() {
            assert_eq!(old.raw().slice(input), new.raw().slice(output));
        }
    }

    #[test]
    fn random_edits_change_only_the_edited_cells() {
        let mut rng = Xorshift(0x5EED_1234_ABCD_0001);
        for _ in 0..2_000_u32 {
            let input = random_table(&mut rng);
            let before: Vec<Record> = Records::new(&input, Dialect::rfc4180()).collect();
            if before.is_empty() {
                continue;
            }
            let mut edits = CellEdits::empty();
            let mut expected: Vec<(usize, usize, &str)> = Vec::new();
            for _ in 0..rng.below(4) {
                let row = rng.below(before.len());
                // 行のセルの数より 2 つ先まで（列の数が足りない行への書き込み・GitHub Issue #83）。
                let column = rng.below(before[row].fields().len() + 2);
                let value = rng.pick(&VALUES);
                let row_id = RowId::Source(u64::try_from(row).unwrap());
                edits.set(row_id, ColumnId::Source(column), text(value));
                expected.retain(|&(r, c, _)| (r, c) != (row, column));
                expected.push((row, column, value));
            }
            let output = saved(&input, &edits);
            let after: Vec<Record> = Records::new(&output, Dialect::rfc4180()).collect();
            assert_eq!(after.len(), before.len(), "{input:?} -> {output:?}");
            for (row, (old, new)) in before.iter().zip(&after).enumerate() {
                assert_row(row, (old, &input), (new, &output), &expected);
            }
        }
    }
}
