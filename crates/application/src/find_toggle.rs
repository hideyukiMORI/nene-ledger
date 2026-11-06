//! 検索の欄で切り替える条件（ADR 0030 の決定 3）。

/// 検索の欄で切り替える条件。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindToggle {
    /// 大文字と小文字を区別するか（Alt+C）。
    Case,
    /// 部分一致かセル全体か（Alt+W）。
    WholeCell,
}
