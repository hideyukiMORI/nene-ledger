//! 列の足し消しの知らせ（ADR 0027）。

/// 列の足し消しの知らせ。数は列の数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnsNotice {
    /// 列を足した。
    Inserted(usize),
    /// 列を消した（Ctrl+Z で戻せる）。
    Removed(usize),
    /// 列の足し消しを戻した。
    Undone(usize),
    /// 列の足し消しをやり直した。
    Redone(usize),
    /// 表示の上で入れ替えた列が離れているので、足し消しできない（列の並びを元に戻してから）。
    Reordered,
    /// すべての列は消せない（1 列は残す）。
    KeepOne,
    /// 列を複製した（すぐ右に同じ値の列を足した・GitHub Issue #488）。
    Duplicated(usize),
    /// 列の複製は表の全部の行を読むので、索引を作り終えて行の数が分かってから。
    AwaitIndex,
}
