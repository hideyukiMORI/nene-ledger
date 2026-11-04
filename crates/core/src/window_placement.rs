//! 窓の置かれ方。

/// 窓の置かれ方。最大化している間は縁でリサイズせず、縁の線も引かない。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowPlacement {
    /// 通常。
    Normal,
    /// 最大化。
    Maximized,
}
