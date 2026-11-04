//! 選んだ範囲をコピーするときの形。

use alloc::vec::Vec;
use core::ops::RangeInclusive;

/// 選んだ範囲をコピーするときの形。行（上から）と列（左から）の並びで、その積のセルを 1 つの表として詰めて並べる。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyShape {
    rows: Vec<RangeInclusive<u64>>,
    columns: Vec<RangeInclusive<usize>>,
}

impl CopyShape {
    /// 本体の行 `rows` と列 `columns`（どちらも並べる順・重ならない）。
    #[must_use]
    pub fn new(rows: Vec<RangeInclusive<u64>>, columns: Vec<RangeInclusive<usize>>) -> Self {
        Self { rows, columns }
    }

    /// 本体の行。
    #[must_use]
    pub fn rows(&self) -> &[RangeInclusive<u64>] {
        &self.rows
    }

    /// 列。
    #[must_use]
    pub fn columns(&self) -> &[RangeInclusive<usize>] {
        &self.columns
    }

    /// 本体の行 `body` が入るか。
    #[must_use]
    pub fn has_row(&self, body: u64) -> bool {
        self.rows.iter().any(|rows| rows.contains(&body))
    }

    /// 行の数。
    #[must_use]
    pub fn row_count(&self) -> u64 {
        self.rows
            .iter()
            .map(|rows| rows.end().saturating_sub(*rows.start()).saturating_add(1))
            .sum()
    }

    /// 列の数。
    #[must_use]
    pub fn column_count(&self) -> usize {
        self.columns
            .iter()
            .map(|columns| {
                columns
                    .end()
                    .saturating_sub(*columns.start())
                    .saturating_add(1)
            })
            .sum()
    }
}
