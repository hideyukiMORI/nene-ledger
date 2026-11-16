//! 書き出しの形（方言を変えた書き出し・ADR 0036 の決定 1）。

use nene_ledger_core::dialect::Dialect;
use nene_ledger_core::line_ending::LineEnding;
use nene_ledger_core::quote_policy::QuotePolicy;

/// 書き出す先の方言・引用符の付け方・行の終わり方。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportShape {
    dialect: Dialect,
    policy: QuotePolicy,
    line_ending: LineEnding,
}

impl ExportShape {
    /// 方言 `dialect`・引用符の付け方 `policy`・行の終わり方 `line_ending`（終わり方が無い印なら CRLF）。
    #[must_use]
    pub const fn new(dialect: Dialect, policy: QuotePolicy, line_ending: LineEnding) -> Self {
        let line_ending = match line_ending {
            LineEnding::EndOfInput => LineEnding::CrLf,
            LineEnding::CrLf | LineEnding::Lf | LineEnding::Cr => line_ending,
        };
        Self {
            dialect,
            policy,
            line_ending,
        }
    }

    /// 方言。
    #[must_use]
    pub const fn dialect(&self) -> &Dialect {
        &self.dialect
    }

    /// 引用符の付け方。
    #[must_use]
    pub const fn policy(&self) -> QuotePolicy {
        self.policy
    }

    /// 行の終わり方（`EndOfInput` にはならない）。
    #[must_use]
    pub const fn line_ending(&self) -> LineEnding {
        self.line_ending
    }

    /// 行の終わり方の文字列。
    #[must_use]
    pub const fn line_break(&self) -> &'static str {
        match self.line_ending {
            LineEnding::Lf => "\n",
            LineEnding::Cr => "\r",
            LineEnding::CrLf | LineEnding::EndOfInput => "\r\n",
        }
    }
}
