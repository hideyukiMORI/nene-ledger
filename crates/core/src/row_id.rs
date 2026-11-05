//! 行の同一性（ADR 0024 の決定 1）。

/// 行の同一性。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RowId {
    /// 原本のレコード（原本の中のレコードの番号。前書きと見出しも数える）。
    Source(u64),
    /// 開いている間に足した行（通し番号）。
    Added(u64),
}
