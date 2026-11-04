//! ファイルを開き、先頭の行を表示の値にする（索引を待たない・BIG-01）。

use std::path::Path;

use nene_ledger_application::format_summary::FormatSummary;
use nene_ledger_core::binary_sniff::looks_binary;
use nene_ledger_core::detection::Detection;
use nene_ledger_core::encoding_guess::EncodingGuess;
use nene_ledger_core::text_format::TextFormat;
use nene_ledger_scan::block_size::BlockSize;
use nene_ledger_scan::record_range::RecordRange;
use nene_ledger_scan::source::Source;
use nene_ledger_scan::sparse_index::SparseIndex;

use crate::guessed::Guessed;
use crate::open_failure::OpenFailure;
use crate::rows::{WINDOW_RECORDS, read_rows};

/// 判別に読む量（方言の判別の標本と同じ 64 KiB）。
const SAMPLE_BYTES: usize = 64 * 1024;

/// 開いた先頭の行と、窓の題名と、推し量った読み方（の要約）と、見出しのレコードの番号。
#[derive(Debug)]
pub(crate) struct Opened {
    rows: Vec<Vec<String>>,
    title: String,
    summary: FormatSummary,
    header: Option<u64>,
    binary: bool,
}

impl Opened {
    /// 読み方の要約（状態の行に出す）。
    pub(crate) fn summary(&self) -> FormatSummary {
        self.summary
    }

    /// 先頭から読んだ行（復号したセルの値）。
    pub(crate) fn rows(&self) -> &[Vec<String>] {
        &self.rows
    }

    /// 文字のファイルでないようか（Issue #57）。
    pub(crate) fn looks_binary(&self) -> bool {
        self.binary
    }

    /// 見出しのレコードの番号。
    pub(crate) fn header(&self) -> Option<u64> {
        self.header
    }

    pub(crate) fn title(&self) -> &str {
        &self.title
    }
}

/// `path` を開き、文字コードと方言を推し量り、先頭の行を表示の値にする。
pub(crate) fn open(path: &Path) -> Result<Opened, OpenFailure> {
    let source = Source::open(path)?;
    let guessed = guess_format(&source)?;
    let format = guessed.format();
    let rows = read_rows(
        &source,
        &SparseIndex::empty(),
        format,
        RecordRange::new(0, WINDOW_RECORDS),
    )?;
    let name = path
        .file_name()
        .map_or_else(|| path.to_string_lossy(), |name| name.to_string_lossy());
    Ok(Opened {
        rows,
        title: format!("{name} — NeNe Ledger"),
        summary: FormatSummary::new(format, guessed.line_endings()),
        header: guessed.header(),
        binary: guessed.looks_binary(),
    })
}

