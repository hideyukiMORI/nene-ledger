//! 表の所の中の、本体とスクロールバーの置き場所。

use alloc::vec::Vec;

use nene_ledger_core::pixel_point::PixelPoint;
use nene_ledger_core::pixel_rect::PixelRect;
use nene_ledger_core::pixel_size::PixelSize;
use nene_ledger_core::scroll_axis::ScrollAxis;

use crate::bar_hit::BarHit;
use crate::fixed_parts::FixedParts;
use crate::scroll_bar::ScrollBar;
use crate::scroll_orientation::ScrollOrientation;

/// 表の所の中の、本体（見出し・行番号・セル）とスクロールバーの置き場所。収まらない向きにだけ出し、表に重ねずに場所を取る。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scrollbars {
    area: PixelRect,
    body: PixelRect,
    fixed: FixedParts,
    vertical: Option<ScrollBar>,
    horizontal: Option<ScrollBar>,
}

impl Scrollbars {
    /// 表の所 `area` に、固定の部分 `fixed` の表を置く。`rows` は本体の行（全部の数と最初の行）、`columns` は列の画素
    /// （全部の幅と最初の列の左端）。見えている量は置き場所から決める（`visible` は見ない）。
    ///
    /// 縦のバーで幅が減ると横も要ることがあり、その逆もあるので、2 回ずつ確かめる。
    #[must_use]
    pub fn fit(area: PixelRect, fixed: FixedParts, rows: ScrollAxis, columns: ScrollAxis) -> Self {
        let size = area.size();
        let scale = fixed.scale();
        let (bar_space, track_width) = (ScrollBar::space(scale), ScrollBar::track_width(scale));
        let space = |needed: bool| if needed { bar_space } else { 0 };
        let needs_vertical = |height: u32| rows.total() > fixed.rows_in(height);
        let needs_horizontal = |width: u32| columns.total() > fixed.columns_in(width);
        let first_vertical = needs_vertical(size.height());
        let first_horizontal = needs_horizontal(size.width().saturating_sub(space(first_vertical)));
        let vertical = needs_vertical(size.height().saturating_sub(space(first_horizontal)));
        let horizontal = needs_horizontal(size.width().saturating_sub(space(vertical)));
        let body = PixelRect::new(
            area.origin(),
            PixelSize::new(
                size.width().saturating_sub(space(vertical)),
                size.height().saturating_sub(space(horizontal)),
            ),
        );
        let (left, top) = (area.origin().x(), area.origin().y());
        let (width, height) = (body.size().width(), body.size().height());
        let vertical = vertical.then(|| {
            ScrollBar::new(
                ScrollOrientation::Vertical,
                PixelRect::new(
                    PixelPoint::new(
                        left.saturating_add(width)
                            .saturating_add(bar_space.saturating_sub(track_width)),
                        top.saturating_add(fixed.header()),
                    ),
                    PixelSize::new(track_width, height.saturating_sub(fixed.header())),
                ),
                ScrollAxis::new(rows.total(), fixed.rows_in(height), rows.first()),
                scale,
            )
        });
        let horizontal = horizontal.then(|| {
            ScrollBar::new(
                ScrollOrientation::Horizontal,
                PixelRect::new(
                    PixelPoint::new(
                        left.saturating_add(fixed.gutter()),
                        top.saturating_add(height)
                            .saturating_add(bar_space.saturating_sub(track_width)),
                    ),
                    PixelSize::new(width.saturating_sub(fixed.gutter()), track_width),
                ),
                ScrollAxis::new(columns.total(), fixed.columns_in(width), columns.first()),
                scale,
            )
        });
        Self {
            area,
            body,
            fixed,
            vertical: vertical.filter(|scrollbar| scrollbar.length() > 0),
            horizontal: horizontal.filter(|scrollbar| scrollbar.length() > 0),
        }
    }

    /// 表の所の全体。
    #[must_use]
    pub fn area(self) -> PixelRect {
        self.area
    }

    /// 本体（見出し・行番号・セル）の所。
    #[must_use]
    pub fn body(self) -> PixelRect {
        self.body
    }

    /// 固定の部分の大きさ。
    #[must_use]
    pub fn fixed(self) -> FixedParts {
        self.fixed
    }

    /// 向き `orientation` のバー（出していなければ `None`）。
    #[must_use]
    pub fn bar(self, orientation: ScrollOrientation) -> Option<ScrollBar> {
        match orientation {
            ScrollOrientation::Vertical => self.vertical,
            ScrollOrientation::Horizontal => self.horizontal,
        }
    }

    /// 出しているバー（縦・横の順）。
    pub fn bars(self) -> impl Iterator<Item = ScrollBar> {
        self.vertical.into_iter().chain(self.horizontal)
    }

    /// 窓の中の位置 `at` の当たったバーと所。
    #[must_use]
    pub fn hit(self, at: PixelPoint) -> Option<(ScrollBar, BarHit)> {
        self.bars()
            .find_map(|scrollbar| scrollbar.hit(at).map(|hit| (scrollbar, hit)))
    }

