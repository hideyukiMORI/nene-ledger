//! スクロールバーの描画命令（Issue #44 のデザインの相談）。

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use nene_ledger_core::color::Color;
use nene_ledger_core::display_width::display_columns;
use nene_ledger_core::draw_command::DrawCommand;
use nene_ledger_core::glyph_metrics::GlyphMetrics;
use nene_ledger_core::palette::Palette;
use nene_ledger_core::pixel_point::PixelPoint;
use nene_ledger_core::pixel_rect::PixelRect;
use nene_ledger_core::pixel_size::PixelSize;
use nene_ledger_core::text_role::TextRole;

use crate::bar_marks::BarMarks;
use crate::scroll_bar::ScrollBar;
use crate::scroll_orientation::ScrollOrientation;
use crate::scrollbars::Scrollbars;
use crate::thumb_state::ThumbState;

/// 選んだ範囲の棒の太さ（溝の表側の端）。
const RANGE_THICKNESS: u32 = 3;
/// 選んだ範囲の棒の最小の長さ。
const RANGE_MIN: u32 = 4;
/// カーソルの線の太さ。
const CURSOR_THICKNESS: u32 = 2;
/// 札とつまみの間。
const TIP_GAP: u32 = 4;

/// スクロールバーの描画命令を作る。溝は面の色、表との間に罫線、つまみ、選んだ範囲の棒（強調の色）、カーソルの線（字の色・
/// つまみに重なる所は背景の色）の順に重ねる。
#[derive(Debug, Clone, Copy)]
pub struct ScrollbarPainter {
    bars: Scrollbars,
    palette: Palette,
}

impl ScrollbarPainter {
    /// 置き場所 `bars` を配色 `palette` で描く。
    #[must_use]
    pub fn new(bars: Scrollbars, palette: Palette) -> Self {
        Self { bars, palette }
    }

    /// 固定の画素 `pixels` に画面の倍率を掛ける（ADR 0020 の 5）。
    fn px(self, pixels: u32) -> u32 {
        self.bars.fixed().scale().px(pixels)
    }

    /// 角を面の色で埋め、表と溝の間の罫線を角まで引く。
    #[must_use]
    pub fn frame(self) -> Vec<DrawCommand> {
        let mut commands: Vec<DrawCommand> = self
            .bars
            .corners()
            .into_iter()
            .map(|rect| fill(rect, self.palette.gutter()))
            .collect();
        let area = self.bars.area();
        let body = self.bars.body();
        let (left, top) = (area.origin().x(), area.origin().y());
        if self.bars.bar(ScrollOrientation::Vertical).is_some() {
            let x = left.saturating_add(body.size().width());
            commands.push(fill(
                PixelRect::new(
                    PixelPoint::new(x, top),
                    PixelSize::new(self.px(1), area.size().height()),
                ),
                self.palette.grid(),
            ));
            let header = self.bars.fixed().header();
            if header > 0 {
                // 見出しの行の下の罫線を角まで伸ばす。
                commands.push(fill(
                    PixelRect::new(
                        PixelPoint::new(x, top.saturating_add(header).saturating_sub(self.px(1))),
                        PixelSize::new(ScrollBar::space(self.bars.fixed().scale()), self.px(1)),
                    ),
                    self.palette.grid(),
                ));
            }
        }
        if self.bars.bar(ScrollOrientation::Horizontal).is_some() {
            let y = top.saturating_add(body.size().height());
            commands.push(fill(
                PixelRect::new(
                    PixelPoint::new(left, y),
                    PixelSize::new(area.size().width(), self.px(1)),
                ),
                self.palette.grid(),
            ));
        }
        commands
    }

