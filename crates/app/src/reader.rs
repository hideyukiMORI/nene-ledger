//! 動いた先の範囲を読む。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use nene_ledger_application::file_stamp::FileStamp;
use nene_ledger_application::wakeup::Wakeup;
use nene_ledger_core::dialect_candidate::DialectCandidate;
use nene_ledger_core::text_encoding::TextEncoding;
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
use nene_ledger_store::stamping::stamp_of;

use crate::opening::detect;
use crate::rows::{Rows, WINDOW_RECORDS, read_rows};
use crate::scan_lookup::ScanLookup;

/// 動いた先の範囲を読む材料。画面のスレッドが持つ原本と読み方と、作り終えた索引（まだなら作り始める前の索引）と、
/// 裏の仕事が画面のスレッドを起こす口。
pub(crate) struct Reader {
    source: Source,
    format: TextFormat,
    index: SparseIndex,
    wakeup: Arc<dyn Wakeup>,
    /// 原本の道筋（保存と、保存の後の開き直しに使う）。
    path: PathBuf,
    /// 開く前に取った原本の印（保存の前に比べる・FILE-01）。取れなければ `None`（保存できない）。
    opened: Option<FileStamp>,
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
    pub(crate) fn new(
        (source, path, opened): (Source, PathBuf, Option<FileStamp>),
        format: TextFormat,
        wakeup: Arc<dyn Wakeup>,
    ) -> Self {
        Self {
            source,
            format,
            index: SparseIndex::empty(),
            wakeup,
            path,
            opened,
        }
    }

    /// 開く前に取った原本の印。
    pub(crate) fn opened(&self) -> Option<FileStamp> {
        self.opened
    }

    /// 原本の道筋。
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// 窓を起こす口（裏の仕事に渡す）。
    pub(crate) fn wakeup(&self) -> Arc<dyn Wakeup> {
        Arc::clone(&self.wakeup)
    }

    /// 原本の大きさ（バイト）。
    pub(crate) fn source_length(&self) -> u64 {
        self.source.len()
    }

    /// 原本の別の口（裏の保存が読む）。
    pub(crate) fn source_copy(&self) -> Result<Source, ScanFailure> {
        self.source.try_clone()
    }

    /// 保存の計画のために、原本のレコードを今の索引と読み方で引く口。
    pub(crate) fn lookup(&self) -> ScanLookup<'_> {
        ScanLookup::new(&self.source, &self.index, self.format)
    }

    /// 保存して置き換えた後のファイルを、同じ読み方で開き直して索引を作り直す（ADR 0026 の決定 6）。
    /// 置き換えの前の口（Linux では前のファイルを指したまま）は捨てる。
    pub(crate) fn reopen_saved(&mut self) -> Result<IndexJob, ScanFailure> {
        // 印は開く前に取る（開いた後の外からの変更を次の保存で見つける）。
        let opened = stamp_of(&self.path).ok();
        let source = Source::open(&self.path)?;
        let job = IndexJob::spawn(
            source.try_clone()?,
            self.format,
            BlockSize::MEBIBYTE,
            Arc::clone(&self.wakeup),
        )?;
        self.source = source;
        self.index = SparseIndex::empty();
        self.opened = opened;
        Ok(job)
    }

    /// 読み方。
    pub(crate) fn format(&self) -> TextFormat {
        self.format
    }

    /// 読み方 `format` で、先頭からの窓の分の行を読む（開き直す前に試す・Issue #52）。索引は使わない。
    pub(crate) fn read_trial(&self, format: TextFormat) -> Result<Rows, ScanFailure> {
        read_rows(
            &self.source,
            &SparseIndex::empty(),
            format,
            RecordRange::new(0, WINDOW_RECORDS),
        )
    }

    /// 文字コード `encoding` で先頭の標本から方言を推し量った候補（点数の順）。判別の候補を選ぶ段で見せる（Issue #3）。
    pub(crate) fn candidates(
        &self,
        encoding: TextEncoding,
    ) -> Result<Vec<DialectCandidate>, ScanFailure> {
        Ok(detect(&self.source, encoding)?.candidates().to_vec())
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
    pub(crate) fn read(&self, first: u64) -> Result<Rows, ScanFailure> {
        self.read_count(first, WINDOW_RECORDS)
    }

    /// `first` からの `count` 個の行を読み、復号した値にする（コピー）。
    pub(crate) fn read_count(&self, first: u64, count: usize) -> Result<Rows, ScanFailure> {
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
