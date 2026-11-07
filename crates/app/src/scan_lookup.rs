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

/// 原本のレコードを、範囲の取り出しで引く。最近 2 回に引いたレコードの始まりを覚えておき、番号の順に引くときは目印からでなく
/// そこから探す（GitHub Issue #209）。2 つ覚えるのは、保存の計画が区間の終わりを先に引いてから、区間の中の行を順に引くため
/// （1 つだけでは、先の終わりに上書きされて役に立たない）。
pub(crate) struct ScanLookup<'a> {
    source: &'a Source,
    index: &'a SparseIndex,
    format: TextFormat,
    known: [Option<Mark>; 2],
}

impl<'a> ScanLookup<'a> {
    /// 原本 `source` を、索引 `index` と読み方 `format` で引く。
    pub(crate) fn new(source: &'a Source, index: &'a SparseIndex, format: TextFormat) -> Self {
        Self {
            source,
            index,
            format,
            known: [None, None],
        }
    }
}

impl RecordLookup for ScanLookup<'_> {
    type Failure = ScanFailure;

    fn record(&mut self, number: u64) -> Result<Option<LocatedRecord>, ScanFailure> {
        Ok(self.records(number, 1)?.into_iter().next())
    }

    fn records(&mut self, first: u64, count: usize) -> Result<Vec<LocatedRecord>, ScanFailure> {
        let best = self
            .known
            .into_iter()
            .flatten()
            .filter(|mark| mark.record() <= first)
            .max_by_key(|mark| mark.record());
        let window = read_records_after(
            self.source,
            self.index,
            self.format,
            (RecordRange::new(first, count), best),
        )?;
        if let Some(mark) = window.last_mark() {
            self.known = [self.known[1], Some(mark)];
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
