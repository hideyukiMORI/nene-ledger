//! 範囲のレコードを読み、セルを表示の値にする（開くとき・動いたとき・`--dump` の 1 つの経路）。

use core::ops::{ControlFlow, Range};

use nene_ledger_application::cell_row::CellRow;
use nene_ledger_application::cut_record::CutRecord;
use nene_ledger_application::document::FIRST_HELD_COLUMNS;
use nene_ledger_core::text_format::TextFormat;
use nene_ledger_core::value::value;
use nene_ledger_scan::decoding::decode_into;
use nene_ledger_scan::record_length::RecordLength;
use nene_ledger_scan::record_range::RecordRange;
use nene_ledger_scan::scan_failure::ScanFailure;
use nene_ledger_scan::source::Source;
use nene_ledger_scan::sparse_index::SparseIndex;
use nene_ledger_scan::window_record::WindowRecord;
use nene_ledger_scan::windowing::visit_records;

/// 窓のために一度に読む行の数。画面に入る行より多ければよい。
pub(crate) const WINDOW_RECORDS: usize = 200;

/// 範囲から読んだ行（復号したセルの値）と、そのうち上限で切り詰めて先頭の部分だけを読んだレコード（GitHub Issue #36）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Rows {
    cells: Vec<CellRow>,
    cuts: Vec<CutRecord>,
}

impl Rows {
    /// 行の値。
    pub(crate) fn cells(&self) -> &[CellRow] {
        &self.cells
    }

    /// 切り詰めたレコード（番号の順）。
    pub(crate) fn cuts(&self) -> &[CutRecord] {
        &self.cuts
    }

    /// 行の値を取り出す（後には空の並びが残る）。
    pub(crate) fn take_cells(&mut self) -> Vec<CellRow> {
        core::mem::take(&mut self.cells)
    }

    /// 行の値と切り詰めたレコードに分ける。
    pub(crate) fn into_parts(self) -> (Vec<CellRow>, Vec<CutRecord>) {
        (self.cells, self.cuts)
    }
}

/// `range` の行を、索引の目印から読み、引用符とエスケープを外して復号する（全部の列）。レコードは 1 つずつ受け取って行にし、すぐ
/// 手放す。行はセルごとの `String` でなく 1 本の文字列で持つ（レコードの生のバイト列とセルの位置の並びも、範囲の全部の
/// セルの `String` も一度に持たない・ADR 0033 の決定 2・GitHub Issue #22）。
pub(crate) fn read_rows(
    source: &Source,
    index: &SparseIndex,
    format: TextFormat,
    range: RecordRange,
) -> Result<Rows, ScanFailure> {
    read_columns(source, (index, format), (range, None))
}

/// 先頭からの窓（`WINDOW_RECORDS` 行・左から `FIRST_HELD_COLUMNS` 列）を読む。見出しのレコード `header` は全部の列で読む
/// （見出しの名前は、まだ幅を決めていない列の仮の幅と列の操作に全部の列が要る・ADR 0033 の決定 1・3）。開くときと、読み方を
/// 選び直して試すときの 1 つの経路。
pub(crate) fn read_first_window(
    source: &Source,
    format: TextFormat,
    header: Option<u64>,
) -> Result<Rows, ScanFailure> {
    let index = SparseIndex::empty();
    let first = 0..FIRST_HELD_COLUMNS;
    let mut rows = read_columns(
        source,
        (&index, format),
        (
            RecordRange::new(0, WINDOW_RECORDS),
            Some(core::slice::from_ref(&first)),
        ),
    )?;
    let at = header.and_then(|record| usize::try_from(record).ok());
    if let Some((record, at)) = header.zip(at)
        && rows.cells.get(at).is_some_and(|row| !row.is_whole())
    {
        let whole = read_rows(source, &index, format, RecordRange::new(record, 1))?;
        if let (Some(slot), Some(row)) = (rows.cells.get_mut(at), whole.cells.into_iter().next()) {
            *slot = row;
        }
    }
    Ok(rows)
}

