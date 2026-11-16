//! 原本 ＋ 差分の表を、別の方言で全部の行を組み立て直して書く（方言を変えた書き出し・ADR 0036 の決定 4）。
//!
//! 保存（[`crate::save_planning`]）は触っていないセルを原本のバイト列のまま写すが、書き出しは全部のセルを値から
//! 書き直す。行の 1 行の書き方の正本は core の [`exported_row`]。

use alloc::string::String;
use alloc::vec::Vec;

use nene_ledger_core::cell_edits::CellEdits;
use nene_ledger_core::cell_value::CellValue;
use nene_ledger_core::column_id::ColumnId;
use nene_ledger_core::dialect::Dialect;
use nene_ledger_core::line_ending::LineEnding;
use nene_ledger_core::order_run::OrderRun;
use nene_ledger_core::plan_sink::PlanSink;
use nene_ledger_core::row_export::exported_row;
use nene_ledger_core::row_id::RowId;
use nene_ledger_core::value::value;

use crate::export_findings::ExportFindings;
use crate::export_shape::ExportShape;
use crate::located_record::LocatedRecord;
use crate::record_lookup::RecordLookup;
use crate::save_plan_failure::SavePlanFailure;
use crate::table_changes::TableChanges;

/// 続いた原本の行をまとめて読む数。
const CHUNK_RECORDS: usize = 4_096;

/// 書き出す 1 行の様子。`blank` は、触っていない原本の空の行か。`ended` は、行の終わり方があるか（原本の最後の行に
/// 終わり方が無ければ `false`）。
type RowTraits = (bool, bool);

/// 原本に差分 `changes` を重ねた表の全部の行を、書き出しの形 `shape` で書き先 `sink` へ順に渡す。
///
/// 原本のセルは、原本の方言 `source` で引用符とエスケープを外し、`decode` で復号した値を書く。BOM は書かない
/// （呼ぶ側が先に書く）。行の終わり方は `shape` の 1 つにそろえる。原本の最後の行に終わり方が無ければ、書き出しの
/// 最後の行にも付けない。「セルが無い」所は書かない（短い行は短いまま・KEEP-05）。記憶は塊（4,096 行）の大きさに
/// しか比例しない。
///
/// # Errors
///
/// [`SavePlanFailure`] のとおり（書き出す方言で書けない値は `Unsupported`）。
pub fn write_export<L, D, S>(
    changes: TableChanges<'_>,
    (lookup, source, decode): (&mut L, Dialect, D),
    shape: ExportShape,
    sink: &mut S,
) -> Result<(), SavePlanFailure<L::Failure>>
where
    L: RecordLookup,
    D: Fn(&[u8]) -> String,
    S: PlanSink + ?Sized,
{
    let mut pending = false;
    each_row(
        changes,
        (lookup, source, decode),
        |_position, cells: &[String], (blank, ended): RowTraits| {
            if sink.stopped() {
                return Err(SavePlanFailure::Stopped);
            }
            if pending {
                sink.text(shape.line_break());
            }
            pending = ended;
            let written = written_row(cells, blank, shape).ok_or(SavePlanFailure::Unsupported)?;
            sink.text(&written);
            Ok(())
        },
    )?;
    if pending {
        sink.text(shape.line_break());
    }
    Ok(())
}

/// [`write_export`] が書く表のセルのうち、`unwritable` が真を返す値のセルを数える（書き始める前の検査・SAVE-04）。
/// `stopped` が真を返したら、行の境目でやめる（[`SavePlanFailure::Stopped`]）。
///
/// # Errors
///
/// [`SavePlanFailure`] のとおり。
pub fn count_cells<L, D, U, P>(
    changes: TableChanges<'_>,
    (lookup, source, decode): (&mut L, Dialect, D),
    (unwritable, stopped): (U, P),
) -> Result<ExportFindings, SavePlanFailure<L::Failure>>
where
    L: RecordLookup,
    D: Fn(&[u8]) -> String,
    U: Fn(&str) -> bool,
    P: Fn() -> bool,
{
    let mut findings = ExportFindings::none();
    each_row(
        changes,
        (lookup, source, decode),
        |position, cells: &[String], _traits: RowTraits| {
            if stopped() {
                return Err(SavePlanFailure::Stopped);
            }
            for (column, cell) in cells.iter().enumerate() {
                if unwritable(cell) {
                    findings.add(position, column);
                }
            }
            Ok(())
        },
    )?;
    Ok(findings)
}

