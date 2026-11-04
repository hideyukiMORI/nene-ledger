//! 選択を畳むか広げるか。

/// カーソルを動かしたときに、選択を畳むか広げるか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    /// 選択をカーソルの 1 つのセルに畳む。
    Collapse,
    /// 起点はそのままにして、選択をカーソルまで広げる（Shift）。
    Extend,
    /// 今の範囲を残して、カーソルの 1 つのセルから新しい範囲を足す（Ctrl+クリック・Issue #46）。
    Add,
}
