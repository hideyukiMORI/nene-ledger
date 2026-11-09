//! ファイル全体の、列の数が見出しと違う行を数える畳み込み（GitHub Issue #391）。

use nene_ledger_core::header_rows::HeaderRows;
use nene_ledger_scan::record_fold::RecordFold;
use nene_ledger_scan::window_record::WindowRecord;

/// ファイル全体の、列の数が見出しと違う行を数える畳み込み。見出しとそれより前（前書き）のレコードは数えない。
/// 数え方は F8（`FindScope::shape`）と範囲の集計（`SelectionStats::add_shape`）と同じ：レコードのセルの数が見出しの
/// セルの数と違えば 1 つ。
#[derive(Debug, Clone)]
pub(crate) struct RaggedFold {
    header: HeaderRows,
    expected: usize,
    rows: u64,
    /// 見つけた行のレコードの番号（昇順・多くて `POSITIONS` 個）。F8 で今いる行が何件目かを言う（GitHub Issue #415）。
    positions: Vec<u64>,
}

/// 覚えておく壊れた行の位置の数（8 バイト × 10 万 = 800 KB まで）。越えたら「何件目」は言わない。
const POSITIONS: usize = 100_000;

impl RaggedFold {
    /// 見出しのレコード `header`（セルの数 `expected`）の後ろを数える。
    pub(crate) const fn new(header: HeaderRows, expected: usize) -> Self {
        Self {
            header,
            expected,
            rows: 0,
            positions: Vec::new(),
        }
    }

    /// 数えた行の数。
    pub(crate) const fn rows(&self) -> u64 {
        self.rows
    }

    /// 見つけた行のレコードの番号（昇順）。数えた行の数より少なければ、覚えきれなかった。
    pub(crate) fn into_positions(self) -> Vec<u64> {
        self.positions
    }
}

impl RecordFold for RaggedFold {
    fn record(&mut self, number: u64, record: &WindowRecord) {
        if number >= self.header.end() && record.record().fields().len() != self.expected {
            self.rows = self.rows.saturating_add(1);
            if self.positions.len() < POSITIONS {
                self.positions.push(number);
            }
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
                assert_eq!(
                    u64::try_from(fold.clone().into_positions().len()).unwrap(),
                    fold.rows()
                );
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