/// 1 行を書いた文字列（終わり方を除く）。空の行になるとき（空のセルが 1 つだけ・セルが 1 つも残らない）は、触って
/// いない原本の空の行でなければ `""` と書く（行が消えたように読まれない・ADR 0024 の決定 4 と同じ）。
fn written_row(cells: &[String], blank: bool, shape: ExportShape) -> Option<String> {
    let lone = cells.len() <= 1 && cells.iter().all(String::is_empty);
    if lone && blank {
        return Some(String::new());
    }
    if lone {
        let quote = char::from(shape.dialect().quote());
        return Some(String::from_iter([quote, quote]));
    }
    exported_row(
        cells,
        (shape.dialect(), shape.policy()),
        shape.line_ending(),
    )
    .ok()
}

/// 原本に差分 `changes` を重ねた表の行を、頭から順に `take` へ渡す。
fn each_row<L, D, T>(
    changes: TableChanges<'_>,
    (lookup, source, decode): (&mut L, Dialect, D),
    take: T,
) -> Result<(), SavePlanFailure<L::Failure>>
where
    L: RecordLookup,
    D: Fn(&[u8]) -> String,
    T: FnMut(u64, &[String], RowTraits) -> Result<(), SavePlanFailure<L::Failure>>,
{
    let width = changes.shape().columns();
    let restructured = (!changes.columns().is_whole()).then(|| changes.columns().columns(width));
    let mut exporter = Exporter {
        lookup,
        source,
        decode,
        take,
        edits: changes.edits(),
        restructured,
        position: 0,
    };
    for run in changes.order().runs() {
        match run {
            OrderRun::Source { first, count } => exporter.source_run(*first, *count)?,
            OrderRun::Added(rows) => exporter.added_rows(rows, width)?,
        }
    }
    Ok(())
}

/// 行を辿る途中の状態。`position` は、次に渡す行の位置。
struct Exporter<'export, L, D, T> {
    lookup: &'export mut L,
    source: Dialect,
    decode: D,
    take: T,
    edits: &'export CellEdits,
    restructured: Option<Vec<ColumnId>>,
    position: u64,
}