/// [`read_rows`] と同じ行の、列の範囲 `columns`（左から・重ならない元の列）のセルだけを復号する（ほかの列は数えるだけ・
/// ADR 0033 の決定 1・2）。`None` なら全部の列。行は列の数と持っている列の範囲を覚える（`CellRow::holding`）。
pub(crate) fn read_columns(
    source: &Source,
    (index, format): (&SparseIndex, TextFormat),
    (range, columns): (RecordRange, Option<&[Range<usize>]>),
) -> Result<Rows, ScanFailure> {
    let mut cells = Vec::new();
    let mut cuts = Vec::new();
    let mut number = range.first();
    visit_records(source, (index, format), range, |record: WindowRecord| {
        let mut row = decoded_row(&record, format, columns);
        row.shrink_to_fit();
        cells.push(row);
        if record.is_truncated() {
            let total = match record.length() {
                RecordLength::Exact(length) => Some(length),
                RecordLength::MoreThan(_) => None,
            };
            cuts.push(CutRecord::new(number, total));
        }
        number = number.saturating_add(1);
        ControlFlow::Continue(())
    })?;
    Ok(Rows { cells, cuts })
}

/// レコード `record` の、列の範囲 `columns` のセル（`None` なら全部）を復号した行。セルの字は行の文字列に直接書き足す。
fn decoded_row(
    record: &WindowRecord,
    format: TextFormat,
    columns: Option<&[Range<usize>]>,
) -> CellRow {
    let dialect = format.dialect();
    let fields = record.record().fields();
    let push = |row: &mut CellRow, field| {
        let raw = value(record.bytes(), field, &dialect);
        row.push_with(|text| decode_into(format.encoding(), &raw, text));
    };
    let Some(spans) = columns else {
        let mut row = CellRow::with_capacity(fields.len(), record.bytes().len());
        for &field in fields {
            push(&mut row, field);
        }
        return row;
    };
    let mut row = CellRow::empty();
    let held = spans
        .iter()
        .filter_map(|span| fields.get(span.start..span.end.min(fields.len())))
        .flatten();
    for &field in held {
        push(&mut row, field);
    }
    row.holding(fields.len(), spans)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// `name` に `contents` を書いた一時ファイル（`target/tmp`）。
    fn written(name: &str, contents: &[u8]) -> PathBuf {
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/tmp");
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join(format!("rows-{name}"));
        std::fs::write(&path, contents).unwrap();
        path
    }

    /// 上限を超えて先頭だけを読んだレコードを、番号と（索引が終わりを知っていれば）全体の大きさで返す（GitHub Issue #36）。
    #[test]
    fn records_cut_at_the_limit_are_listed_with_their_whole_size() {
        let mut contents = b"id,memo\n1,a\n2,\"".to_vec();
        contents.extend(core::iter::repeat_n(b'z', 2 * 1_048_576));
        contents.extend_from_slice(b"\"\n3,c\n");
        let path = written("cut.csv", &contents);
        let source = Source::open(&path).unwrap();
        let format = crate::opening::guess_format(&source).unwrap().format();
        let without_index = read_rows(
            &source,
            &SparseIndex::empty(),
            format,
            RecordRange::new(0, 10),
        );
        let cuts = without_index.unwrap().cuts().to_vec();
        assert_eq!(cuts, [CutRecord::new(2, None)]);
        let outcome = nene_ledger_scan::indexing::build_index(
            &source,
            format,
            nene_ledger_scan::block_size::BlockSize::MEBIBYTE,
            |_| core::ops::ControlFlow::Continue(()),
        )
        .unwrap();
        let nene_ledger_scan::index_outcome::IndexOutcome::Finished(index) = outcome else {
            panic!("中止を求めていない");
        };
        let rows = read_rows(&source, &index, format, RecordRange::new(0, 10)).unwrap();
        assert_eq!(rows.cells().len(), 4);
        assert_eq!(rows.cuts(), [CutRecord::new(2, Some(2 * 1_048_576 + 5))]);
    }
}
