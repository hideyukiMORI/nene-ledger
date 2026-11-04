//! 状態の行に出す、選んだ範囲の集計の状態。

use crate::job_progress::JobProgress;
use crate::selection_stats::SelectionStats;

/// 状態の行に出す、選んだ範囲の集計の状態。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionTotals {
    /// 出さない（1 つのセルだけの選択・読めなかった）。
    Hidden,
    /// 数えた。
    Counted(SelectionStats),
    /// 裏で数えている（進み具合はレコードの数）。
    Counting(JobProgress),
    /// 裏の計算を止めた。
    Stopped,
    /// 裏の計算が読めずに終わった。
    Failed,
}
