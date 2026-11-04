//! 窓の縁の向き。

/// 窓の縁の向き。ここを押すとその向きにリサイズする（ADR 0013）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResizeEdge {
    /// 左上の角。
    TopLeft,
    /// 上の辺。
    Top,
    /// 右上の角。
    TopRight,
    /// 右の辺。
    Right,
    /// 右下の角。
    BottomRight,
    /// 下の辺。
    Bottom,
    /// 左下の角。
    BottomLeft,
    /// 左の辺。
    Left,
}
