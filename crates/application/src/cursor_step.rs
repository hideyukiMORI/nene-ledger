//! カーソルの動かし方。

/// カーソルの動かし方（キー）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStep {
    /// 1 行上へ。
    Up,
    /// 1 行下へ。
    Down,
    /// 1 列左へ。
    Left,
    /// 1 列右へ。
    Right,
    /// 1 画面上へ。
    PageUp,
    /// 1 画面下へ。
    PageDown,
    /// 行の先頭の列へ（Home）。
    RowStart,
    /// 行の末尾の列へ（End）。
    RowEnd,
    /// 表の先頭の行の先頭の列へ（Ctrl+Home）。
    Top,
    /// 表の末尾の行の末尾の列へ（Ctrl+End・行の数が分かってから）。
    Bottom,
}
