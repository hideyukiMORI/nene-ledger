//! 保存の計画を、一時ファイルに直に書く書き先（GitHub Issue #130）。

use std::fs::File;
use std::io::{BufWriter, ErrorKind, Write};

use nene_ledger_application::source_bytes::SourceBytes;
use nene_ledger_core::plan_sink::PlanSink;
use nene_ledger_core::text_encoding::TextEncoding;

use crate::encoding::encoded;
use crate::save_failure::SaveFailure;

/// 原本を読み置く塊の大きさ（と書く側の緩衝の大きさ）。
const CHUNK: usize = 1 << 20;

/// 保存の計画を、一時ファイルに直に書く書き先。歩の列を記憶に持たない。
///
/// 写す区間は、原本を [`CHUNK`] ずつ読み置いた塊から写す（列を足し消しした表の、1 行に十数の小さな区間が続いても、
/// 原本を区間ごとに読まない）。文字列はその場でファイルの文字コードで符号化する。書けなくなったら最初の理由を覚え、
/// 以後の歩は捨てる（[`PlanSink::stopped`]）。記憶は塊の大きさにしか比例しない。
pub(crate) struct FileSink<'source, S> {
    out: BufWriter<File>,
    encoding: TextEncoding,
    source: &'source mut S,
    cache: Vec<u8>,
    cache_at: u64,
    failure: Option<SaveFailure>,
}

impl<'source, S: SourceBytes<Failure = ErrorKind>> FileSink<'source, S> {
    /// 一時ファイル `file` に、文字コード `encoding` で書き、原本は `source` から読む。
    pub(crate) fn new(file: File, encoding: TextEncoding, source: &'source mut S) -> Self {
        Self {
            out: BufWriter::with_capacity(CHUNK, file),
            encoding,
            source,
            cache: Vec::new(),
            cache_at: 0,
            failure: None,
        }
    }

    /// 書き終えた。計画を作る側の結果 `planned` より、書き先の失敗を先に返す。書けていれば同期する。
    pub(crate) fn finish(self, planned: Result<(), SaveFailure>) -> Result<(), SaveFailure> {
        if let Some(failure) = self.failure {
            return Err(failure);
        }
        planned?;
        let file = self
            .out
            .into_inner()
            .map_err(|error| SaveFailure::Write(error.error().kind()))?;
        file.sync_all()
            .map_err(|error| SaveFailure::Write(error.kind()))
    }

    /// 写す区間の 1 つ。
    fn copy_range(&mut self, mut offset: u64, mut length: u64) -> Result<(), SaveFailure> {
        while length > 0 {
            let start = self.cached(offset)?;
            let rest = self.cache.get(start..).unwrap_or_default();
            let wanted = usize::try_from(length).unwrap_or(usize::MAX);
            let part = rest.get(..wanted.min(rest.len())).unwrap_or_default();
            self.out
                .write_all(part)
                .map_err(|error| SaveFailure::Write(error.kind()))?;
            let done = u64::try_from(part.len()).unwrap_or(u64::MAX);
            offset = offset.saturating_add(done);
            length = length.saturating_sub(done);
        }
        Ok(())
    }

    /// 原本の `offset` の、読み置いた塊の中の位置。塊の外なら `offset` から読み直す。原本が短ければ `SourceShrank`。
    fn cached(&mut self, offset: u64) -> Result<usize, SaveFailure> {
        let end = self
            .cache_at
            .saturating_add(u64::try_from(self.cache.len()).unwrap_or(u64::MAX));
        if !(self.cache_at <= offset && offset < end) {
            self.refill(offset)?;
        }
        let start = usize::try_from(offset.saturating_sub(self.cache_at)).unwrap_or(usize::MAX);
        if start >= self.cache.len() {
            return Err(SaveFailure::SourceShrank);
        }
        Ok(start)
    }

    /// 原本の `offset` から [`CHUNK`] までを読み置く。
    fn refill(&mut self, offset: u64) -> Result<(), SaveFailure> {
        self.cache.resize(CHUNK, 0);
        let mut filled = 0_usize;
        while let Some(rest) = self.cache.get_mut(filled..).filter(|rest| !rest.is_empty()) {
            let at = offset.saturating_add(u64::try_from(filled).unwrap_or(u64::MAX));
            match self.source.read_at(at, rest) {
                Ok(0) => break,
                Ok(read) => filled = filled.saturating_add(read),
                Err(ErrorKind::Interrupted) => {}
                Err(kind) => return Err(SaveFailure::Read(kind)),
            }
        }
        self.cache.truncate(filled);
        self.cache_at = offset;
        Ok(())
    }

    /// 文字列の 1 つ。
    fn write_text(&mut self, text: &str) -> Result<(), SaveFailure> {
        let bytes = encoded(text, self.encoding)
            .ok_or_else(|| SaveFailure::Unrepresentable(String::from(text)))?;
        self.out
            .write_all(&bytes)
            .map_err(|error| SaveFailure::Write(error.kind()))
    }
}

impl<S: SourceBytes<Failure = ErrorKind>> PlanSink for FileSink<'_, S> {
    fn copy(&mut self, offset: u64, length: u64) {
        if self.failure.is_none()
            && let Err(failure) = self.copy_range(offset, length)
        {
            self.failure = Some(failure);
        }
    }

    fn text(&mut self, text: &str) {
        if self.failure.is_none()
            && let Err(failure) = self.write_text(text)
        {
            self.failure = Some(failure);
        }
    }

    fn stopped(&self) -> bool {
        self.failure.is_some()
    }
}
