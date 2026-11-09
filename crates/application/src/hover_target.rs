//! 指が乗っている、見た目の変わる所（GitHub Issue #448）。

use crate::scroll_orientation::ScrollOrientation;

/// 指が乗っている、見た目の変わる所。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoverTarget {
    /// スクロールバーのつまみ。
    Thumb(ScrollOrientation),
    /// タイトルバーの「≡ 操作」。
    Menu,
}
