//! 行の足し消しの 1 手（ADR 0024 の決定 5・ADR 0027 の決定 2）。

use alloc::vec::Vec;

use nene_ledger_core::order_run::OrderRun;
use nene_ledger_core::row_order::RowOrder;

/// 行の足し消しの 1 手。位置 `at` に足した（消した）区間 `runs`。戻すときは逆の操作をする。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowChange {
    /// 位置 `at` に `runs`（足した行）を足した。
    Inserted {
        /// 足した位置。
        at: u64,
        /// 足した区間。
        runs: Vec<OrderRun>,
    },
    /// 位置 `at` から `runs` を消した。
    Removed {
        /// 消した位置。
        at: u64,
        /// 消した区間。
        runs: Vec<OrderRun>,
    },
}

impl RowChange {
    /// 足した（消した）位置。
    #[must_use]
    pub fn at(&self) -> u64 {
        match self {
            Self::Inserted { at, .. } | Self::Removed { at, .. } => *at,
        }
    }

    /// 足した（消した）行の数。
    #[must_use]
    pub fn count(&self) -> u64 {
        let (Self::Inserted { runs, .. } | Self::Removed { runs, .. }) = self;
        runs.iter()
            .fold(0, |sum: u64, run| sum.saturating_add(run.len(0)))
    }

    /// 並び `order` に、この手をもう一度行う（やり直す）。
    pub fn redo(&self, order: &mut RowOrder) {
        match self {
            Self::Inserted { at, runs } => order.restore(*at, runs.clone()),
            Self::Removed { at, .. } => drop(order.remove(*at, self.count())),
        }
    }

    /// 並び `order` から、この手を戻す。
    pub fn undo(&self, order: &mut RowOrder) {
        match self {
            Self::Inserted { at, .. } => drop(order.remove(*at, self.count())),
            Self::Removed { at, runs } => order.restore(*at, runs.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nene_ledger_core::row_id::RowId;

    #[test]
    fn undoing_and_redoing_returns_the_order_to_each_state() {
        let mut order = RowOrder::whole();
        let added = order.insert(2, 2);
        let inserted = RowChange::Inserted {
            at: 2,
            runs: order.slice(2, 2),
        };
        let removed = RowChange::Removed {
            at: 5,
            runs: order.remove(5, 3),
        };
        assert_eq!((inserted.count(), removed.count()), (2, 3));
        let after = order.clone();
        removed.undo(&mut order);
        inserted.undo(&mut order);
        assert!(order.is_whole());
        inserted.redo(&mut order);
        assert_eq!(order.row_at(3), added.get(1).copied());
        removed.redo(&mut order);
        assert_eq!(order, after);
        assert_eq!(order.row_at(5), Some(RowId::Source(6)));
    }
}
