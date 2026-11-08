//! 動いた先の範囲を読む。

use core::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use nene_ledger_application::file_stamp::FileStamp;
use nene_ledger_application::wakeup::Wakeup;
use nene_ledger_core::dialect_candidate::DialectCandidate;
use nene_ledger_core::header_rows::HeaderRows;
use nene_ledger_core::line_ending_counts::LineEndingCounts;
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
use nene_ledger_scan::split_fold::SplitFold;
use nene_ledger_scan::split_job::{SplitJob, usual_parts};
use nene_ledger_store::stamping::stamp_of;

use crate::opening::detect;
use crate::rows::{Rows, read_columns, read_first_window};

/// 動いた先の範囲を読む材料。画面のスレッドが持つ原本と読み方と、作り終えた索引（まだなら作り始める前の索引）と、
/// 裏の仕事が画面のスレッドを起こす口。
pub(crate) struct Reader {
    source: Source,
    format: TextFormat,
    index: SparseIndex,
    wakeup: Arc<dyn Wakeup>,
    /// 原本の道筋（保存と、保存の後の開き直しに使う）。新しい表は `None`（ADR 0028）。
    path: Option<PathBuf>,
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
        (source, path, opened): (Source, Option<PathBuf>, Option<FileStamp>),
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

    /// 原本の道筋。新しい表は `None`。
    pub(crate) fn path(&self) -> Option<&Path> {
        self.path.as_deref()
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

    /// 今の索引の写し（裏の保存が計画を作るときに使う・GitHub Issue #130）。
    pub(crate) fn index_copy(&self) -> SparseIndex {
        self.index.clone()
    }

    /// 保存して置き換えた後のファイルを、同じ読み方で開き直して索引を作り直す（ADR 0026 の決定 6）。
    /// 置き換えの前の口（Linux では前のファイルを指したまま）は捨てる。名前を付けて保存なら `saved_as` の道筋を開き、
    /// 開けたときだけ、これからの原本の道筋にする（開けなければ前の原本・道筋・印のまま・ADR 0028 の決定 5）。
    pub(crate) fn reopen_saved(
        &mut self,
        saved_as: Option<PathBuf>,
    ) -> Result<IndexJob, ScanFailure> {
        let path = saved_as
            .or_else(|| self.path.clone())
            .ok_or(ScanFailure::Open(std::io::ErrorKind::NotFound))?;
        // 印は開く前に取る（開いた後の外からの変更を次の保存で見つける）。
        let opened = stamp_of(&path).ok();
        let source = Source::open(&path)?;
        let job = IndexJob::spawn(
            source.try_clone()?,
            self.format,
            BlockSize::MEBIBYTE,
            Arc::clone(&self.wakeup),
        )?;
        self.source = source;
        self.index = SparseIndex::empty();
        self.opened = opened;
        self.path = Some(path);
        Ok(job)
    }

    /// 原本の先頭の標本から数えた改行の内訳（名前を付けて保存して開き直した後に数え直す・ADR 0028）。読めなければ `None`。
    pub(crate) fn line_endings(&self) -> Option<LineEndingCounts> {
        crate::opening::guess_format(&self.source)
            .ok()
            .map(crate::guessed::Guessed::line_endings)
    }

    /// 読み方。
    pub(crate) fn format(&self) -> TextFormat {
        self.format
    }

    /// 読み方 `format`・見出しのレコード `header` で、先頭からの窓の分の行を読む（開き直す前に試す・Issue #52）。索引は
    /// 使わない。開くときと同じ読み（`read_first_window`）。
    pub(crate) fn read_trial(
        &self,
        format: TextFormat,
        header: Option<HeaderRows>,
    ) -> Result<Rows, ScanFailure> {
        read_first_window(&self.source, format, header)
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

    /// `first` からの `count` 個の行を読み、復号した値にする（表示の窓・コピー・集計。行の並びの区間ごとに呼ぶ）。
    /// 復号するのは元の列の範囲 `columns` だけ（`None` なら全部・表示の窓は見えている列の範囲・ADR 0033 の決定 1）。
    pub(crate) fn read_count(
        &self,
        first: u64,
        count: usize,
        columns: Option<&[Range<usize>]>,
    ) -> Result<Rows, ScanFailure> {
        read_columns(
            &self.source,
            (&self.index, self.format),
            (RecordRange::new(first, count), columns),
        )
    }

    /// `first` からの `count` 個の行を、索引の目印で分けて裏のいくつかのスレッドで `fold` の写しに畳み込み始める
    /// （索引を作り終える前は 1 つ・ADR 0030 の決定 4）。
    pub(crate) fn spawn_split<F: SplitFold>(
        &self,
        first: u64,
        count: usize,
        fold: F,
    ) -> Result<SplitJob<F>, ScanFailure> {
        let from = IndexedSource::new(self.source.try_clone()?, self.index.clone(), self.format);
        SplitJob::spawn(
            &from,
            RecordRange::new(first, count),
            (fold, usual_parts()),
            &self.wakeup,
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
