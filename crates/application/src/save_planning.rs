//! 原本 ＋ 差分から、保存の計画を作る（ADR 0024 の決定 4・ADR 0025 の決定 1・ADR 0027 の決定 5）。

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use nene_ledger_core::cell_edits::CellEdits;
use nene_ledger_core::cell_value::CellValue;
use nene_ledger_core::cell_writing::{CellWriteFailure, written_cell};
use nene_ledger_core::column_id::ColumnId;
use nene_ledger_core::dialect::Dialect;
use nene_ledger_core::line_ending::LineEnding;
use nene_ledger_core::order_run::OrderRun;
use nene_ledger_core::plan_sink::PlanSink;
use nene_ledger_core::record::Record;
use nene_ledger_core::row_id::RowId;
use nene_ledger_core::row_piece::RowPiece;
use nene_ledger_core::row_write_failure::RowWriteFailure;
use nene_ledger_core::row_writing::{edited_row, restructured_row};
use nene_ledger_core::save_plan::SavePlan;

use crate::added_row_shape::AddedRowShape;
use crate::located_record::LocatedRecord;
use crate::record_lookup::RecordLookup;
use crate::save_plan_failure::SavePlanFailure;
use crate::table_changes::TableChanges;

/// 書き換えた行ごとの、列と値の列（`CellEdits::rows` の形）。
type EditedRows<'edits> = Vec<(RowId, Vec<(ColumnId, &'edits CellValue)>)>;

/// 列を足し消しした表で、続いた原本の行をまとめて読む数。
const CHUNK_RECORDS: usize = 4_096;

/// 原本（長さ `source_length`）に差分 `changes` を重ねた内容を書く、保存の計画。
///
/// 行の並びの区間を頭から辿る。原本の区間は、書き換えていない行を原本の区間として写し、書き換えた行は
/// `edited_row` で組み立てる。足した行は方言どおりに書く。並びに無い行（消した行）の書き換えは使わない。
/// 列を足し消ししていれば、原本の行は全部を列の並びどおりに組み立て直す（触っていないセルのバイト列は写す）。
/// 最初のレコードより前のバイト列（BOM）は、最初のレコードを消しても残す。
/// 🔴 並びも書き換えも触っていなければ、原本の全体を写す 1 歩になり、原本は引かない（KEEP-02）。
///
/// # Errors
///
/// [`SavePlanFailure`] のとおり。
pub fn save_plan<L: RecordLookup>(
    changes: TableChanges<'_>,
    lookup: &mut L,
    source_length: u64,
    dialect: Dialect,
) -> Result<SavePlan, SavePlanFailure<L::Failure>> {
    let mut plan = SavePlan::empty();
    write_plan(changes, lookup, (source_length, dialect), &mut plan)?;
    Ok(plan)
}

/// [`save_plan`] と同じ計画を、歩を記憶に持たずに書き先 `sink` へ順に渡す（GitHub Issue #130）。
/// 列を足し消しした巨大な表でも、記憶は塊（4,096 行）の大きさにしか比例しない。
/// 書き先が書けなくなったら（[`PlanSink::stopped`]）、塊の境目でやめて [`SavePlanFailure::Stopped`] を返す。
///
/// # Errors
///
/// [`SavePlanFailure`] のとおり。
pub fn write_plan<L: RecordLookup, S: PlanSink + ?Sized>(
    changes: TableChanges<'_>,
    lookup: &mut L,
    (source_length, dialect): (u64, Dialect),
    sink: &mut S,
) -> Result<(), SavePlanFailure<L::Failure>> {
    let edited = changes.edits().rows();
    let width = changes.shape().columns();
    let restructured = (!changes.columns().is_whole()).then(|| changes.columns().columns(width));
    let mut planner = Planner {
        lookup,
        source_length,
        dialect,
        plan: sink,
        unterminated: false,
        after_cr: false,
        body_start: 0,
        restructured,
    };
    if !changes.order().is_whole() || planner.restructured.is_some() {
        planner.body_start = planner.offset_of(0)?.unwrap_or(source_length);
        planner.plan.copy(0, planner.body_start);
    }
    for run in changes.order().runs() {
        match run {
            OrderRun::Source { first, count } if planner.restructured.is_some() => {
                planner.restructured_run((*first, *count), &edited)?;
            }
            OrderRun::Source { first, count } => planner.source_run((*first, *count), &edited)?,
            OrderRun::Added(rows) => planner.added_rows(rows, changes.edits(), changes.shape())?,
        }
        if planner.plan.stopped() {
            return Err(SavePlanFailure::Stopped);
        }
    }
    Ok(())
}