/// 先頭を読んで文字コードを推し量り、BOM の後ろから方言を推し量る。
pub(crate) fn guess_format(source: &Source) -> Result<Guessed, OpenFailure> {
    let size = BlockSize::new(SAMPLE_BYTES).unwrap_or(BlockSize::MEBIBYTE);
    let sample = source.read_block(0, size)?;
    let encoding = EncodingGuess::of(&sample).best();
    let start = usize::try_from(source.content_start(encoding)?).unwrap_or(0);
    let content = sample.get(start..).unwrap_or_default();
    let detection = Detection::of_encoded(content, encoding.units());
    let format = TextFormat::new(encoding, detection.best());
    let binary = looks_binary(content, encoding.units());
    Ok(Guessed::new(format, &detection, binary))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join(name)
    }

    #[test]
    fn values_that_look_like_dates_numbers_or_genes_are_kept_as_text() {
        let path = fixture("keep-values.csv");
        let raw = std::fs::read_to_string(&path).unwrap();
        let expected: Vec<Vec<String>> = raw
            .lines()
            .map(|line| line.split(',').map(String::from).collect())
            .collect();
        let opened = open(&path).unwrap();
        assert_eq!(opened.header(), Some(0));
        assert_eq!(opened.rows(), expected.as_slice());
        assert!(expected.iter().any(|row| row[1] == "MARCH1"));
        assert!(expected.iter().any(|row| row[1] == " 7 "));
    }

    #[test]
    fn duplicate_and_empty_headers_are_kept_as_they_are() {
        let opened = open(&fixture("duplicate-headers.csv")).unwrap();
        assert_eq!(opened.header(), Some(0));
        assert_eq!(opened.rows()[0], ["id", "名前", "名前", "", "id"]);
    }

    #[test]
    fn the_screenshot_sample_opens_with_its_bom_skipped() {
        let opened = open(&fixture("screenshot.csv")).unwrap();
        assert_eq!(opened.header(), Some(0));
        let rows = opened.rows();
        assert_eq!(rows.len(), 7);
        assert_eq!(rows[0][0], "社員番号");
        assert_eq!(rows[1][0], "00123");
        // 改行の見せ方（¶）は画面の状態が決める。ここは復号した値のまま。
        assert_eq!(rows[1][3], "東京都千代田区\n丸の内1-1-1");
        assert_eq!(rows[3][1], "Smith, John");
        assert_eq!(rows[3][5], "引用符の中の \"二重\" 引用符");
        assert_eq!(rows[5].len(), 2);
        assert_eq!(rows[6][4], "1.20E+03");
    }

    #[test]
    fn the_title_and_the_guessed_encoding_are_kept() {
        let opened = open(&fixture("screenshot.csv")).unwrap();
        assert_eq!(opened.title(), "screenshot.csv — NeNe Ledger");
        assert_eq!(
            opened.summary().format().encoding(),
            nene_ledger_core::text_encoding::TextEncoding::Utf8WithBom
        );
    }

    #[test]
    fn a_pipe_delimited_shift_jis_file_opens_and_keeps_its_kanji_whole() {
        // 「表」（95 5C）・「ソ」（83 5C）の 2 バイト目はバックスラッシュ。Issue #18 では断っていた（ADR 0019）。
        let opened = open(&fixture("shift-jis-pipe.csv")).unwrap();
        let format = opened.summary().format();
        assert_eq!(
            format.encoding(),
            nene_ledger_core::text_encoding::TextEncoding::ShiftJis
        );
        assert_eq!(format.dialect().delimiter().byte(), b'|');
        let expected: Vec<Vec<String>> = [
            ["コード", "品名", "単価"],
            ["1", "表計算ソフト", "1200"],
            ["2", "能力表", "300"],
            ["3", "ソース", "50"],
        ]
        .iter()
        .map(|row| row.iter().map(|cell| String::from(*cell)).collect())
        .collect();
        assert_eq!(opened.rows(), expected.as_slice());
    }

    #[test]
    fn an_excel_unicode_text_file_opens_as_utf16_tabs() {
        // Excel の「Unicode テキスト」の形（BOM 付きの UTF-16LE・タブ区切り・CRLF）。Issue #18 では断っていた（ADR 0019）。
        let opened = open(&fixture("utf16le-tab.txt")).unwrap();
        let format = opened.summary().format();
        assert_eq!(
            format.encoding(),
            nene_ledger_core::text_encoding::TextEncoding::Utf16LeWithBom
        );
        assert_eq!(format.dialect().delimiter().byte(), b'\t');
        assert_eq!(opened.header(), Some(0));
        let expected: Vec<Vec<String>> = [
            &["社員番号", "氏名", "所属", "備考"][..],
            &["0012", "山田 太郎", "経理部", "月末\t締め"][..],
            &["0034", "𠮷田 花子", "営業部", ""][..],
            &["0056", "Smith, John", "開発部", "1.20E+03"][..],
        ]
        .iter()
        .map(|row| row.iter().map(|cell| String::from(*cell)).collect())
        .collect();
        assert_eq!(opened.rows(), expected.as_slice());
    }
}
