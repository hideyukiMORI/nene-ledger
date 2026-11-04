//! 配色を替える向き。

/// 配色を表の中で替える向き。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeStep {
    /// 次の配色へ。
    Next,
    /// 前の配色へ。
    Previous,
}
