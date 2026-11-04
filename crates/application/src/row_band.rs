//! 選んだ範囲を行で切った帯。

use alloc::vec::Vec;
use core::ops::RangeInclusive;

/// 選んだ範囲を行で切った帯。帯の中のどの行でも、選んだ列（重なりを除いて並べた列の範囲）が同じ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowBand {
    rows: RangeInclusive<u64>,
    columns: Vec<RangeInclusive<usize>>,
}

impl RowBand {
    /// 本体の行 `rows` で、列 `columns`（左から・重ならない）。
    #[must_use]
    pub fn new(rows: RangeInclusive<u64>, columns: Vec<RangeInclusive<usize>>) -> Self {
        Self { rows, columns }
    }

    /// 本体の行。
    #[must_use]
    pub fn rows(&self) -> RangeInclusive<u64> {
        self.rows.clone()
    }

    /// 列（左から・重ならない）。
    #[must_use]
    pub fn columns(&self) -> &[RangeInclusive<usize>] {
        &self.columns
    }
}
