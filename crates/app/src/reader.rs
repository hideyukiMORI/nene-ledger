//! 動いた先の範囲を読む。

use std::sync::Arc;

use nene_ledger_application::wakeup::Wakeup;
use nene_ledger_core::text_format::TextFormat;
use nene_ledger_scan::block_size::BlockSize;
use nene_ledger_scan::index_job::IndexJob;
use nene_ledger_scan::indexed_source::IndexedSource;
use nene_ledger_scan::range_job::RangeJob;
use nene_ledger_scan::record_fold::RecordFold;
use nene_ledger_scan::record_range::RecordRange;
use nene_ledger_scan::scan_failure::ScanFailure;
use nene_ledger_scan::source::Source;
use nene_ledger_scan::sparse_index::SparseIndex;

use crate::rows::{WINDOW_RECORDS, read_rows};

/// 動いた先の範囲を読む材料。画面のスレッドが持つ原本と読み方と、作り終えた索引（まだなら作り始める前の索引）と、
/// 裏の仕事が画面のスレッドを起こす口。
pub(crate) struct Reader {
    source: Source,
    format: TextFormat,
    index: SparseIndex,
    wakeup: Arc<dyn Wakeup>,
}

impl core::fmt::Debug for Reader {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("Reader")
            .field("source", &self.source)
            .field("format", &self.format)
            .field("index", &self.index)
            .finish_non_exhaustive()
    }
}

impl Reader {
    /// 作り始める前の索引（先頭から切り進める）で始める。
    pub(crate) fn new(source: Source, format: TextFormat, wakeup: Arc<dyn Wakeup>) -> Self {
        Self {
            source,
            format,
            index: SparseIndex::empty(),
            wakeup,
        }
    }

    /// 読み方。
    pub(crate) fn format(&self) -> TextFormat {
        self.format
    }

    /// 読み方 `format` で、先頭からの窓の分の行を読む（開き直す前に試す・Issue #52）。索引は使わない。
    pub(crate) fn read_trial(&self, format: TextFormat) -> Result<Vec<Vec<String>>, ScanFailure> {
        read_rows(
            &self.source,
            &SparseIndex::empty(),
            format,
            RecordRange::new(0, WINDOW_RECORDS),
        )
    }

    /// 読み方 `format` で開き直す。索引は作り始める前に戻し、裏で作り直す仕事を始めて返す。
    pub(crate) fn reopen(&mut self, format: TextFormat) -> Result<IndexJob, ScanFailure> {
        self.format = format;
        self.index = SparseIndex::empty();
        IndexJob::spawn(
            self.source.try_clone()?,
            format,
            BlockSize::MEBIBYTE,
            Arc::clone(&self.wakeup),
        )
    }

    /// 作り終えた索引に替える。以後は近い目印から読む（BIG-02）。
    pub(crate) fn use_index(&mut self, index: SparseIndex) {
        self.index = index;
    }

    /// `first` からの窓の分の行を読み、表示の値にする。
    pub(crate) fn read(&self, first: u64) -> Result<Vec<Vec<String>>, ScanFailure> {
        self.read_count(first, WINDOW_RECORDS)
    }

    /// `first` からの `count` 個の行を読み、復号した値にする（コピー）。
    pub(crate) fn read_count(
        &self,
        first: u64,
        count: usize,
    ) -> Result<Vec<Vec<String>>, ScanFailure> {
        read_rows(
            &self.source,
            &self.index,
            self.format,
            RecordRange::new(first, count),
        )
    }

    /// `first` からの `count` 個の行を、裏のスレッドで `fold` に畳み込み始める（原本は別に読む口を作って渡す）。
    pub(crate) fn spawn_fold<F: RecordFold + 'static>(
        &self,
        first: u64,
        count: usize,
        fold: F,
    ) -> Result<RangeJob<F>, ScanFailure> {
        let from = IndexedSource::new(self.source.try_clone()?, self.index.clone(), self.format);
        RangeJob::spawn(
            from,
            RecordRange::new(first, count),
            fold,
            Arc::clone(&self.wakeup),
        )
    }
}
