//! 開いたファイルの推し量った読み方。

use nene_ledger_core::detection::Detection;
use nene_ledger_core::header_guess::HeaderGuess;
use nene_ledger_core::line_ending_counts::LineEndingCounts;
use nene_ledger_core::text_format::TextFormat;

/// 推し量った読み方（文字コードと方言）と、見出しの前の前書きの行の数と、見出しのレコードの番号と、標本の改行の内訳。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Guessed {
    format: TextFormat,
    preamble_rows: u64,
    header: Option<u64>,
    line_endings: LineEndingCounts,
    binary: bool,
}

impl Guessed {
    /// 判別が「見出しなし」と言ったとき以外は、前書きの次のレコードを見出しとする（VIEW-01・Issue #26）。
    pub(crate) fn new(format: TextFormat, detection: &Detection, binary: bool) -> Self {
        let preamble_rows = u64::try_from(detection.preamble_rows()).unwrap_or(0);
        let header = match detection.header() {
            HeaderGuess::Present | HeaderGuess::Unknown => Some(preamble_rows),
            HeaderGuess::Absent => None,
        };
        Self {
            format,
            preamble_rows,
            header,
            line_endings: detection.line_endings(),
            binary,
        }
    }

    /// 判別の標本の改行の内訳。
    /// 文字のファイルでないようか（標本に NUL がある・Issue #57）。
    pub(crate) fn looks_binary(self) -> bool {
        self.binary
    }

    pub(crate) fn line_endings(self) -> LineEndingCounts {
        self.line_endings
    }

    /// 見出しのレコードの番号。窓の上に固定する（表示だけ・原本は変えない）。
    pub(crate) fn header(self) -> Option<u64> {
        self.header
    }

    pub(crate) fn format(self) -> TextFormat {
        self.format
    }

    /// 前書きの行の数。表として書き出すときは飛ばす（窓は原本どおり全部を見せる）。
    pub(crate) fn preamble_rows(self) -> u64 {
        self.preamble_rows
    }
}
