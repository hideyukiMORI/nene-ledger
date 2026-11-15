//! 押して始めたドラッグが何を広げるか。

/// 押して始めたドラッグが何を広げるか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressKind {
    /// セルの矩形。
    Cells,
    /// 行全体（行番号を押した）。
    Rows,
    /// 列全体（見出しを押した）。
    Columns,
    /// 選んでいる列を移す（選んだ列の見出しを押した・Issue #49）。
    MoveColumns,
    /// 選んでいる行を移す（選んだ行の行番号を押した・GitHub Issue #492）。
    MoveRows,
}
