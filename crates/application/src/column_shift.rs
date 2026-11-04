//! 列を移す向き。

/// 選んだ列を表示の上で移す向き（Issue #62）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColumnShift {
    /// 1 つ左へ。
    Left,
    /// 1 つ右へ。
    Right,
}