impl<L, D, T> Exporter<'_, L, D, T>
where
    L: RecordLookup,
    D: Fn(&[u8]) -> String,
    T: FnMut(u64, &[String], RowTraits) -> Result<(), SavePlanFailure<L::Failure>>,
{
    /// 原本のレコード `first` から `count` 行（`None` は終わりまで）。
    fn source_run(
        &mut self,
        first: u64,
        count: Option<u64>,
    ) -> Result<(), SavePlanFailure<L::Failure>> {
        let end = count.map(|count| first.saturating_add(count));
        let mut number = first;
        loop {
            let left = end.map_or(CHUNK_RECORDS, |end| {
                usize::try_from(end.saturating_sub(number))
                    .unwrap_or(usize::MAX)
                    .min(CHUNK_RECORDS)
            });
            if left == 0 {
                return Ok(());
            }
            let records = self
                .lookup
                .records(number, left)
                .map_err(SavePlanFailure::Lookup)?;
            let read = records.len();
            for located in records {
                self.source_record(number, &located)?;
                number = number.saturating_add(1);
            }
            if read < left {
                return Ok(());
            }
        }
    }

    /// 原本のレコード `number` の 1 行。
    fn source_record(
        &mut self,
        number: u64,
        located: &LocatedRecord,
    ) -> Result<(), SavePlanFailure<L::Failure>> {
        if !located.is_whole() {
            return Err(SavePlanFailure::TooLong(number));
        }
        let row = RowId::Source(number);
        let touched = self.edits.touches_row(row);
        let fields = located.record().fields();
        let columns = match &self.restructured {
            Some(columns) => columns.clone(),
            None => (0..self.edits.edited_width(row).max(fields.len()))
                .map(ColumnId::Source)
                .collect(),
        };
        let mut cells: Vec<Option<String>> = Vec::with_capacity(columns.len());
        for column in columns {
            let edit = if touched {
                self.edits.get(row, column)
            } else {
                None
            };
            cells.push(match (edit, column) {
                (Some(CellValue::Text(text)), _) => Some(text.clone()),
                (Some(CellValue::Absent), _) => None,
                (None, ColumnId::Source(index)) => fields
                    .get(index)
                    .map(|&field| (self.decode)(&value(located.bytes(), field, &self.source))),
                (None, ColumnId::Added(_)) => Some(String::new()),
            });
        }
        // 原本の空の行（触っていない）は、空の行のまま書く。
        let blank = !touched && located.record().raw().is_empty();
        let ended = located.record().terminator() != LineEnding::EndOfInput;
        self.row(cells, (blank, ended))
    }

    /// 足した行 `rows`（表の列の数 `width`）。
    fn added_rows(
        &mut self,
        rows: &[u64],
        width: usize,
    ) -> Result<(), SavePlanFailure<L::Failure>> {
        let columns: Vec<ColumnId> = self
            .restructured
            .clone()
            .unwrap_or_else(|| (0..width).map(ColumnId::Source).collect());
        for &added in rows {
            let cells = columns
                .iter()
                .map(
                    |&column| match self.edits.get(RowId::Added(added), column) {
                        Some(CellValue::Text(text)) => Some(text.clone()),
                        Some(CellValue::Absent) | None => Some(String::new()),
                    },
                )
                .collect();
            self.row(cells, (false, true))?;
        }
        Ok(())
    }

    /// 1 行を渡す。後ろに続く「セルが無い」所は除き、間の所は空のセルにする。
    fn row(
        &mut self,
        mut cells: Vec<Option<String>>,
        traits: RowTraits,
    ) -> Result<(), SavePlanFailure<L::Failure>> {
        while cells.last().is_some_and(Option::is_none) {
            cells.pop();
        }
        let cells: Vec<String> = cells.into_iter().map(Option::unwrap_or_default).collect();
        let position = self.position;
        self.position = position.saturating_add(1);
        (self.take)(position, &cells, traits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nene_ledger_core::column_order::ColumnOrder;
    use nene_ledger_core::delimiter::Delimiter;
    use nene_ledger_core::quote_policy::QuotePolicy;
    use nene_ledger_core::quoting::Quoting;
    use nene_ledger_core::records::Records;
    use nene_ledger_core::row_order::RowOrder;

    use crate::added_row_shape::AddedRowShape;

    /// 入力を記憶に持ち、参照実装で切って引く。
    struct InMemory<'a> {
        input: &'a [u8],
        dialect: Dialect,
    }

    impl RecordLookup for InMemory<'_> {
        type Failure = ();

        fn record(&mut self, number: u64) -> Result<Option<LocatedRecord>, ()> {
            let nth = usize::try_from(number).unwrap_or(usize::MAX);
            let Some(record) = Records::new(self.input, self.dialect).nth(nth) else {
                return Ok(None);
            };
            let bytes = self.input[record.raw().start()..].to_vec();
            let Some(again) = Records::new(&bytes, self.dialect).next() else {
                return Ok(None);
            };
            let offset = u64::try_from(record.raw().start()).unwrap_or(u64::MAX);
            Ok(Some(LocatedRecord::new(offset, bytes, again, true)))
        }
    }

    /// 書いた文字列を集める書き先。
    struct Collected(String);

    impl PlanSink for Collected {
        fn copy(&mut self, _offset: u64, _length: u64) {
            panic!("書き出しは原本を写さない");
        }

        fn text(&mut self, text: &str) {
            self.0.push_str(text);
        }
    }

    fn dialect(delimiter: u8) -> Dialect {
        Dialect::new(Delimiter::new(delimiter).unwrap(), Quoting::Doubled, b'"').unwrap()
    }

    fn shape(delimiter: u8, policy: QuotePolicy, ending: LineEnding) -> ExportShape {
        ExportShape::new(dialect(delimiter), policy, ending)
    }

    fn decode(bytes: &[u8]) -> String {
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    fn exported(
        input: &str,
        (order, columns, edits): (&RowOrder, &ColumnOrder, &CellEdits),
        shape: ExportShape,
    ) -> String {
        let source = dialect(b',');
        let mut lookup = InMemory {
            input: input.as_bytes(),
            dialect: source,
        };
        let changes = TableChanges::new(
            (order, columns),
            edits,
            AddedRowShape::new(3, LineEnding::CrLf),
        );
        let mut out = Collected(String::new());
        write_export(changes, (&mut lookup, source, decode), shape, &mut out).unwrap();
        out.0
    }

    fn untouched(input: &str, shape: ExportShape) -> String {
        exported(
            input,
            (
                &RowOrder::whole(),
                &ColumnOrder::whole(),
                &CellEdits::empty(),
            ),
            shape,
        )
    }

    #[test]
    fn every_row_is_rewritten_in_the_chosen_dialect() {
        let input = "id,name,memo\r\n0012,\"a,b\",x\n3,\"say \"\"hi\"\"\",\"plain\"\r\n";
        assert_eq!(
            untouched(input, shape(b'\t', QuotePolicy::AsNeeded, LineEnding::Lf)),
            "id\tname\tmemo\n0012\ta,b\tx\n3\t\"say \"\"hi\"\"\"\tplain\n"
        );
        assert_eq!(
            untouched(input, shape(b',', QuotePolicy::All, LineEnding::CrLf)),
            "\"id\",\"name\",\"memo\"\r\n\"0012\",\"a,b\",\"x\"\r\n\"3\",\"say \"\"hi\"\"\",\"plain\"\r\n"
        );
    }

    #[test]
    fn short_rows_blank_rows_and_a_missing_last_break_stay_as_they_are() {
        let input = "a,b,c\n1\n\n\"\"\n4,,\n5,6,7";
        assert_eq!(
            untouched(input, shape(b';', QuotePolicy::AsNeeded, LineEnding::CrLf)),
            "a;b;c\r\n1\r\n\r\n\"\"\r\n4;;\r\n5;6;7"
        );
        // 終わり方のある最後の行は、選んだ終わり方で終わる。
        assert_eq!(
            untouched("a\r\n", shape(b',', QuotePolicy::AsNeeded, LineEnding::Lf)),
            "a\n"
        );
    }

    #[test]
    fn edits_row_order_and_column_order_are_all_applied() {
        let input = "id,name\n1,x\n2,y\n3,z\n";
        let mut edits = CellEdits::empty();
        edits.set(
            RowId::Source(1),
            ColumnId::Source(1),
            CellValue::Text(String::from("X,1")),
        );
        // 原本の行に無いセルへ書く（間は空のセル）。
        edits.set(
            RowId::Source(2),
            ColumnId::Source(3),
            CellValue::Text(String::from("far")),
        );
        // 後ろのセルを「セルが無い」にする。
        edits.set(RowId::Source(3), ColumnId::Source(1), CellValue::Absent);
        let mut order = RowOrder::whole();
        let removed = order.remove(0, 1);
        assert_eq!(removed.len(), 1);
        let added = order.insert(1, 1);
        edits.set(
            added[0],
            ColumnId::Source(0),
            CellValue::Text(String::from("new")),
        );
        let shape = shape(b',', QuotePolicy::AsNeeded, LineEnding::Lf);
        assert_eq!(
            exported(input, (&order, &ColumnOrder::whole(), &edits), shape),
            "1,\"X,1\"\nnew,,\n2,y,,far\n3\n"
        );
        let mut columns = ColumnOrder::whole();
        let extra = columns.insert(0, 1);
        columns.remove(2, 1);
        edits.set(
            RowId::Source(1),
            extra[0],
            CellValue::Text(String::from("head")),
        );
        assert_eq!(
            exported(input, (&order, &columns, &edits), shape),
            "head,1\n,new,\n,2\n,3\n"
        );
    }

    /// 書けないセルを、書き出す表の所で数える（消した行の値は数えない・書き換えた値を見る）。
    #[test]
    fn unwritable_cells_are_counted_where_they_will_be_written() {
        let input = "id,name\n1,😀\n2,y\n3,z😀\n";
        let mut edits = CellEdits::empty();
        edits.set(
            RowId::Source(2),
            ColumnId::Source(0),
            CellValue::Text(String::from("😀2")),
        );
        let mut order = RowOrder::whole();
        order.remove(1, 1);
        let source = dialect(b',');
        let mut lookup = InMemory {
            input: input.as_bytes(),
            dialect: source,
        };
        let columns = ColumnOrder::whole();
        let changes = TableChanges::new(
            (&order, &columns),
            &edits,
            AddedRowShape::new(2, LineEnding::Lf),
        );
        let emoji = |text: &str| text.contains('😀');
        let found = count_cells(changes, (&mut lookup, source, decode), (emoji, || false));
        let found = found.unwrap();
        assert_eq!((found.count(), found.first()), (2, Some((1, 0))));
        let stopped = count_cells(changes, (&mut lookup, source, decode), (emoji, || true));
        assert_eq!(stopped, Err(SavePlanFailure::Stopped));
    }

    /// 種を固定した xorshift（テストは乱数の名前に頼らない・ARC-007）。
    struct Xorshift(u64);

    impl Xorshift {
        fn below(&mut self, bound: usize) -> usize {
            self.0 ^= self.0 << 13_u32;
            self.0 ^= self.0 >> 7_u32;
            self.0 ^= self.0 << 17_u32;
            usize::try_from(self.0 % u64::try_from(bound).unwrap()).unwrap()
        }
    }

    /// 入力 `input` を方言 `dialect` で読んだ表（行ごとのセルの値）。
    fn table(input: &str, dialect: Dialect) -> Vec<Vec<String>> {
        Records::new(input.as_bytes(), dialect)
            .map(|record| {
                record
                    .fields()
                    .iter()
                    .map(|&field| decode(&value(input.as_bytes(), field, &dialect)))
                    .collect()
            })
            .collect()
    }

    /// 無作為のセル（引用したセルには区切り・引用符・改行も混ぜる）。
    fn random_cell(rng: &mut Xorshift) -> String {
        const PARTS: [&str; 10] = ["a", "007", ",", "\t", "\"\"", ";", " ", "東京", "\n", "|"];
        let quoted = rng.below(2) == 0;
        let text: String = (0..rng.below(4))
            .map(|_| PARTS[rng.below(PARTS.len())])
            .filter(|part| quoted || !matches!(*part, "," | "\"\"" | "\n"))
            .collect();
        if quoted {
            alloc::format!("\"{text}\"")
        } else {
            text
        }
    }

    /// 無作為の表（カンマ区切り・行ごとに違う終わり方・短い行と空の行を含む）。
    fn random_input(rng: &mut Xorshift) -> String {
        let mut input = String::new();
        for _ in 0..rng.below(8) {
            let cells: Vec<String> = (0..rng.below(5)).map(|_| random_cell(rng)).collect();
            input.push_str(&cells.join(","));
            input.push_str(["\n", "\r\n"][rng.below(2)]);
        }
        if rng.below(2) == 0 {
            input.push_str("last,\"x\"");
        }
        input
    }

    /// 書き出したファイルを同じ方言で読み戻すと、セルの値が元の表と同じ（乱数の表 × 方言 × 引用符の付け方 × 改行・
    /// ADR 0036 の強制・QLT-008）。セルの中の改行は、選んだ終わり方にそろう。
    #[test]
    fn exported_tables_read_back_as_the_same_cells() {
        let mut rng = Xorshift(0x9E37_79B9_7F4A_7C15);
        for round in 0..2_000_usize {
            let input = random_input(&mut rng);
            let delimiter = b",\t;|"[round % 4];
            let policy = [QuotePolicy::AsNeeded, QuotePolicy::All][rng.below(2)];
            let (ending, line_break) =
                [(LineEnding::Lf, "\n"), (LineEnding::CrLf, "\r\n")][rng.below(2)];
            let out = untouched(&input, shape(delimiter, policy, ending));
            let wanted: Vec<Vec<String>> = table(&input, dialect(b','))
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|cell| cell.replace('\n', line_break))
                        .collect()
                })
                .collect();
            assert_eq!(
                table(&out, dialect(delimiter)),
                wanted,
                "{input:?} → {out:?}"
            );
            assert!(
                !out.replace("\r\n", "")
                    .contains(if ending == LineEnding::CrLf {
                        '\n'
                    } else {
                        '\r'
                    })
            );
        }
    }
}
