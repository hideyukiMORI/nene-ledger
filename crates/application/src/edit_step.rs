//! 元に戻す・やり直すの 1 手（ADR 0024 の決定 5）。

use alloc::vec::Vec;

use nene_ledger_core::cell_change::CellChange;

use crate::column_change::ColumnChange;
use crate::row_change::RowChange;
use crate::trim_change::TrimChange;

/// 元に戻す・やり直すの 1 手。セルの書き換えの束か、行の足し消しか、それらを 1 手にまとめた束。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditStep {
    /// セルの書き換えの束（確定 1 回で 1 つのセル・貼り付け 1 回で貼ったセル）。
    Cells(Vec<CellChange>),
    /// 行の足し消し（ADR 0027）。
    Rows(RowChange),
    /// 列の足し消し（ADR 0027）。
    Columns(ColumnChange),
    /// 行の後ろの余計なセルを消した（GitHub Issue #413）。
    Trim(TrimChange),
    /// 複数の手を 1 手にまとめた束（貼り付けで足した列・足した行・書き換え・ADR 0029 の決定 3）。中の順に行い、
    /// 戻すときは逆の順に戻す。
    Batch(Vec<Self>),
}

impl EditStep {
    /// セルの書き換えの手なら、その書き換え（ほかの手は空・束は中を見ない）。
    #[must_use]
    pub fn cell_changes(&self) -> &[CellChange] {
        match self {
            Self::Cells(changes) => changes,
            Self::Rows(_) | Self::Columns(_) | Self::Trim(_) | Self::Batch(_) => &[],
        }
    }

    /// 列の足し消しの手なら、その手。
    #[must_use]
    pub const fn column_change(&self) -> Option<&ColumnChange> {
        match self {
            Self::Columns(change) => Some(change),
            Self::Cells(_) | Self::Rows(_) | Self::Trim(_) | Self::Batch(_) => None,
        }
    }

    /// 行の足し消しの手なら、その手。
    #[must_use]
    pub const fn row_change(&self) -> Option<&RowChange> {
        match self {
            Self::Rows(change) => Some(change),
            Self::Cells(_) | Self::Columns(_) | Self::Trim(_) | Self::Batch(_) => None,
        }
    }
}
