//! 列の同一性（ADR 0024 の決定 1）。

/// 列の同一性。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ColumnId {
    /// 原本の列（原本の行の中のセルの番号）。
    Source(usize),
    /// 開いている間に足した列（通し番号）。
    Added(u64),
}