    /// バーの周りで、表にもバーにも使わない角（面の色で埋める）。見出しの行の右端・行番号の列の下端・右下。
    #[must_use]
    pub fn corners(self) -> Vec<PixelRect> {
        let bar_space = ScrollBar::space(self.fixed.scale());
        let (left, top) = (self.area.origin().x(), self.area.origin().y());
        let (width, height) = (self.body.size().width(), self.body.size().height());
        let right = left.saturating_add(width);
        let bottom = top.saturating_add(height);
        let mut corners = Vec::new();
        if self.vertical.is_some() && self.fixed.header() > 0 {
            corners.push(PixelRect::new(
                PixelPoint::new(right, top),
                PixelSize::new(bar_space, self.fixed.header()),
            ));
        }
        if self.horizontal.is_some() {
            corners.push(PixelRect::new(
                PixelPoint::new(left, bottom),
                PixelSize::new(self.fixed.gutter().min(width), bar_space),
            ));
        }
        if self.vertical.is_some() && self.horizontal.is_some() {
            corners.push(PixelRect::new(
                PixelPoint::new(right, bottom),
                PixelSize::new(bar_space, bar_space),
            ));
        }
        corners
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nene_ledger_core::display_scale::DisplayScale;

    fn area() -> PixelRect {
        PixelRect::new(PixelPoint::new(1, 28), PixelSize::new(400, 250))
    }

    /// 見出し 25・行 25・行番号 40。250 の高さに本体は 9 行、400 の幅にセルは 360。
    fn fixed() -> FixedParts {
        FixedParts::new(25, 25, 40)
    }

    fn fit(rows: u64, columns: u64) -> Scrollbars {
        Scrollbars::fit(
            area(),
            fixed(),
            ScrollAxis::new(rows, 0, 0),
            ScrollAxis::new(columns, 0, 0),
        )
    }

    #[test]
    fn bars_appear_only_in_the_directions_that_do_not_fit() {
        let none = fit(9, 360);
        assert_eq!((none.bars().count(), none.body()), (0, area()));
        assert!(none.corners().is_empty());
        // 列は 300 なので、縦のバーで幅が減っても入る。
        let tall = fit(10, 300);
        let scrollbar = tall.bar(ScrollOrientation::Vertical).unwrap();
        // 縦の溝は見出しの行の下から、表の右端に罫線 1 ＋ 溝 16。
        assert_eq!(
            scrollbar.track(),
            PixelRect::new(
                PixelPoint::new(1 + 383 + 1, 28 + 25),
                PixelSize::new(16, 225)
            )
        );
        assert_eq!(scrollbar.axis(), ScrollAxis::new(10, 9, 0));
        assert!(tall.bar(ScrollOrientation::Horizontal).is_none());
        assert_eq!(tall.corners().len(), 1);
        let wide = fit(9, 361);
        let sideways = wide.bar(ScrollOrientation::Horizontal).unwrap();
        // 横の溝は行番号の列の右から。横のバーで高さが減って 8 行しか入らないので、縦も要る。
        assert!(wide.bar(ScrollOrientation::Vertical).is_some());
        assert_eq!(
            sideways.track().origin(),
            PixelPoint::new(1 + 40, 28 + 233 + 1)
        );
        assert_eq!(wide.corners().len(), 3);
    }

    #[test]
    fn a_vertical_bar_can_make_the_horizontal_one_needed() {
        // 列は 350 で 360 に入るが、縦のバーで 343 に減ると入らない。
        let both = fit(100, 350);
        assert!(both.bar(ScrollOrientation::Horizontal).is_some());
        assert_eq!(both.body().size(), PixelSize::new(383, 233));
        let (scrollbar, hit) = both.hit(PixelPoint::new(390, 60)).unwrap();
        assert_eq!(scrollbar.orientation(), ScrollOrientation::Vertical);
        assert_eq!(hit, BarHit::Thumb { grab: 7 });
        assert!(both.hit(PixelPoint::new(100, 100)).is_none());
    }

    #[test]
    fn a_tiny_area_has_no_bar_without_room() {
        let tiny = Scrollbars::fit(
            PixelRect::new(PixelPoint::new(0, 0), PixelSize::new(30, 20)),
            fixed(),
            ScrollAxis::new(100, 0, 0),
            ScrollAxis::new(100, 0, 0),
        );
        assert!(tiny.bar(ScrollOrientation::Vertical).is_none());
    }

    #[test]
    fn the_bars_grow_with_the_display_scale() {
        let scale = DisplayScale::of_percent(150);
        let tall = Scrollbars::fit(
            area(),
            fixed().with_scale(scale),
            ScrollAxis::new(10, 0, 0),
            ScrollAxis::new(300, 0, 0),
        );
        // 溝 16 は 24、罫線 1 は 2。表は 400 − 26 = 374 の幅。
        let scrollbar = tall.bar(ScrollOrientation::Vertical).unwrap();
        assert_eq!(
            scrollbar.track(),
            PixelRect::new(
                PixelPoint::new(1 + 374 + 2, 28 + 25),
                PixelSize::new(24, 225)
            )
        );
        assert_eq!(tall.body().size().width(), 374);
        // 普段のつまみ 8 は 12、溝の真ん中に置く。
        assert_eq!(scrollbar.thumb_rect(false).size().width(), 12);
        assert_eq!(scrollbar.thumb_rect(false).origin().x(), 1 + 374 + 2 + 6);
    }
}
