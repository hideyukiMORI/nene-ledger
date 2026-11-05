//! 保存の計画のために、原本のレコードを scan の範囲の取り出しで引く（application の `RecordLookup` の実装）。

use nene_ledger_application::located_record::LocatedRecord;
use nene_ledger_application::record_lookup::RecordLookup;
use nene_ledger_core::text_format::TextFormat;
use nene_ledger_scan::record_range::RecordRange;
use nene_ledger_scan::scan_failure::ScanFailure;
use nene_ledger_scan::source::Source;
use nene_ledger_scan::sparse_index::SparseIndex;
use nene_ledger_scan::windowing::read_records;

/// 原本のレコードを、範囲の取り出しで引く。
pub(crate) struct ScanLookup<'a> {
    source: &'a Source,
    index: &'a SparseIndex,
    format: TextFormat,
}

impl<'a> ScanLookup<'a> {
    /// 原本 `source` を、索引 `index` と読み方 `format` で引く。
    pub(crate) fn new(source: &'a Source, index: &'a SparseIndex, format: TextFormat) -> Self {
        Self {
            source,
            index,
            format,
        }
    }
}

impl RecordLookup for ScanLookup<'_> {
    type Failure = ScanFailure;

    fn record(&mut self, number: u64) -> Result<Option<LocatedRecord>, ScanFailure> {
        let window = read_records(
            self.source,
            self.index,
            self.format,
            RecordRange::new(number, 1),
        )?;
        Ok(window.records().first().map(|found| {
            LocatedRecord::new(
                found.offset(),
                found.bytes().to_vec(),
                found.record().clone(),
                !found.is_truncated(),
            )
        }))
    }
}
