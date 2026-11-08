//! 窓を出さずに、全部のレコードを RFC 4180 の UTF-8 で標準出力に書く（測定・BIG-11）。

use std::io::{BufWriter, ErrorKind, Write};
use std::ops::ControlFlow;
use std::path::Path;

use nene_ledger_core::csv_writing::rfc4180_record;
use nene_ledger_scan::block_size::BlockSize;
use nene_ledger_scan::index_outcome::IndexOutcome;
use nene_ledger_scan::indexing::build_index;
use nene_ledger_scan::record_range::RecordRange;
use nene_ledger_scan::source::Source;
use nene_ledger_scan::sparse_index::SparseIndex;

use nene_ledger_application::cut_record::CutRecord;

use crate::open_failure::OpenFailure;
use crate::opening::guess_format;
use crate::rows::read_rows;

/// 一度に読む行の数（索引の目印の間隔と同じにして、毎回目印から読む）。
const DUMP_RECORDS: usize = 1024;

/// `path` を窓と同じ経路（判別・索引・範囲の取り出し・復号）で読み、全部のレコードを標準出力に書く。
///
/// 🔴 Pollock などの測定のための出力で、保存ではない（原本のバイト列を写さず、値を書き直す）。
/// 読む側が先に閉じた（`| head`）ら、そこで黙って終える（GitHub Issue #20。`print!` は閉じた管で panic した）。
///
/// 上限（1 MiB）を超えて先頭の部分だけを書いたレコードがあれば、その数と最初の行を返す（呼び出し側が標準エラーで
/// 知らせる・GitHub Issue #36。測定で値の欠けが分からなかった）。
pub(crate) fn dump(path: &Path) -> Result<DumpCuts, OpenFailure> {
    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    let written = dump_to(path, &mut out)
        .and_then(|cuts| out.flush().map(|()| cuts).map_err(OpenFailure::Write));
    quiet_when_the_reader_left(written)
}

/// `--dump` で切り詰めたレコード。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DumpCuts {
    count: u64,
    first: Option<u64>,
}

impl DumpCuts {
    /// 切り詰めたレコードが無い。
    const fn none() -> Self {
        Self {
            count: 0,
            first: None,
        }
    }

    /// 切り詰めたレコードの数と、最初のレコードの番号（0 始まり）。無ければ `None`。
    pub(crate) fn summary(self) -> Option<(u64, u64)> {
        self.first.map(|first| (self.count, first))
    }

    fn add(&mut self, cuts: &[CutRecord]) {
        self.count = self
            .count
            .saturating_add(u64::try_from(cuts.len()).unwrap_or(u64::MAX));
        if self.first.is_none() {
            self.first = cuts.first().map(|cut| cut.record());
        }
    }
}

/// 読む側が閉じた（`BrokenPipe`）だけなら成功とみなす。
fn quiet_when_the_reader_left(
    result: Result<DumpCuts, OpenFailure>,
) -> Result<DumpCuts, OpenFailure> {
    match result {
        Err(OpenFailure::Write(error)) if error.kind() == ErrorKind::BrokenPipe => {
            Ok(DumpCuts::none())
        }
        Ok(_) | Err(OpenFailure::Scan(_) | OpenFailure::Write(_)) => result,
    }
}

/// `path` の全部のレコードを、RFC 4180 の UTF-8 で `out` に書く。
fn dump_to(path: &Path, out: &mut impl Write) -> Result<DumpCuts, OpenFailure> {
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
    let mut cuts = DumpCuts::none();
    loop {
        let rows = read_rows(
            &source,
            &index,
            format,
            RecordRange::new(first, DUMP_RECORDS),
        )?;
        cuts.add(rows.cuts());
        let rows = rows.cells();
        if rows.is_empty() {
            return Ok(cuts);
        }
        for row in rows {
            out.write_all(rfc4180_record(row.iter()).as_bytes())
                .map_err(OpenFailure::Write)?;
        }
        first = first.saturating_add(u64::try_from(rows.len()).unwrap_or(u64::MAX));
    }
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
        rows.cells()
            .iter()
            .map(|row| rfc4180_record(row.iter()))
            .collect()
    }

    /// 書き込みを `failure` で断る書き手。
    struct Refusing(ErrorKind);

    impl Write for Refusing {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::from(self.0))
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join(name)
    }

    /// 読む側が閉じたら黙って終え、ほかの書き込みの失敗は失敗として返す（GitHub Issue #20）。
    #[test]
    fn a_closed_reader_ends_quietly_but_other_write_failures_are_reported() {
        let path = fixture("screenshot.csv");
        let closed = dump_to(&path, &mut Refusing(ErrorKind::BrokenPipe));
        assert!(matches!(closed, Err(OpenFailure::Write(_))));
        assert!(quiet_when_the_reader_left(closed).is_ok());
        let full = dump_to(&path, &mut Refusing(ErrorKind::StorageFull));
        assert!(quiet_when_the_reader_left(full).is_err());
    }

    /// `dump` と同じ関数で書いた出力が、範囲の取り出しの行と同じ。
    #[test]
    fn the_dump_writes_the_same_rows_as_the_window() {
        let mut out = Vec::new();
        dump_to(&fixture("screenshot.csv"), &mut out).unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), dumped("screenshot.csv"));
    }

    /// 上限を超えて先頭だけを書いたレコードを数え、最初の番号を返す（GitHub Issue #36）。
    #[test]
    fn the_dump_counts_records_cut_at_the_limit() {
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/tmp");
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("dumping-cut.csv");
        let mut contents = b"id,memo\n1,\"".to_vec();
        contents.extend(core::iter::repeat_n(b'z', 2 * 1_048_576));
        contents.extend_from_slice(b"\"\n2,b\n");
        std::fs::write(&path, contents).unwrap();
        let mut out = Vec::new();
        assert_eq!(dump_to(&path, &mut out).unwrap().summary(), Some((1, 1)));
        let mut plain = Vec::new();
        assert_eq!(
            dump_to(&fixture("screenshot.csv"), &mut plain)
                .unwrap()
                .summary(),
            None
        );
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
