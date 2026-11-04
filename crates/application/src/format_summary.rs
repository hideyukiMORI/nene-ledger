//! 開いたファイルの読み方の要約。

use nene_ledger_core::line_ending_counts::LineEndingCounts;
use nene_ledger_core::text_format::TextFormat;

/// 開いたファイルの読み方の要約。推し量った文字コードと方言と、判別の標本の改行の内訳。状態の行に常に出す（VIEW-13）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatSummary {
    format: TextFormat,
    line_endings: LineEndingCounts,
}

impl FormatSummary {
    /// 要約を作る。
    #[must_use]
    pub fn new(format: TextFormat, line_endings: LineEndingCounts) -> Self {
        Self {
            format,
            line_endings,
        }
    }

    /// 文字コードと方言。
    #[must_use]
    pub fn format(self) -> TextFormat {
        self.format
    }

    /// 判別の標本の改行の内訳。
    #[must_use]
    pub fn line_endings(self) -> LineEndingCounts {
        self.line_endings
    }
}
