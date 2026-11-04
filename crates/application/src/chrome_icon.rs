//! タイトルバーのボタンの印。

use alloc::vec::Vec;

use nene_ledger_core::color::Color;
use nene_ledger_core::draw_command::DrawCommand;
use nene_ledger_core::glyph_metrics::GlyphMetrics;
use nene_ledger_core::pixel_point::PixelPoint;
use nene_ledger_core::pixel_rect::PixelRect;

/// 印の線の太さ（画素・100% のとき）。
const ICON_LINE: u32 = 1;
/// 元に戻すの印で、後ろの四角をずらす量（画素・100% のとき）。
const RESTORE_SHIFT: u32 = 2;

/// ボタンの真ん中の、1 辺が 1 行の高さの 5/8 の印。枠は左上と右下の画素、`ink` は線の色。
/// 線の太さとずらす量は画面の倍率を掛けたもの。
#[derive(Debug, Clone, Copy)]
pub(crate) struct ChromeIcon {
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
    width: u32,
    shift: u32,
    ink: Color,
}

impl ChromeIcon {
    pub(crate) fn centered(rect: PixelRect, metrics: GlyphMetrics, ink: Color) -> Self {
        let side = metrics.line_height().saturating_mul(5) / 8;
        let left = rect
            .origin()
            .x()
            .saturating_add(rect.size().width().saturating_sub(side) / 2);
        let top = rect
            .origin()
            .y()
            .saturating_add(rect.size().height().saturating_sub(side) / 2);
        let last = side.saturating_sub(1);
        Self {
            left,
            top,
            right: left.saturating_add(last),
            bottom: top.saturating_add(last),
            width: metrics.px(ICON_LINE),
            shift: metrics.px(RESTORE_SHIFT),
            ink,
        }
    }

    /// 真ん中の横線。
    pub(crate) fn minimize(self) -> Vec<DrawCommand> {
        let middle = self.top.midpoint(self.bottom);
        Vec::from([self.line(
            PixelPoint::new(self.left, middle),
            PixelPoint::new(self.right, middle),
        )])
    }

    /// 枠いっぱいの四角の 4 辺。
    pub(crate) fn maximize(self) -> Vec<DrawCommand> {
        let (left, top, right, bottom) = (self.left, self.top, self.right, self.bottom);
        Vec::from([
            self.line(PixelPoint::new(left, top), PixelPoint::new(right, top)),
            self.line(PixelPoint::new(right, top), PixelPoint::new(right, bottom)),
            self.line(
                PixelPoint::new(left, bottom),
                PixelPoint::new(right, bottom),
            ),
            self.line(PixelPoint::new(left, top), PixelPoint::new(left, bottom)),
        ])
    }

    /// 左下にずらした四角と、その右上に覗く後ろの四角の 2 辺。
    pub(crate) fn restore(self) -> Vec<DrawCommand> {
        let front = Self {
            top: self.top.saturating_add(self.shift),
            right: self.right.saturating_sub(self.shift),
            ..self
        };
        let back_left = self.left.saturating_add(self.shift);
        let back_bottom = self.bottom.saturating_sub(self.shift);
        let mut lines = front.maximize();
        lines.push(self.line(
            PixelPoint::new(back_left, self.top),
            PixelPoint::new(self.right, self.top),
        ));
        lines.push(self.line(
            PixelPoint::new(self.right, self.top),
            PixelPoint::new(self.right, back_bottom),
        ));
        lines
    }

    /// 斜めの 2 本。
    pub(crate) fn close(self) -> Vec<DrawCommand> {
        Vec::from([
            self.line(
                PixelPoint::new(self.left, self.top),
                PixelPoint::new(self.right, self.bottom),
            ),
            self.line(
                PixelPoint::new(self.right, self.top),
                PixelPoint::new(self.left, self.bottom),
            ),
        ])
    }

    /// 画素 `from` から `to` までの線。
    fn line(self, from: PixelPoint, to: PixelPoint) -> DrawCommand {
        DrawCommand::Line {
            from,
            to,
            width: self.width,
            color: self.ink,
        }
    }
}
