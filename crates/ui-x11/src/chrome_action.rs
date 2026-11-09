//! 窓の飾りの操作が求めること。

use nene_ledger_core::resize_edge::ResizeEdge;

use crate::cursor_shape::CursorShape;

/// 窓の飾りの操作が求めること。移動・リサイズ・最大化・最小化はウィンドウマネージャーに頼む（ADR 0013 の決定 2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChromeAction {
    /// 何もしない。
    Nothing,
    /// 描き直す（指の乗ったボタンが変わった）。
    Redraw,
    /// 窓を動かす（`_NET_WM_MOVERESIZE` の移動）。
    Move,
    /// その向きにリサイズする（`_NET_WM_MOVERESIZE`）。
    Resize(ResizeEdge),
    /// 最大化と元に戻すを切り替える（`_NET_WM_STATE`）。
    ToggleMaximize,
    /// 最小化する（`WM_CHANGE_STATE`）。
    Minimize,
    /// 閉じる。
    Close,
}

/// `_NET_WM_MOVERESIZE` の向きの番号（EWMH）。
pub(crate) fn moveresize_direction(edge: ResizeEdge) -> u32 {
    match edge {
        ResizeEdge::TopLeft => 0,
        ResizeEdge::Top => 1,
        ResizeEdge::TopRight => 2,
        ResizeEdge::Right => 3,
        ResizeEdge::BottomRight => 4,
        ResizeEdge::Bottom => 5,
        ResizeEdge::BottomLeft => 6,
        ResizeEdge::Left => 7,
    }
}

/// `_NET_WM_MOVERESIZE` の移動の番号（EWMH）。
pub(crate) const MOVERESIZE_MOVE: u32 = 8;

/// カーソルの字（X の `cursorfont.h`）。矢印・縁の 8 方向・列の幅（`sb_h_double_arrow`）。
pub(crate) fn cursor_glyph(shape: CursorShape) -> u16 {
    match shape {
        CursorShape::Arrow => 68,
        CursorShape::Edge(ResizeEdge::TopLeft) => 134,
        CursorShape::Edge(ResizeEdge::Top) => 138,
        CursorShape::Edge(ResizeEdge::TopRight) => 136,
        CursorShape::Edge(ResizeEdge::Right) => 96,
        CursorShape::Edge(ResizeEdge::BottomRight) => 14,
        CursorShape::Edge(ResizeEdge::Bottom) => 16,
        CursorShape::Edge(ResizeEdge::BottomLeft) => 12,
        CursorShape::Edge(ResizeEdge::Left) => 70,
        CursorShape::ColumnResize => 108,
        // sb_v_double_arrow（GitHub Issue #452）。
        CursorShape::RowResize => 116,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edges_map_to_ewmh_directions_and_cursor_glyphs() {
        assert_eq!(moveresize_direction(ResizeEdge::TopLeft), 0);
        assert_eq!(moveresize_direction(ResizeEdge::Bottom), 5);
        assert_eq!(moveresize_direction(ResizeEdge::Left), 7);
        assert_eq!(cursor_glyph(CursorShape::Arrow), 68);
        assert_eq!(cursor_glyph(CursorShape::Edge(ResizeEdge::BottomRight)), 14);
        assert_eq!(cursor_glyph(CursorShape::Edge(ResizeEdge::Right)), 96);
        assert_eq!(cursor_glyph(CursorShape::ColumnResize), 108);
        assert_eq!(cursor_glyph(CursorShape::RowResize), 116);
    }
}
