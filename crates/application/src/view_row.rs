//! 見えている 1 行。

use alloc::string::String;
use alloc::vec::Vec;

use nene_ledger_core::row_id::RowId;

use crate::cell_display::single_line;

/// 見えている 1 行。行の並びの位置（0 始まり・前書きと見出しも数える・ADR 0027 の決定 1）と、行の同一性と、
/// 表示の文字列（1 行）にしたセルと、元の値（復号したまま）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewRow {
    line: u64,
    row: RowId,
    cells: Vec<String>,
    values: Vec<String>,
}

impl ViewRow {
    /// 行を作る。`cells` は復号したセルの値で、表示の文字列（1 行）にしたものと、元の値の両方を持つ（詳細ペイン・VIEW-07）。
    /// 行の同一性は、行を足し消ししていないときの原本のレコード `Source(line)`（[`Self::with_row`] で替える）。
    #[must_use]
    pub fn new(line: u64, cells: &[String]) -> Self {
        Self {
            line,
            row: RowId::Source(line),
            cells: cells.iter().map(|cell| single_line(cell)).collect(),
            values: cells.to_vec(),
        }
    }

    /// 行の同一性を `row` にした行。
    #[must_use]
    pub fn with_row(self, row: RowId) -> Self {
        Self { row, ..self }
    }

    /// 行の並びの位置（0 始まり）。行を足し消ししていなければ、ファイルの中のレコードの番号と同じ。
    #[must_use]
    pub fn line(&self) -> u64 {
        self.line
    }

    /// 行の同一性（書き換えの差分を引く鍵・ADR 0024 の決定 1）。
    #[must_use]
    pub fn row(&self) -> RowId {
        self.row
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