    /// 1 本のバー `scrollbar` を、つまみの状態 `state` と印 `marks` で描く。
    #[must_use]
    pub fn bar(
        self,
        scrollbar: ScrollBar,
        state: ThumbState,
        marks: &BarMarks,
    ) -> Vec<DrawCommand> {
        let (wide, thumb_color) = match state {
            ThumbState::Normal => (false, self.palette.scroll_thumb()),
            ThumbState::Hovered | ThumbState::Held => (true, self.palette.gutter_text()),
        };
        let axis = scrollbar.axis();
        let length = scrollbar.length();
        let thumb = scrollbar.thumb_rect(wide);
        let (cursor_at, cursor_length) = axis.mark(
            length,
            marks.cursor()..marks.cursor().saturating_add(1),
            self.px(CURSOR_THICKNESS),
        );
        let cursor_length = cursor_length.min(self.px(CURSOR_THICKNESS));
        let cursor = scrollbar.span(
            (cursor_at, cursor_length),
            (0, ScrollBar::track_width(self.bars.fixed().scale())),
        );
        let mut commands = vec![
            fill(scrollbar.track(), self.palette.gutter()),
            fill(thumb, thumb_color),
        ];
        commands.extend(marks.ranges().iter().map(|&(from, to)| {
            let (at, range_length) = axis.mark(length, from..to, self.px(RANGE_MIN));
            fill(
                scrollbar.span((at, range_length), (0, self.px(RANGE_THICKNESS))),
                self.palette.cursor(),
            )
        }));
        commands.push(fill(cursor, self.palette.text()));
        commands.extend(overlap(cursor, thumb).map(|rect| fill(rect, self.palette.background())));
        commands
    }

    /// 縦のつまみのドラッグ中に、つまみの左に札 `text`（「1,234,567 行目」）を描く。字の寸法は窓の部品の `metrics`。
    #[must_use]
    pub fn tip(
        self,
        scrollbar: ScrollBar,
        text: String,
        metrics: GlyphMetrics,
    ) -> Vec<DrawCommand> {
        let width = display_columns(&text)
            .saturating_mul(metrics.half_width())
            .saturating_add(metrics.half_width().saturating_mul(2));
        let height = metrics.line_height().saturating_add(self.px(8));
        let thumb = scrollbar.thumb_rect(true);
        let center = thumb.origin().y().saturating_add(thumb.size().height() / 2);
        let area = self.bars.area();
        let top = center
            .saturating_sub(height / 2)
            .max(area.origin().y())
            .min(
                area.origin()
                    .y()
                    .saturating_add(area.size().height())
                    .saturating_sub(height),
            );
        let right = scrollbar
            .track()
            .origin()
            .x()
            .saturating_sub(self.px(TIP_GAP));
        let left = right.saturating_sub(width).max(area.origin().x());
        let rect = PixelRect::new(
            PixelPoint::new(left, top),
            PixelSize::new(right.saturating_sub(left), height),
        );
        let mut commands = vec![fill(rect, self.palette.gutter())];
        commands.extend(edges(rect, self.px(1)).map(|edge| fill(edge, self.palette.border())));
        commands.push(DrawCommand::Text {
            baseline: PixelPoint::new(
                left.saturating_add(metrics.half_width()),
                top.saturating_add(self.px(4))
                    .saturating_add(metrics.ascent()),
            ),
            text,
            color: self.palette.text(),
            clip: rect,
            role: TextRole::Chrome,
        });
        commands
    }
}

/// 矩形 `rect` の内側の 4 辺（太さ `line`）。上・下・左・右。
fn edges(rect: PixelRect, line: u32) -> [PixelRect; 4] {
    let (x, y) = (rect.origin().x(), rect.origin().y());
    let (width, height) = (rect.size().width(), rect.size().height());
    let line = line.min(width).min(height);
    [
        PixelRect::new(PixelPoint::new(x, y), PixelSize::new(width, line)),
        PixelRect::new(
            PixelPoint::new(x, y.saturating_add(height).saturating_sub(line)),
            PixelSize::new(width, line),
        ),
        PixelRect::new(PixelPoint::new(x, y), PixelSize::new(line, height)),
        PixelRect::new(
            PixelPoint::new(x.saturating_add(width).saturating_sub(line), y),
            PixelSize::new(line, height),
        ),
    ]
}

/// `rect` を `color` で塗る命令。
fn fill(rect: PixelRect, color: Color) -> DrawCommand {
    DrawCommand::Fill { rect, color }
}

