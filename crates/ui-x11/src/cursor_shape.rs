//! 窓に出すマウスのカーソル。

use nene_ledger_application::pointer_shape::PointerShape;
use nene_ledger_core::chrome_hit::ChromeHit;
use nene_ledger_core::resize_edge::ResizeEdge;

/// 窓に出すマウスのカーソル。矢印・縁のリサイズの 8 方向・列の幅（Issue #64）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CursorShape {
    /// 矢印。
    Arrow,
    /// 窓の縁のリサイズ。
    Edge(ResizeEdge),
    /// 列の幅を変える（左右の矢印）。
    ColumnResize,
}

impl CursorShape {
    /// すべて（窓を開くときに作るカーソルの並び）。
    pub(crate) const ALL: [Self; 10] = [
        Self::Arrow,
        Self::Edge(ResizeEdge::TopLeft),
        Self::Edge(ResizeEdge::Top),
        Self::Edge(ResizeEdge::TopRight),
        Self::Edge(ResizeEdge::Right),
        Self::Edge(ResizeEdge::BottomRight),
        Self::Edge(ResizeEdge::Bottom),
        Self::Edge(ResizeEdge::BottomLeft),
        Self::Edge(ResizeEdge::Left),
        Self::ColumnResize,
    ];

    /// 飾りの当たり判定 `hit` と、中身が求める形 `shape` のカーソル。縁が先で、中身の形は中身の上でだけ。
    pub(crate) fn of(hit: ChromeHit, shape: PointerShape) -> Self {
        match (hit, shape) {
            (ChromeHit::Edge(edge), _) => Self::Edge(edge),
            (ChromeHit::Client, PointerShape::ColumnResize) => Self::ColumnResize,
            (ChromeHit::Client, PointerShape::Arrow)
            | (ChromeHit::Caption | ChromeHit::Button(_), _) => Self::Arrow,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nene_ledger_core::chrome_button::ChromeButton;

    #[test]
    fn edges_come_first_and_the_content_shape_only_over_the_content() {
        let resize = PointerShape::ColumnResize;
        assert_eq!(
            CursorShape::of(ChromeHit::Edge(ResizeEdge::Left), resize),
            CursorShape::Edge(ResizeEdge::Left)
        );
        assert_eq!(
            CursorShape::of(ChromeHit::Client, resize),
            CursorShape::ColumnResize
        );
        assert_eq!(
            CursorShape::of(ChromeHit::Client, PointerShape::Arrow),
            CursorShape::Arrow
        );
        assert_eq!(
            CursorShape::of(ChromeHit::Caption, resize),
            CursorShape::Arrow
        );
        assert_eq!(
            CursorShape::of(ChromeHit::Button(ChromeButton::Close), resize),
            CursorShape::Arrow
        );
    }
}
