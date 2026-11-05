//! 範囲のレコードを読み、セルを表示の値にする（開くとき・動いたとき・`--dump` の 1 つの経路）。

use nene_ledger_application::cut_record::CutRecord;
use nene_ledger_core::text_format::TextFormat;
use nene_ledger_core::value::value;
use nene_ledger_scan::decoding::decode;
use nene_ledger_scan::record_length::RecordLength;
use nene_ledger_scan::record_range::RecordRange;
use nene_ledger_scan::scan_failure::ScanFailure;
use nene_ledger_scan::source::Source;
use nene_ledger_scan::sparse_index::SparseIndex;
use nene_ledger_scan::windowing::read_records;

/// 窓のために一度に読む行の数。画面に入る行より多ければよい。
pub(crate) const WINDOW_RECORDS: usize = 200;

/// 範囲から読んだ行（復号したセルの値）と、そのうち上限で切り詰めて先頭の部分だけを読んだレコード（GitHub Issue #36）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Rows {
    cells: Vec<Vec<String>>,
    cuts: Vec<CutRecord>,
}

impl Rows {
    /// 行の値。
    pub(crate) fn cells(&self) -> &[Vec<String>] {
        &self.cells
    }

    /// 切り詰めたレコード（番号の順）。
    pub(crate) fn cuts(&self) -> &[CutRecord] {
        &self.cuts
    }

    /// 行の値と切り詰めたレコードに分ける。
    pub(crate) fn into_parts(self) -> (Vec<Vec<String>>, Vec<CutRecord>) {
        (self.cells, self.cuts)
    }
}

/// `range` の行を、索引の目印から読み、引用符とエスケープを外して復号する。
pub(crate) fn read_rows(
    source: &Source,
    index: &SparseIndex,
    format: TextFormat,
    range: RecordRange,
) -> Result<Rows, ScanFailure> {
    let dialect = format.dialect();
    let window = read_records(source, index, format, range)?;
    let cells = window
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
        .collect();
    let cuts = window
        .records()
        .iter()
        .zip(range.first()..)
        .filter(|(record, _)| record.is_truncated())
        .map(|(record, number)| {
            let total = match record.length() {
                RecordLength::Exact(length) => Some(length),
                RecordLength::MoreThan(_) => None,
            };
            CutRecord::new(number, total)
        })
        .collect();
    Ok(Rows { cells, cuts })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// `name` に `contents` を書いた一時ファイル（`target/tmp`）。
    fn written(name: &str, contents: &[u8]) -> PathBuf {
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/tmp");
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join(format!("rows-{name}"));
        std::fs::write(&path, contents).unwrap();
        path
    }

    /// 上限を超えて先頭だけを読んだレコードを、番号と（索引が終わりを知っていれば）全体の大きさで返す（GitHub Issue #36）。
    #[test]
    fn records_cut_at_the_limit_are_listed_with_their_whole_size() {
        let mut contents = b"id,memo\n1,a\n2,\"".to_vec();
        contents.extend(core::iter::repeat_n(b'z', 2 * 1_048_576));
        contents.extend_from_slice(b"\"\n3,c\n");
        let path = written("cut.csv", &contents);
        let source = Source::open(&path).unwrap();
        let format = crate::opening::guess_format(&source).unwrap().format();
        let without_index = read_rows(
            &source,
            &SparseIndex::empty(),
            format,
            RecordRange::new(0, 10),
        );
        let cuts = without_index.unwrap().cuts().to_vec();
        assert_eq!(cuts, [CutRecord::new(2, None)]);
        let outcome = nene_ledger_scan::indexing::build_index(
            &source,
            format,
            nene_ledger_scan::block_size::BlockSize::MEBIBYTE,
            |_| core::ops::ControlFlow::Continue(()),
        )
        .unwrap();
        let nene_ledger_scan::index_outcome::IndexOutcome::Finished(index) = outcome else {
            panic!("中止を求めていない");
        };
        let rows = read_rows(&source, &index, format, RecordRange::new(0, 10)).unwrap();
        assert_eq!(rows.cells().len(), 4);
        assert_eq!(rows.cuts(), [CutRecord::new(2, Some(2 * 1_048_576 + 5))]);
    }
}