/// 計画を組み立てる途中の状態。`unterminated` は、書いた最後の行が終わり方の無い原本の最後の行か。
/// `after_cr` は、書いた最後の行が `\r` だけで終わったか（次に `\n` で始まる行を書くとつながる）。
struct Planner<'lookup, L, S: ?Sized> {
    lookup: &'lookup mut L,
    source_length: u64,
    dialect: Dialect,
    plan: &'lookup mut S,
    unterminated: bool,
    after_cr: bool,
    body_start: u64,
    /// 列を足し消ししていれば、表の列の順の同一性（原本の行を全部組み立て直す）。
    restructured: Option<Vec<ColumnId>>,
}

impl<L: RecordLookup, S: PlanSink + ?Sized> Planner<'_, L, S> {
    /// レコード `number` を引く。
    fn located(
        &mut self,
        number: u64,
    ) -> Result<Option<LocatedRecord>, SavePlanFailure<L::Failure>> {
        self.lookup.record(number).map_err(SavePlanFailure::Lookup)
    }

    /// レコード `number` の始まりの位置。ファイルの外なら `None`。
    fn offset_of(&mut self, number: u64) -> Result<Option<u64>, SavePlanFailure<L::Failure>> {
        Ok(self.located(number)?.map(|located| located.offset()))
    }

    /// 原本のレコード `first` から `count` 行（`None` は終わりまで）。書き換えた行は組み立て、ほかは写す。
    fn source_run(
        &mut self,
        (first, count): (u64, Option<u64>),
        edited: &EditedRows<'_>,
    ) -> Result<(), SavePlanFailure<L::Failure>> {
        let start = if first == 0 {
            self.body_start
        } else {
            match self.offset_of(first)? {
                Some(offset) => offset,
                // 並びの終わりの、空の「終わりまで」の区間。
                None => return Ok(()),
            }
        };
        if self.after_cr && !edited.iter().any(|(row, _)| *row == RowId::Source(first)) {
            self.keep_apart(first)?;
        }
        let end_record = count.map(|count| first.saturating_add(count));
        let end = match end_record {
            Some(number) => self.offset_of(number)?.unwrap_or(self.source_length),
            None => self.source_length,
        };
        let mut cursor = start;
        for (row, cells) in edited {
            let RowId::Source(number) = *row else {
                continue;
            };
            if number < first || end_record.is_some_and(|end_record| number >= end_record) {
                continue;
            }
            cursor = self.edited_row(number, cells, cursor)?;
        }
        self.plan.copy(cursor, end.saturating_sub(cursor));
        // 区間の最後の行の終わり方。後ろに続く行（足した行・離れた原本の行）の書き方が変わる。
        let last = match end_record {
            Some(end_record) if end_record > first => self.located(end_record.saturating_sub(1))?,
            Some(_) | None => None,
        };
        let ending = last.map(|located| located.record().terminator());
        self.unterminated = ending == Some(LineEnding::EndOfInput);
        self.after_cr = ending == Some(LineEnding::Cr);
        Ok(())
    }

    /// 列を足し消しした表の、原本のレコード `first` から `count` 行（`None` は終わりまで）。続いた行をまとめて読み、
    /// 1 行ずつ列の並びどおりに組み立て直す。
    fn restructured_run(
        &mut self,
        (first, count): (u64, Option<u64>),
        edited: &EditedRows<'_>,
    ) -> Result<(), SavePlanFailure<L::Failure>> {
        let cells: BTreeMap<RowId, &[(ColumnId, &CellValue)]> = edited
            .iter()
            .map(|(row, cells)| (*row, cells.as_slice()))
            .collect();
        let end = count.map(|count| first.saturating_add(count));
        let mut number = first;
        let mut ending = None;
        loop {
            let left = end.map_or(CHUNK_RECORDS, |end| {
                usize::try_from(end.saturating_sub(number))
                    .unwrap_or(usize::MAX)
                    .min(CHUNK_RECORDS)
            });
            if left == 0 {
                break;
            }
            let records = self
                .lookup
                .records(number, left)
                .map_err(SavePlanFailure::Lookup)?;
            let read = records.len();
            for located in records {
                let row = cells
                    .get(&RowId::Source(number))
                    .copied()
                    .unwrap_or_default();
                self.restructured_record(number, &located, row)?;
                ending = Some(located.record().terminator());
                number = number.saturating_add(1);
            }
            if read < left {
                break;
            }
            if self.plan.stopped() {
                return Err(SavePlanFailure::Stopped);
            }
        }
        self.unterminated = ending == Some(LineEnding::EndOfInput);
        self.after_cr = false;
        Ok(())
    }

    /// 列を足し消しした表の 1 行（原本のレコード `number`・書き換え `cells`）を組み立て直して書く。
    fn restructured_record(
        &mut self,
        number: u64,
        located: &LocatedRecord,
        cells: &[(ColumnId, &CellValue)],
    ) -> Result<(), SavePlanFailure<L::Failure>> {
        if !located.is_whole() {
            return Err(SavePlanFailure::TooLong(number));
        }
        let record = located.record();
        let mut values = BTreeMap::new();
        for &(column, value) in cells {
            let CellValue::Text(text) = value else {
                continue;
            };
            let quoted = match column {
                ColumnId::Source(source) => record
                    .fields()
                    .get(source)
                    .is_some_and(|field| field.quoted()),
                ColumnId::Added(_) => false,
            };
            let cell = written_cell(text, quoted, &self.dialect, record.terminator()).map_err(
                |failure| match failure {
                    CellWriteFailure::NeedsQuoting => SavePlanFailure::Unwritable {
                        row: number,
                        column: match column {
                            ColumnId::Source(source) => source,
                            ColumnId::Added(_) => usize::MAX,
                        },
                    },
                },
            )?;
            values.insert(column, cell);
        }
        let lone = written_cell("", true, &self.dialect, record.terminator()).unwrap_or_default();
        let delimiter = char::from(self.dialect.delimiter().byte());
        let columns = self.restructured.as_deref().unwrap_or_default();
        let pieces = restructured_row(record, columns, &values, (delimiter, &lone));
        for piece in pieces {
            match piece {
                RowPiece::Source(range) => {
                    let start = located.offset().saturating_add(widen(range.start()));
                    self.plan.copy(start, widen(range.len()));
                }
                RowPiece::Text(text) => self.plan.text(&text),
            }
        }
        Ok(())
    }

    /// `\r` で終わった行の直後に、空の行 `first`（`\n` で終わる）を写すと `\r\n` につながって行が消える。
    /// 引用の方言なら、その行を `""`（空のセル 1 つ）と書く（ADR 0024 の決定 4 と同じ穴・同じ直し方）。
    /// `Plain` と `Backslash` の方言では引用できないので、この穴は残る。
    fn keep_apart(&mut self, first: u64) -> Result<(), SavePlanFailure<L::Failure>> {
        let Some(located) = self.located(first)? else {
            return Ok(());
        };
        let record = located.record();
        if record.raw().is_empty() && record.terminator() == LineEnding::Lf {
            let cell = written_cell("", true, &self.dialect, LineEnding::Lf).unwrap_or_default();
            self.plan.text(&cell);
        }
        Ok(())
    }

    /// 書き換えた行 `number` を、`cursor` からの続きとして書く。書いた後の位置を返す。
    fn edited_row(
        &mut self,
        number: u64,
        cells: &[(ColumnId, &CellValue)],
        cursor: u64,
    ) -> Result<u64, SavePlanFailure<L::Failure>> {
        let located = self
            .located(number)?
            .ok_or(SavePlanFailure::Missing(number))?;
        if !located.is_whole() {
            return Err(SavePlanFailure::TooLong(number));
        }
        let pieces = row_pieces(number, located.record(), cells, self.dialect)?;
        self.plan
            .copy(cursor, located.offset().saturating_sub(cursor));
        let mut written = located.offset();
        for piece in pieces {
            match piece {
                RowPiece::Source(range) => {
                    let start = located.offset().saturating_add(widen(range.start()));
                    self.plan.copy(start, widen(range.len()));
                    written = start.saturating_add(widen(range.len()));
                }
                RowPiece::Text(text) => self.plan.text(&text),
            }
        }
        Ok(written.max(
            located
                .offset()
                .saturating_add(record_end(located.record())),
        ))
    }

    /// 足した行 `rows` を方言どおりに書く（ADR 0027 の決定 5）。前が終わり方の無い最後の行なら、先に終わり方を書く。
    fn added_rows(
        &mut self,
        rows: &[u64],
        edits: &CellEdits,
        shape: AddedRowShape,
    ) -> Result<(), SavePlanFailure<L::Failure>> {
        let line_break = line_text(shape.line_ending());
        if self.unterminated {
            self.plan.text(line_break);
            self.unterminated = false;
        }
        let columns: Vec<ColumnId> = self
            .restructured
            .clone()
            .unwrap_or_else(|| (0..shape.columns()).map(ColumnId::Source).collect());
        for &added in rows {
            let line = added_line((added, &columns), edits, shape, self.dialect)?;
            self.plan.text(&line);
            self.plan.text(line_break);
        }
        self.after_cr = !rows.is_empty() && shape.line_ending() == LineEnding::Cr;
        Ok(())
    }
}

