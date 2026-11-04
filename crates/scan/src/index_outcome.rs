//! 索引を作る処理の終わり方。

use crate::sparse_index::SparseIndex;

/// 索引を作る処理の終わり方。中止したときの途中の結果は捨てる（ADR 0014 の決定 4）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexOutcome {
    /// 終わりまで読んだ。
    Finished(SparseIndex),
    /// 中止を求められて止まった。
    Cancelled,
}
