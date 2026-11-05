//! 元に戻す・やり直すの 1 手（ADR 0024 の決定 5）。

use alloc::vec::Vec;

use nene_ledger_core::cell_change::CellChange;

use crate::column_change::ColumnChange;
use crate::row_change::RowChange;

/// 元に戻す・やり直すの 1 手。セルの書き換えの束か、行の足し消し。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditStep {
    /// セルの書き換えの束（今は確定 1 回で 1 つのセル）。
    Cells(Vec<CellChange>),
    /// 行の足し消し（ADR 0027）。
    Rows(RowChange),
    /// 列の足し消し（ADR 0027）。
    Columns(ColumnChange),
}
