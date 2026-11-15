//! 行の足し消しの知らせ（ADR 0027）。

/// 行の足し消しの知らせ。数は行の数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowsNotice {
    /// 行を足した。
    Inserted(u64),
    /// 行を複製した（すぐ下に同じ値の行を足した・GitHub Issue #486）。
    Duplicated(u64),
    /// 複製する行を読めなかったので、足さなかった（行と列の複製・GitHub Issue #486・#488）。
    Unread,
    /// 行が多すぎるので複製しなかった（行の複製は選んだ行・列の複製は表の全部の行が、コピーと同じ上限まで）。
    TooMany,
    /// 行を消した（Ctrl+Z で戻せる）。
    Removed(u64),
    /// 行の足し消しを戻した。
    Undone(u64),
    /// 行の足し消しをやり直した。
    Redone(u64),
    /// 表の末尾は、索引を作り終えて行の数が分かってから（末尾に足す・終わりまでの範囲を消す）。
    AwaitIndex,
}
