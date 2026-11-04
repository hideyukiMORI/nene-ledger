//! 見えている 1 行。

use alloc::string::String;
use alloc::vec::Vec;

use crate::cell_display::single_line;

/// 見えている 1 行。ファイルの中のレコードの番号（0 始まり）と、表示の文字列（1 行）にしたセルと、元の値（復号したまま）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewRow {
    record: u64,
    cells: Vec<String>,
    values: Vec<String>,
}

impl ViewRow {
    /// 行を作る。`cells` は復号したセルの値で、表示の文字列（1 行）にしたものと、元の値の両方を持つ（詳細ペイン・VIEW-07）。
    #[must_use]
    pub fn new(record: u64, cells: &[String]) -> Self {
        Self {
            record,
            cells: cells.iter().map(|cell| single_line(cell)).collect(),
            values: cells.to_vec(),
        }
    }

    /// レコードの番号（0 始まり）。
    #[must_use]
    pub fn record(&self) -> u64 {
        self.record
    }

    /// 表示の文字列。
    #[must_use]
    pub fn cells(&self) -> &[String] {
        &self.cells
    }

    /// 元の値（復号したまま）。
    #[must_use]
    pub fn values(&self) -> &[String] {
        &self.values
    }
}
