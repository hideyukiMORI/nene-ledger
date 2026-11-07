//! 保存の計画のために、原本のレコードを scan の範囲の取り出しで引く（application の `RecordLookup` の実装）。

use nene_ledger_application::located_record::LocatedRecord;
use nene_ledger_application::record_lookup::RecordLookup;
use nene_ledger_core::text_format::TextFormat;
use nene_ledger_scan::mark::Mark;
use nene_ledger_scan::record_range::RecordRange;
use nene_ledger_scan::scan_failure::ScanFailure;
use nene_ledger_scan::source::Source;
use nene_ledger_scan::sparse_index::SparseIndex;
use nene_ledger_scan::windowing::read_records_after;

/// 原本のレコードを、範囲の取り出しで引く。前に引いたレコードの始まりを覚えておき、番号の順に引くときは目印からでなく
/// そこから探す（GitHub Issue #209）。
pub(crate) struct ScanLookup<'a> {
    source: &'a Source,
    index: &'a SparseIndex,
    format: TextFormat,
    known: Option<Mark>,
}

impl<'a> ScanLookup<'a> {
    /// 原本 `source` を、索引 `index` と読み方 `format` で引く。
    pub(crate) fn new(source: &'a Source, index: &'a SparseIndex, format: TextFormat) -> Self {
        Self {
            source,
            index,
            format,
            known: None,
        }
    }
}

impl RecordLookup for ScanLookup<'_> {
    type Failure = ScanFailure;

    fn record(&mut self, number: u64) -> Result<Option<LocatedRecord>, ScanFailure> {
        Ok(self.records(number, 1)?.into_iter().next())
    }

    fn records(&mut self, first: u64, count: usize) -> Result<Vec<LocatedRecord>, ScanFailure> {
        let window = read_records_after(
            self.source,
            self.index,
            self.format,
            (RecordRange::new(first, count), self.known),
        )?;
        if let Some(mark) = window.last_mark(first) {
            self.known = Some(mark);
        }
        Ok(window
            .records()
            .iter()
            .map(|found| {
                LocatedRecord::new(
                    found.offset(),
                    found.bytes().to_vec(),
                    found.record().clone(),
                    !found.is_truncated(),
                )
            })
            .collect())
    }
}
