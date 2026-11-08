//! 選んだ範囲の集計を、裏の仕事で 1 レコードずつ足す畳み込み。

use nene_ledger_application::row_band::RowBand;
use nene_ledger_application::selection_stats::SelectionStats;
use nene_ledger_core::header_rows::HeaderRows;
use nene_ledger_core::text_format::TextFormat;
use nene_ledger_core::value::value;
use nene_ledger_scan::decoding::decode;
use nene_ledger_scan::record_fold::RecordFold;
use nene_ledger_scan::window_record::WindowRecord;

/// 選んだ範囲の集計を、裏の仕事で 1 レコードずつ足す畳み込み。選んだ範囲の帯の列だけを、その場の集計
/// （`rows::read_rows`）と同じ規則で復号して `SelectionStats` に足す（重なりは 1 回だけ）。見出しのレコードは数えない。
#[derive(Debug)]
pub(crate) struct TotalsFold {
    stats: SelectionStats,
    bands: Vec<RowBand>,
    header: Option<HeaderRows>,
    format: TextFormat,
}

impl TotalsFold {
    /// 帯 `bands`（行はレコードの番号・上から）を数える。見出しのレコード `header` は除く。
    pub(crate) fn new(bands: Vec<RowBand>, header: Option<HeaderRows>, format: TextFormat) -> Self {
        Self {
            stats: SelectionStats::empty(),
            bands,
            header,
            format,
        }
    }

    /// 数えた集計。
    pub(crate) fn stats(&self) -> SelectionStats {
        self.stats
    }
}

impl RecordFold for TotalsFold {
    fn record(&mut self, number: u64, record: &WindowRecord) {
        if self.header.is_some_and(|header| header.contains(number)) {
            return;
        }
        let Some(band) = self.bands.iter().find(|band| band.rows().contains(&number)) else {
            return;
        };
        let dialect = self.format.dialect();
        let fields = record.record().fields();
        for &field in band
            .columns()
            .iter()
            .cloned()
            .flatten()
            .filter_map(|column| fields.get(column))
        {
            let raw = value(record.bytes(), field, &dialect);
            self.stats.add(decode(self.format.encoding(), &raw).text());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opening::guess_format;
    use crate::rows::read_rows;
    use core::ops::{ControlFlow, RangeInclusive};
    use nene_ledger_scan::indexed_source::IndexedSource;
    use nene_ledger_scan::range_folding::fold_range;
    use nene_ledger_scan::record_range::RecordRange;
    use nene_ledger_scan::source::Source;
    use nene_ledger_scan::sparse_index::SparseIndex;
    use std::path::PathBuf;

    /// 見本 `name` を、その場の集計（行を読んで見出しを除く）と、畳み込みとで数える。
    fn both(name: &str, columns: RangeInclusive<usize>) -> (SelectionStats, SelectionStats) {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join(name);
        let source = Source::open(&path).unwrap();
        let format = guess_format(&source).unwrap().format();
        let range = RecordRange::new(0, 100);
        let rows = read_rows(&source, &SparseIndex::empty(), format, range).unwrap();
        let strings: Vec<Vec<String>> = rows
            .cells()
            .iter()
            .map(nene_ledger_application::cell_row::CellRow::to_strings)
            .collect();
        let body: Vec<&[String]> = strings.iter().skip(1).map(Vec::as_slice).collect();
        let at_once = SelectionStats::of(&body, &columns);
        let from = IndexedSource::new(source, SparseIndex::empty(), format);
        let mut fold = TotalsFold::new(
            vec![RowBand::new(0..=u64::MAX, vec![columns.clone()])],
            Some(HeaderRows::single(0)),
            format,
        );
        let flow = fold_range(&from, range.chunks(2), &mut fold, |_| {
            ControlFlow::Continue(())
        });
        assert_eq!(flow.unwrap(), ControlFlow::Continue(()));
        (at_once, fold.stats())
    }

    #[test]
    fn the_background_fold_counts_the_same_as_the_rows_read_at_once() {
        let (at_once, folded) = both("totals.csv", 0..=3);
        assert_eq!(folded, at_once);
        // 品目 5・単価 5・数量 4・備考 4（空の 1 つを除く）。数は 120・80.5・1,200・-15.25・１２・3・10・2・4・-1・0.1
        // （桁区切りと全角の数字は GitHub Issue #41 から数として読む）。
        assert_eq!((folded.cells(), folded.numbers()), (18, 11));
        let (sample_at_once, sample_folded) = both("screenshot.csv", 0..=5);
        assert_eq!(sample_folded, sample_at_once);
    }

    #[test]
    fn every_header_row_is_left_out_of_the_count() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join("two-header-rows.csv");
        let source = Source::open(&path).unwrap();
        let format = guess_format(&source).unwrap().format();
        let from = IndexedSource::new(source, SparseIndex::empty(), format);
        let columns = 0..=1_usize;
        let count = |header| {
            let mut fold = TotalsFold::new(
                vec![RowBand::new(0..=u64::MAX, vec![columns.clone()])],
                header,
                format,
            );
            let flow = fold_range(&from, RecordRange::new(0, 100).chunks(2), &mut fold, |_| {
                ControlFlow::Continue(())
            });
            assert_eq!(flow.unwrap(), ControlFlow::Continue(()));
            let stats = fold.stats();
            (stats.cells(), stats.numbers())
        };
        // 見出しの 2 行（売上・今年 / 前年）を除いた 4 つの数だけ（ADR 0034）。1 行の見出しなら 2 行目が混ざる。
        assert_eq!(count(HeaderRows::new(0, 2)), (4, 4));
        assert_eq!(count(Some(HeaderRows::single(0))), (6, 4));
    }
}
