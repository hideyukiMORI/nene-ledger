//! ファイル全体の、列の数が見出しと違う行を数える畳み込み（GitHub Issue #391）。

use nene_ledger_core::header_rows::HeaderRows;
use nene_ledger_scan::record_fold::RecordFold;
use nene_ledger_scan::window_record::WindowRecord;

/// ファイル全体の、列の数が見出しと違う行を数える畳み込み。見出しとそれより前（前書き）のレコードは数えない。
/// 数え方は F8（`FindScope::shape`）と範囲の集計（`SelectionStats::add_shape`）と同じ：レコードのセルの数が見出しの
/// セルの数と違えば 1 つ。
#[derive(Debug, Clone, Copy)]
pub(crate) struct RaggedFold {
    header: HeaderRows,
    expected: usize,
    rows: u64,
}

impl RaggedFold {
    /// 見出しのレコード `header`（セルの数 `expected`）の後ろを数える。
    pub(crate) const fn new(header: HeaderRows, expected: usize) -> Self {
        Self {
            header,
            expected,
            rows: 0,
        }
    }

    /// 数えた行の数。
    pub(crate) const fn rows(self) -> u64 {
        self.rows
    }
}

impl RecordFold for RaggedFold {
    fn record(&mut self, number: u64, record: &WindowRecord) {
        if number >= self.header.end() && record.record().fields().len() != self.expected {
            self.rows = self.rows.saturating_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opening::guess_format;
    use core::ops::ControlFlow;
    use nene_ledger_scan::indexed_source::IndexedSource;
    use nene_ledger_scan::range_folding::fold_range;
    use nene_ledger_scan::record_range::RecordRange;
    use nene_ledger_scan::source::Source;
    use nene_ledger_scan::sparse_index::SparseIndex;
    use std::path::PathBuf;

    fn counted(name: &str, header: HeaderRows, expected: usize, records: usize) -> Vec<u64> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join(name);
        let source = Source::open(&path).unwrap();
        let format = guess_format(&source).unwrap().format();
        let from = IndexedSource::new(source, SparseIndex::empty(), format);
        [1, 2, records]
            .into_iter()
            .map(|chunk| {
                let mut fold = RaggedFold::new(header, expected);
                let flow = fold_range(
                    &from,
                    RecordRange::new(0, records).chunks(chunk),
                    &mut fold,
                    |_| ControlFlow::Continue(()),
                );
                assert_eq!(flow.unwrap(), ControlFlow::Continue(()));
                fold.rows()
            })
            .collect()
    }

    /// 見本 `ragged.csv`（見出し 3 列・2 行目が 2 列・4 行目が 4 列）は 2 行。塊の大きさに依らない。
    #[test]
    fn rows_whose_cells_differ_from_the_header_are_counted() {
        assert_eq!(
            counted("ragged.csv", HeaderRows::single(0), 3, 5),
            [2, 2, 2]
        );
    }

    /// 見出しより前の前書き（列の数が違う）と見出しは数えない（見本 `preamble.csv`：見出しは 3 行目・4 列）。
    #[test]
    fn the_preamble_and_the_header_are_not_counted() {
        assert_eq!(
            counted("preamble.csv", HeaderRows::single(2), 4, 5),
            [0, 0, 0]
        );
    }
}
