//! 列の足し消しの 1 手（ADR 0024 の決定 5・ADR 0027 の決定 2）。

use alloc::vec::Vec;

use nene_ledger_core::column_order::ColumnOrder;
use nene_ledger_core::order_run::OrderRun;

/// 列の足し消しの 1 手。位置 `at` に足した（消した）区間 `runs` と、その列の幅 `widths`（戻すときに使う）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnChange {
    /// 位置 `at` に列を足した。
    Inserted {
        /// 足した位置（表の列の順）。
        at: usize,
        /// 足した区間。
        runs: Vec<OrderRun>,
        /// 足した列の幅（半角の字数）。
        widths: Vec<u32>,
    },
    /// 位置 `at` から列を消した。
    Removed {
        /// 消した位置。
        at: usize,
        /// 消した区間。
        runs: Vec<OrderRun>,
        /// 消した列の幅。
        widths: Vec<u32>,
    },
}

impl ColumnChange {
    /// 足した（消した）位置。
    #[must_use]
    pub fn at(&self) -> usize {
        match self {
            Self::Inserted { at, .. } | Self::Removed { at, .. } => *at,
        }
    }

    /// 足した（消した）列の幅（列の数もこの長さ）。
    #[must_use]
    pub fn widths(&self) -> &[u32] {
        match self {
            Self::Inserted { widths, .. } | Self::Removed { widths, .. } => widths,
        }
    }

    /// 足した（消した）列の数。
    #[must_use]
    pub fn count(&self) -> usize {
        self.widths().len()
    }

    /// この手を行うと列が増えるか（足した手をやり直す・消した手を戻す、は `undo` 側で逆になる）。
    #[must_use]
    pub fn adds(&self) -> bool {
        matches!(self, Self::Inserted { .. })
    }

    /// 並び `order` に、この手をもう一度行う。
    pub fn redo(&self, order: &mut ColumnOrder) {
        match self {
            Self::Inserted { at, runs, .. } => order.restore(*at, runs.clone()),
            Self::Removed { at, .. } => drop(order.remove(*at, self.count())),
        }
    }

    /// 並び `order` から、この手を戻す。
    pub fn undo(&self, order: &mut ColumnOrder) {
        match self {
            Self::Inserted { at, .. } => drop(order.remove(*at, self.count())),
            Self::Removed { at, runs, .. } => order.restore(*at, runs.clone()),
        }
    }
}
