//! 足した行の書き方（ADR 0027 の決定 5）。

use nene_ledger_core::line_ending::LineEnding;

/// 足した行の書き方。セルの数（表の列の数）と終わり方（ファイルで最も多い終わり方）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddedRowShape {
    columns: usize,
    line_ending: LineEnding,
}

impl AddedRowShape {
    /// セルの数 `columns`（少なくとも 1）と、終わり方 `line_ending`（終わり方が無い印なら CRLF）。
    #[must_use]
    pub fn new(columns: usize, line_ending: LineEnding) -> Self {
        let line_ending = match line_ending {
            LineEnding::EndOfInput => LineEnding::CrLf,
            LineEnding::CrLf | LineEnding::Lf | LineEnding::Cr => line_ending,
        };
        Self {
            columns: columns.max(1),
            line_ending,
        }
    }

    /// セルの数。
    #[must_use]
    pub const fn columns(&self) -> usize {
        self.columns
    }

    /// 終わり方（`EndOfInput` にはならない）。
    #[must_use]
    pub const fn line_ending(&self) -> LineEnding {
        self.line_ending
    }
}