/// 2 つの矩形の重なり（無ければ `None`）。
fn overlap(first: PixelRect, second: PixelRect) -> Option<PixelRect> {
    let left = first.origin().x().max(second.origin().x());
    let top = first.origin().y().max(second.origin().y());
    let right = (first.origin().x().saturating_add(first.size().width()))
        .min(second.origin().x().saturating_add(second.size().width()));
    let bottom = (first.origin().y().saturating_add(first.size().height()))
        .min(second.origin().y().saturating_add(second.size().height()));
    (right > left && bottom > top).then(|| {
        PixelRect::new(
            PixelPoint::new(left, top),
            PixelSize::new(right - left, bottom - top),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed_parts::FixedParts;
    use crate::scroll_bar::TRACK_WIDTH;
    use nene_ledger_core::scroll_axis::ScrollAxis;
    use nene_ledger_core::theme_catalog::named;
    use nene_ledger_core::theme_derivation::derive;

    fn palette() -> Palette {
        derive(named("neutral-light").unwrap())
    }

    /// 400 × 250 の所に、100 行（9 行目から）と 1,000 画素の列。
    fn bars() -> Scrollbars {
        Scrollbars::fit(
            PixelRect::new(PixelPoint::new(0, 0), PixelSize::new(400, 250)),
            FixedParts::new(25, 25, 40),
            ScrollAxis::new(100, 0, 8),
            ScrollAxis::new(1_000, 0, 0),
        )
    }

    fn fills(commands: &[DrawCommand], color: Color) -> Vec<PixelRect> {
        commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Fill {
                    rect,
                    color: filled,
                } if *filled == color => Some(*rect),
                DrawCommand::Fill { .. } | DrawCommand::Text { .. } | DrawCommand::Line { .. } => {
                    None
                }
            })
            .collect()
    }

    #[test]
    fn the_thumb_is_thin_normally_and_wide_with_another_color_when_hovered() {
        let painter = ScrollbarPainter::new(bars(), palette());
        let scrollbar = bars().bar(ScrollOrientation::Vertical).unwrap();
        let marks = BarMarks::new(vec![(8, 9)], 8);
        let normal = painter.bar(scrollbar, ThumbState::Normal, &marks);
        let thumb = fills(&normal, palette().scroll_thumb());
        assert_eq!(thumb.len(), 1);
        assert_eq!(thumb[0].size().width(), 8);
        let hovered = painter.bar(scrollbar, ThumbState::Hovered, &marks);
        assert_eq!(
            fills(&hovered, palette().gutter_text())[0].size().width(),
            12
        );
        assert!(fills(&hovered, palette().scroll_thumb()).is_empty());
    }

    #[test]
    fn the_range_and_the_cursor_are_marked_on_the_track() {
        let painter = ScrollbarPainter::new(bars(), palette());
        let scrollbar = bars().bar(ScrollOrientation::Vertical).unwrap();
        // 範囲は 50 行目から 60 行目まで。カーソルは 90 行目（つまみの外）。
        let commands = painter.bar(
            scrollbar,
            ThumbState::Normal,
            &BarMarks::new(vec![(50, 60)], 90),
        );
        let range = fills(&commands, palette().cursor());
        assert_eq!(range.len(), 1);
        assert_eq!(range[0].origin().x(), scrollbar.track().origin().x());
        assert_eq!(range[0].size().width(), RANGE_THICKNESS);
        let cursor = fills(&commands, palette().text());
        assert_eq!(
            cursor[0].size(),
            PixelSize::new(TRACK_WIDTH, CURSOR_THICKNESS)
        );
        // つまみの外なので背景の色の重なりは無い。カーソルがつまみの中なら重なりを背景の色で描く。
        assert!(fills(&commands, palette().background()).is_empty());
        let inside = painter.bar(
            scrollbar,
            ThumbState::Normal,
            &BarMarks::new(vec![(8, 9)], 9),
        );
        assert_eq!(fills(&inside, palette().background()).len(), 1);
    }

    #[test]
    fn corners_and_rules_frame_the_bars() {
        let commands = ScrollbarPainter::new(bars(), palette()).frame();
        // 角 3 つは面の色。罫線は縦・見出しの下の伸ばし・横の 3 本。
        assert_eq!(fills(&commands, palette().gutter()).len(), 3);
        assert_eq!(fills(&commands, palette().grid()).len(), 3);
    }

    #[test]
    fn the_tip_sits_left_of_the_thumb_inside_the_area() {
        let scrollbar = bars().bar(ScrollOrientation::Vertical).unwrap();
        let metrics = GlyphMetrics::new(8, 16, 12);
        let commands = ScrollbarPainter::new(bars(), palette()).tip(
            scrollbar,
            String::from("9 行目"),
            metrics,
        );
        let Some(DrawCommand::Text { clip, text, .. }) = commands.last() else {
            panic!("札の字");
        };
        assert_eq!(text, "9 行目");
        assert_eq!(
            clip.origin().x() + clip.size().width(),
            scrollbar.track().origin().x() - TIP_GAP
        );
        assert_eq!(fills(&commands, palette().border()).len(), 4);
    }
}
