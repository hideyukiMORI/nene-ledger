//! 一部の列だけを持つ窓の行の、持っている列（ADR 0033 の決定 1・GitHub Issue #22）。

use alloc::vec::Vec;
use core::ops::Range;

/// 一部の列だけを持つ行の、行の列の数（原本のセルの数）と、持っている列の範囲（左から・重ならない・列の数の中）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct HeldColumns {
    total: usize,
    spans: Vec<Range<usize>>,
}

impl HeldColumns {
    /// 行の列の数 `total` と、持っている列の範囲 `spans`（左から・重ならない・列の数の中）。
    pub(crate) const fn new(total: usize, spans: Vec<Range<usize>>) -> Self {
        Self { total, spans }
    }

    /// 行の列の数。
    pub(crate) const fn total(&self) -> usize {
        self.total
    }

    /// 持っている列の範囲。
    pub(crate) fn spans(&self) -> &[Range<usize>] {
        &self.spans
    }
}
