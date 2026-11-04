//! 範囲のレコードを読み、セルを表示の値にする（開くとき・動いたとき・`--dump` の 1 つの経路）。

use nene_ledger_core::text_format::TextFormat;
use nene_ledger_core::value::value;
use nene_ledger_scan::decoding::decode;
use nene_ledger_scan::record_range::RecordRange;
use nene_ledger_scan::scan_failure::ScanFailure;
use nene_ledger_scan::source::Source;
use nene_ledger_scan::sparse_index::SparseIndex;
use nene_ledger_scan::windowing::read_records;

/// 窓のために一度に読む行の数。画面に入る行より多ければよい。
pub(crate) const WINDOW_RECORDS: usize = 200;

/// `range` の行を、索引の目印から読み、引用符とエスケープを外して復号する。
pub(crate) fn read_rows(
    source: &Source,
    index: &SparseIndex,
    format: TextFormat,
    range: RecordRange,
) -> Result<Vec<Vec<String>>, ScanFailure> {
    let dialect = format.dialect();
    let window = read_records(source, index, format, range)?;
    Ok(window
        .records()
        .iter()
        .map(|record| {
            record
                .record()
                .fields()
                .iter()
                .map(|&field| {
                    let raw = value(record.bytes(), field, &dialect);
                    decode(format.encoding(), &raw).text().to_owned()
                })
                .collect()
        })
        .collect())
}
