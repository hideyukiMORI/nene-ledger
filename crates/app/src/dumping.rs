//! 窓を出さずに、全部のレコードを RFC 4180 の UTF-8 で標準出力に書く（測定・BIG-11）。

use std::ops::ControlFlow;
use std::path::Path;

use nene_ledger_core::csv_writing::rfc4180_record;
use nene_ledger_scan::block_size::BlockSize;
use nene_ledger_scan::index_outcome::IndexOutcome;
use nene_ledger_scan::indexing::build_index;
use nene_ledger_scan::record_range::RecordRange;
use nene_ledger_scan::source::Source;
use nene_ledger_scan::sparse_index::SparseIndex;

use crate::open_failure::OpenFailure;
use crate::opening::guess_format;
use crate::rows::read_rows;

/// 一度に読む行の数（索引の目印の間隔と同じにして、毎回目印から読む）。
const DUMP_RECORDS: usize = 1024;

/// `path` を窓と同じ経路（判別・索引・範囲の取り出し・復号）で読み、全部のレコードを書く。
///
/// 🔴 Pollock などの測定のための出力で、保存ではない（原本のバイト列を写さず、値を書き直す）。
pub(crate) fn dump(path: &Path) -> Result<(), OpenFailure> {
    let source = Source::open(path)?;
    let guessed = guess_format(&source)?;
    let format = guessed.format();
    let outcome = build_index(&source, format, BlockSize::MEBIBYTE, |_| {
        ControlFlow::Continue(())
    })?;
    let index = match outcome {
        IndexOutcome::Finished(index) => index,
        // 中止を求めないので来ないが、来ても先頭から切り進めれば同じ結果になる。
        IndexOutcome::Cancelled => SparseIndex::empty(),
    };
    // 前書きは表に含めない（表として読み込む・Pollock の正しい版と同じ）。
    let mut first = guessed.preamble_rows();
    loop {
        let rows = read_rows(
            &source,
            &index,
            format,
            RecordRange::new(first, DUMP_RECORDS),
        )?;
        if rows.is_empty() {
            return Ok(());
        }
        for row in &rows {
            emit(&rfc4180_record(row));
        }
        first = first.saturating_add(u64::try_from(rows.len()).unwrap_or(u64::MAX));
    }
}

#[expect(
    clippy::print_stdout,
    reason = "RS-014: 合成ルートの測定の出力（--dump）は標準出力に書いてよい唯一の場所"
)]
fn emit(line: &str) {
    print!("{line}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rows::read_rows;
    use nene_ledger_core::csv_writing::rfc4180_record;
    use std::path::PathBuf;

    /// `dump` が書く行を、標準出力の代わりに集める（同じ経路で）。
    fn dumped(name: &str) -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join(name);
        let source = Source::open(&path).unwrap();
        let format = guess_format(&source).unwrap().format();
        let rows = read_rows(
            &source,
            &SparseIndex::empty(),
            format,
            RecordRange::new(0, 100),
        )
        .unwrap();
        rows.iter().map(|row| rfc4180_record(row)).collect()
    }

    #[test]
    fn a_preamble_is_found_so_that_dump_can_skip_it() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/preamble.csv");
        let guessed = guess_format(&Source::open(&path).unwrap()).unwrap();
        assert_eq!(guessed.preamble_rows(), 2);
        assert!(dump(&path).is_ok());
    }

    #[test]
    fn values_that_look_like_dates_numbers_or_genes_are_written_back_unchanged() {
        let raw = std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/keep-values.csv"),
        )
        .unwrap();
        assert_eq!(dumped("keep-values.csv"), raw.replace('\n', "\r\n"));
    }

    #[test]
    fn the_sample_is_written_back_as_utf8_rfc4180() {
        let text = dumped("screenshot.csv");
        let lines: Vec<&str> = text.split("\r\n").collect();
        assert_eq!(lines[0], "社員番号,氏名,所属,住所,入社日,備考");
        assert_eq!(
            lines[1],
            "00123,山田 太郎,経理部,\"東京都千代田区\n丸の内1-1-1\",2019-04-01,先頭の 0 を保つ"
        );
        assert!(text.contains("\"Smith, John\""));
        assert!(text.contains("\"引用符の中の \"\"二重\"\" 引用符\""));
        assert!(
            dump(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/screenshot.csv"))
                .is_ok()
        );
    }
}
