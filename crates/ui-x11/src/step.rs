//! 1 つのイベントの後にすること。

/// 1 つのイベントの後にすること。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Step {
    /// そのまま次を待つ。
    Wait,
    /// 描き直す。
    Redraw,
    /// 閉じる。
    Quit,
}