/// 足した行 `added` の 1 行（終わり方を除く）。セルは表の列の順 `columns`。書いていないセルは空のセル。
/// 列が 1 つで空なら、空の行にしない。
fn added_line<F>(
    (added, columns): (u64, &[ColumnId]),
    edits: &CellEdits,
    shape: AddedRowShape,
    dialect: Dialect,
) -> Result<String, SavePlanFailure<F>> {
    let delimiter = char::from(dialect.delimiter().byte());
    let mut line = String::new();
    for (column, &id) in columns.iter().enumerate() {
        if column > 0 {
            line.push(delimiter);
        }
        let value = match edits.get(RowId::Added(added), id) {
            Some(CellValue::Text(text)) => text.as_str(),
            Some(CellValue::Absent) | None => "",
        };
        let lone_empty = value.is_empty() && columns.len() == 1;
        let cell =
            written_cell(value, lone_empty, &dialect, shape.line_ending()).map_err(|failure| {
                match failure {
                    CellWriteFailure::NeedsQuoting => SavePlanFailure::UnwritableAdded { column },
                }
            })?;
        line.push_str(&cell);
    }
    Ok(line)
}

/// 終わり方の文字列（足した行に書く）。
fn line_text(ending: LineEnding) -> &'static str {
    match ending {
        LineEnding::CrLf => "\r\n",
        LineEnding::Lf => "\n",
        LineEnding::Cr => "\r",
        LineEnding::EndOfInput => "",
    }
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
    use alloc::vec;
    use nene_ledger_core::column_order::ColumnOrder;
    use nene_ledger_core::records::Records;
    use nene_ledger_core::row_order::RowOrder;
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

    fn shape() -> AddedRowShape {
        AddedRowShape::new(2, LineEnding::CrLf)
    }

    fn plan_of(
        input: &[u8],
        (order, edits, shape): (&RowOrder, &CellEdits, AddedRowShape),
    ) -> Result<SavePlan, SavePlanFailure<()>> {
        plan_with_columns(input, (order, &ColumnOrder::whole()), (edits, shape))
    }

    fn plan_with_columns(
        input: &[u8],
        (order, columns): (&RowOrder, &ColumnOrder),
        (edits, shape): (&CellEdits, AddedRowShape),
    ) -> Result<SavePlan, SavePlanFailure<()>> {
        let mut lookup = InMemory { input, lookups: 0 };
        let length = u64::try_from(input.len()).unwrap_or(u64::MAX);
        let changes = TableChanges::new((order, columns), edits, shape);
        save_plan(changes, &mut lookup, length, Dialect::rfc4180())
    }

    fn saved_with(input: &[u8], changes: (&RowOrder, &CellEdits, AddedRowShape)) -> Vec<u8> {
        apply(input, &plan_of(input, changes).unwrap())
    }

    fn saved(input: &[u8], edits: &CellEdits) -> Vec<u8> {
        saved_with(input, (&RowOrder::whole(), edits, shape()))
    }

    fn text(value: &str) -> CellValue {
        CellValue::Text(String::from(value))
    }

    #[test]
    fn without_edits_the_whole_source_is_one_copy() {
        let input = b"\xEF\xBB\xBFa,\"b\"\r\n0012,1-2\n";
        let mut lookup = InMemory { input, lookups: 0 };
        let order = RowOrder::whole();
        let columns = ColumnOrder::whole();
        let edits = CellEdits::empty();
        let changes = TableChanges::new((&order, &columns), &edits, shape());
        let plan = save_plan(changes, &mut lookup, 22, Dialect::rfc4180()).unwrap();
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
        let plan = |edits: &CellEdits| plan_of(input, (&RowOrder::whole(), edits, shape()));
        let mut absent = CellEdits::empty();
        absent.set(RowId::Source(0), ColumnId::Source(0), CellValue::Absent);
        assert_eq!(plan(&absent), Err(SavePlanFailure::Unsupported));
        let mut missing = CellEdits::empty();
        missing.set(RowId::Source(4), ColumnId::Source(0), text("x"));
        assert_eq!(plan(&missing), Err(SavePlanFailure::Missing(4)));
        // 並びに無い行（足して消した行）の書き換えは使わない。
        let mut added = CellEdits::empty();
        added.set(RowId::Added(0), ColumnId::Source(0), text("x"));
        assert_eq!(saved(input, &added), input);
    }

    #[test]
    fn added_rows_are_written_in_the_dialect_between_untouched_rows() {
        let input = b"a,b\r\n\"c\",d\r\n";
        let mut order = RowOrder::whole();
        let rows = order.insert(1, 2);
        let mut edits = CellEdits::empty();
        edits.set(rows[0], ColumnId::Source(0), text("x,y"));
        edits.set(rows[1], ColumnId::Source(1), text("z"));
        assert_eq!(
            saved_with(input, (&order, &edits, shape())),
            b"a,b\r\n\"x,y\",\r\n,z\r\n\"c\",d\r\n"
        );
    }

    #[test]
    fn removed_rows_are_skipped_and_their_edits_are_not_used() {
        let input = b"\xEF\xBB\xBFh\r\n1\r\n2";
        let mut edits = CellEdits::empty();
        edits.set(RowId::Source(1), ColumnId::Source(0), text("one"));
        let mut order = RowOrder::whole();
        let _gone = order.remove(1, 1);
        assert_eq!(
            saved_with(input, (&order, &edits, shape())),
            b"\xEF\xBB\xBFh\r\n2"
        );
        // 最初の行を消しても BOM は残る。
        let mut head = RowOrder::whole();
        let _first = head.remove(0, 1);
        assert_eq!(
            saved_with(input, (&head, &edits, shape())),
            b"\xEF\xBB\xBFone\r\n2"
        );
    }

    #[test]
    fn an_empty_row_after_a_carriage_return_is_kept_apart_when_rows_between_are_removed() {
        // 1 行目は `\r`、3 行目は空の行（`\n`）。間の 2 行目を消すと `\r\n` につながって 3 行目が消えるので、`""` と書く。
        let input = b"a\rb\r\n\nc";
        let mut order = RowOrder::whole();
        let _gone = order.remove(1, 1);
        let edits = CellEdits::empty();
        assert_eq!(saved_with(input, (&order, &edits, shape())), b"a\r\"\"\nc");
    }

    #[test]
    fn columns_added_and_removed_rebuild_every_row_and_keep_untouched_cells() {
        let input = b"\xEF\xBB\xBFid,name\r\n1,\"Sato\"\r\n2\r\n";
        let order = RowOrder::whole();
        let mut columns = ColumnOrder::whole();
        let added = columns.insert(1, 1);
        let mut edits = CellEdits::empty();
        edits.set(RowId::Source(0), added[0], text("age"));
        edits.set(RowId::Source(1), added[0], text("4,0"));
        let shape = AddedRowShape::new(3, LineEnding::CrLf);
        let plan = plan_with_columns(input, (&order, &columns), (&edits, shape)).unwrap();
        assert_eq!(
            apply(input, &plan),
            b"\xEF\xBB\xBFid,age,name\r\n1,\"4,0\",\"Sato\"\r\n2,\r\n"
        );
        // 最初の列を消すと、`2` だけの行は空の行にせず `""` と書く。
        let mut first_gone = ColumnOrder::whole();
        let _id = first_gone.remove(0, 1);
        let narrow = AddedRowShape::new(1, LineEnding::CrLf);
        let none = CellEdits::empty();
        let removed = plan_with_columns(input, (&order, &first_gone), (&none, narrow)).unwrap();
        assert_eq!(
            apply(input, &removed),
            b"\xEF\xBB\xBFname\r\n\"Sato\"\r\n\"\"\r\n"
        );
    }

    #[test]
    fn added_rows_follow_the_column_order() {
        let input = b"a,b\n";
        let mut order = RowOrder::whole();
        let rows = order.insert(1, 1);
        let mut columns = ColumnOrder::whole();
        let added = columns.insert(0, 1);
        let mut edits = CellEdits::empty();
        edits.set(rows[0], added[0], text("new"));
        edits.set(rows[0], ColumnId::Source(1), text("y"));
        let shape = AddedRowShape::new(3, LineEnding::Lf);
        let plan = plan_with_columns(input, (&order, &columns), (&edits, shape)).unwrap();
        assert_eq!(apply(input, &plan), b",a,b\nnew,,y\n");
    }

    #[test]
    fn rows_added_after_an_unterminated_last_row_start_on_a_new_line() {
        let input = b"a\nb";
        let mut order = RowOrder::whole();
        let rows = order.insert(2, 1);
        let mut edits = CellEdits::empty();
        edits.set(rows[0], ColumnId::Source(0), text("c"));
        let lf = AddedRowShape::new(1, LineEnding::Lf);
        assert_eq!(saved_with(input, (&order, &edits, lf)), b"a\nb\nc\n");
        // 列が 1 つの表の空の行は、空の行にしない。
        let empty = CellEdits::empty();
        assert_eq!(saved_with(input, (&order, &empty, lf)), b"a\nb\n\"\"\n");
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

    /// 足した行の値（2 列・終わり方は LF）。
    fn added_values(value: Option<&str>) -> Vec<Vec<u8>> {
        let first = value.map_or_else(Vec::new, |value| {
            value.replace("\r\n", "\n").replace('\r', "\n").into_bytes()
        });
        vec![first, Vec::new()]
    }

    /// 足した行 `row` の 1 列目に、半分の割合で値を書く。
    fn random_added_value(
        rng: &mut Xorshift,
        row: RowId,
        edits: &mut CellEdits,
    ) -> Option<&'static str> {
        let value = (rng.below(2) == 0).then(|| rng.pick(&VALUES))?;
        edits.set(row, ColumnId::Source(0), text(value));
        Some(value)
    }

    /// 無作為に行を足し消しした並びと、素朴な列。足した行の 1 列目には値を書くことがある。
    fn random_order<'v>(
        rng: &mut Xorshift,
        records: usize,
        edits: &mut CellEdits,
    ) -> (RowOrder, Vec<(RowId, Option<&'v str>)>) {
        let mut order = RowOrder::whole();
        let mut model: Vec<(RowId, Option<&str>)> = (0..records)
            .map(|n| (RowId::Source(u64::try_from(n).unwrap()), None))
            .collect();
        for _ in 0..rng.below(5) {
            let at = rng.below(model.len() + 1);
            let position = u64::try_from(at).unwrap();
            if rng.below(2) == 0 {
                let rows = order.insert(position, u64::try_from(1 + rng.below(2)).unwrap());
                let added: Vec<(RowId, Option<&str>)> = rows
                    .into_iter()
                    .map(|row| (row, random_added_value(rng, row, edits)))
                    .collect();
                model.splice(at..at, added);
            } else {
                let count = rng.below(model.len() - at + 1);
                let _gone = order.remove(position, u64::try_from(count).unwrap());
                model.drain(at..at + count);
            }
        }
        (order, model)
    }

    /// 並びの 1 行 `row` が、原本の行なら値とバイト列のまま、足した行なら書いた値で書かれた。
    /// `\r` の直後に来た空の行だけは `""` と書き直す（行がつながって消えないように）。
    fn assert_kept_row(
        (row, value): (RowId, Option<&str>),
        (before, input): (&[Record], &[u8]),
        (new, output): (&Record, &[u8]),
    ) {
        let RowId::Source(n) = row else {
            assert_eq!(values(output, new), added_values(value), "{output:?}");
            return;
        };
        let old = &before[usize::try_from(n).unwrap()];
        assert_eq!(
            values(output, new),
            values(input, old),
            "{input:?} -> {output:?}"
        );
        let kept = new.raw().slice(output);
        if kept != Some(b"\"\"".as_slice()) || !old.raw().is_empty() {
            assert_eq!(old.raw().slice(input), kept, "{output:?}");
        }
    }

    #[test]
    fn random_row_insertions_and_removals_keep_the_other_rows_byte_for_byte() {
        let mut rng = Xorshift(0x5EED_0027_0000_0001);
        let lf = AddedRowShape::new(2, LineEnding::Lf);
        for _ in 0..2_000_u32 {
            let input = random_table(&mut rng);
            let before: Vec<Record> = Records::new(&input, Dialect::rfc4180()).collect();
            let mut edits = CellEdits::empty();
            let (order, model) = random_order(&mut rng, before.len(), &mut edits);
            let output = saved_with(&input, (&order, &edits, lf));
            let after: Vec<Record> = Records::new(&output, Dialect::rfc4180()).collect();
            assert_eq!(
                after.len(),
                model.len(),
                "{input:?} {order:?} -> {output:?}"
            );
            for (&(row, value), new) in model.iter().zip(&after) {
                assert_kept_row((row, value), (&before, &input), (new, &output));
            }
        }
    }

    /// 列を足し消しした後の 1 行の値（`restructured_row` の決め方）：表の列の順に、原本のセル・書いた値・足した列の空の
    /// セル。原本の行より後ろの何も無い原本の列は書かず、何も残らなければ空のセル 1 つ。
    fn restructured_values(
        old: &[Vec<u8>],
        columns: &[ColumnId],
        written: &BTreeMap<ColumnId, &str>,
    ) -> Vec<Vec<u8>> {
        let present = |column: &ColumnId| match column {
            ColumnId::Source(source) => *source < old.len() || written.contains_key(column),
            ColumnId::Added(_) => true,
        };
        let last = columns.iter().rposition(present);
        let mut row: Vec<Vec<u8>> = columns
            .iter()
            .take(last.map_or(0, |last| last + 1))
            .map(|column| match (written.get(column), column) {
                (Some(value), _) => value.replace("\r\n", "\n").replace('\r', "\n").into_bytes(),
                (None, ColumnId::Source(source)) => old.get(*source).cloned().unwrap_or_default(),
                (None, ColumnId::Added(_)) => Vec::new(),
            })
            .collect();
        if row.is_empty() {
            row.push(Vec::new());
        }
        row
    }

    /// 無作為に列を足し消しした並びと、表の列の数（開いたときは 4 列）。
    fn random_columns(rng: &mut Xorshift) -> (ColumnOrder, usize) {
        let mut columns = ColumnOrder::whole();
        let mut width = 4_usize;
        for _ in 0..=rng.below(3) {
            let at = rng.below(width + 1);
            let adding = rng.below(2) == 0;
            if adding {
                let _added = columns.insert(at, 1);
                width += 1;
            } else if at < width {
                let _gone = columns.remove(at, 1);
                width -= 1;
            }
        }
        (columns, width)
    }

    /// 無作為の書き換え（`records` 行の表の、列 `ids` のどれか）と、行と列ごとの書いた値。
    fn random_column_edits(
        rng: &mut Xorshift,
        records: usize,
        ids: &[ColumnId],
    ) -> (CellEdits, BTreeMap<(usize, ColumnId), &'static str>) {
        let mut edits = CellEdits::empty();
        let mut written = BTreeMap::new();
        if records == 0 || ids.is_empty() {
            return (edits, written);
        }
        for _ in 0..rng.below(4) {
            let (row, value) = (rng.below(records), rng.pick(&VALUES));
            let column = ids[rng.below(ids.len())];
            edits.set(
                RowId::Source(u64::try_from(row).unwrap()),
                column,
                text(value),
            );
            written.insert((row, column), value);
        }
        (edits, written)
    }

    /// 行 `row` の、組み立て直した後に読める値。
    fn rebuilt_values(
        (row, old, input): (usize, &Record, &[u8]),
        ids: &[ColumnId],
        written: &BTreeMap<(usize, ColumnId), &str>,
    ) -> Vec<Vec<u8>> {
        let mine: BTreeMap<ColumnId, &str> = written
            .iter()
            .filter(|((r, _), _)| *r == row)
            .map(|((_, column), value)| (*column, *value))
            .collect();
        let mut want = restructured_values(&values(input, old), ids, &mine);
        // 書いた値の改行は、その行の終わり方で書く（読み直すと終わり方の改行になる）。
        for (at, column) in ids.iter().enumerate() {
            if let (Some(value), Some(slot)) = (mine.get(column), want.get_mut(at)) {
                *slot = expected_value(value, old);
            }
        }
        want
    }

    #[test]
    fn random_column_changes_rebuild_rows_with_the_same_values() {
        let mut rng = Xorshift(0x5EED_0027_C011_0001);
        for _ in 0..1_000_u32 {
            let input = random_table(&mut rng);
            let before: Vec<Record> = Records::new(&input, Dialect::rfc4180()).collect();
            let (columns, width) = random_columns(&mut rng);
            let ids = columns.columns(width);
            let (edits, written) = random_column_edits(&mut rng, before.len(), &ids);
            let shape = AddedRowShape::new(width, LineEnding::Lf);
            let order = RowOrder::whole();
            let plan = plan_with_columns(&input, (&order, &columns), (&edits, shape)).unwrap();
            let output = apply(&input, &plan);
            let after: Vec<Record> = Records::new(&output, Dialect::rfc4180()).collect();
            assert_eq!(after.len(), before.len(), "{input:?} -> {output:?}");
            for (row, (old, new)) in before.iter().zip(&after).enumerate() {
                let want = rebuilt_values((row, old, &input), &ids, &written);
                assert_eq!(
                    values(&output, new),
                    want,
                    "{input:?} {ids:?} -> {output:?}"
                );
            }
        }
    }

    /// 書けなくなった書き先には、計画を作るのを途中でやめる（GitHub Issue #130）。
    #[test]
    fn a_stopped_sink_stops_the_plan() {
        struct Broken;
        impl PlanSink for Broken {
            fn copy(&mut self, _: u64, _: u64) {}
            fn text(&mut self, _: &str) {}
            fn stopped(&self) -> bool {
                true
            }
        }
        let input = b"a,b\nc,d\n";
        let order = RowOrder::whole();
        let mut columns = ColumnOrder::whole();
        let _added = columns.insert(0, 1);
        let edits = CellEdits::empty();
        let changes = TableChanges::new(
            (&order, &columns),
            &edits,
            AddedRowShape::new(3, LineEnding::Lf),
        );
        let mut lookup = InMemory { input, lookups: 0 };
        let stopped = write_plan(
            changes,
            &mut lookup,
            (u64::try_from(input.len()).unwrap(), Dialect::rfc4180()),
            &mut Broken,
        );
        assert_eq!(stopped, Err(SavePlanFailure::Stopped));
    }

    /// 書き換えた行でも、触っていないセルの見た目のバイト列を保つ：BOM・最後の行の改行の無さ・末尾の区切り・
    /// 引用符つきの空（`""`）と無しの空・セルの無い所（短い行）・前後の空白（KEEP-03・04・05・07）。列を足して全部の行を
    /// 組み立て直しても同じ（足した列のほかは原本のバイト列。短い行には、足した列までを空のセルで足す・ADR 0027）。
    #[test]
    fn untouched_cells_keep_bom_quotes_blanks_spaces_and_the_missing_end() {
        let input = b"\xEF\xBB\xBF a ,\"\",,x,\r\nshort\r\n\" q \",last";
        let mut edits = CellEdits::empty();
        edits.set(RowId::Source(0), ColumnId::Source(3), text("Y"));
        assert_eq!(
            saved(input, &edits),
            b"\xEF\xBB\xBF a ,\"\",,Y,\r\nshort\r\n\" q \",last"
        );
        let order = RowOrder::whole();
        let mut columns = ColumnOrder::whole();
        let added = columns.insert(2, 1);
        let mut inserted = CellEdits::empty();
        inserted.set(RowId::Source(2), added[0], text("n"));
        let shape = AddedRowShape::new(6, LineEnding::CrLf);
        let plan = plan_with_columns(input, (&order, &columns), (&inserted, shape)).unwrap();
        assert_eq!(
            apply(input, &plan),
            b"\xEF\xBB\xBF a ,\"\",,,x,\r\nshort,,\r\n\" q \",last,n"
        );
    }
}
