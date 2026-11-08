//! `--find`：ファイルの頭から探し、最初に当たったセルを書く（測定用・ADR 0030 の決定 4・QLT-014）。

use core::ops::ControlFlow;
use std::path::Path;

use nene_ledger_application::cell_row::CellRow;
use nene_ledger_application::find_scope::FindScope;
use nene_ledger_core::cell_edits::CellEdits;
use nene_ledger_core::column_id::ColumnId;
use nene_ledger_core::find_options::FindOptions;
use nene_ledger_core::find_query::FindQuery;
use nene_ledger_core::row_order::RowOrder;
use nene_ledger_scan::indexed_source::IndexedSource;
use nene_ledger_scan::range_folding::fold_range;
use nene_ledger_scan::record_range::RecordRange;
use nene_ledger_scan::source::Source;
use nene_ledger_scan::sparse_index::SparseIndex;

use crate::find_fold::FindFold;
use crate::open_failure::OpenFailure;
use crate::opening::guess_format;
use crate::rows::read_rows;
use nene_ledger_application::find_bound::FindBound;

/// 一度に読むレコードの数（裏の仕事 `RangeJob` と同じ）。
const CHUNK_RECORDS: usize = 4_096;
/// 列の数を見積もるために先頭から読むレコードの数。
const SAMPLE_RECORDS: usize = 1_000;

/// `path` を既定の条件（ADR 0030 の決定 3）で `text` を頭から探し、最初に当たったセル（1 始まりのレコードの番号と列）。
/// 見出しのレコードは探さない。列の数は先頭の 1,000 レコードのいちばん多い列の数（画面と同じく、先頭から見積もる）。
///
/// # Errors
///
/// 開けない・読めない。
pub(crate) fn find_first(path: &Path, text: &str) -> Result<Option<(u64, usize)>, OpenFailure> {
    let source = Source::open(path)?;
    let guessed = guess_format(&source)?;
    let format = guessed.format();
    let sample = read_rows(
        &source,
        &SparseIndex::empty(),
        format,
        RecordRange::new(0, SAMPLE_RECORDS),
    )?;
    let width = sample.cells().iter().map(CellRow::len).max().unwrap_or(0);
    let columns = (0..width).map(ColumnId::Source).collect();
    let scope = FindScope::new(
        FindQuery::new(text, FindOptions::standard()),
        columns,
        CellEdits::empty(),
    );
    let mut fold = FindFold::new(
        scope,
        (format, guessed.header()),
        RowOrder::whole(),
        FindBound::After(None),
    );
    let from = IndexedSource::new(source, SparseIndex::empty(), format);
    let _flow = fold_range(
        &from,
        RecordRange::new(0, usize::MAX).chunks(CHUNK_RECORDS),
        &mut fold,
        |_| ControlFlow::Continue(()),
    )?;
    Ok(fold
        .found()
        .map(|(record, column)| (record.saturating_add(1), column.saturating_add(1))))
}
