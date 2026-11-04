//! 1 本のスクロールバーの置き場所と、つまみの位置・当たり判定。

use nene_ledger_core::display_scale::DisplayScale;
use nene_ledger_core::pixel_point::PixelPoint;
use nene_ledger_core::pixel_rect::PixelRect;
use nene_ledger_core::pixel_size::PixelSize;
use nene_ledger_core::pointer_point::PointerPoint;
use nene_ledger_core::scroll_axis::{MIN_THUMB, ScrollAxis};

use crate::bar_hit::BarHit;
use crate::scroll_orientation::ScrollOrientation;
use crate::track_side::TrackSide;

/// 溝の幅（画素・100% のとき）。文字の拡大には従わず、画面の倍率には従う（Issue #44・#58）。
pub const TRACK_WIDTH: u32 = 16;
/// 表と溝の間の罫線（画素・100% のとき）。
const BAR_RULE: u32 = 1;
/// 普段のつまみの太さ。
const THUMB_THIN: u32 = 8;
/// 指が乗った間とドラッグ中のつまみの太さ。
const THUMB_WIDE: u32 = 12;

/// 1 本のスクロールバー。向き・溝の矩形（窓の中の位置）・尺度。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollBar {
    orientation: ScrollOrientation,
    track: PixelRect,
    axis: ScrollAxis,
    scale: DisplayScale,
}

impl ScrollBar {
    /// 向き `orientation` の、溝 `track` と尺度 `axis` のスクロールバー。固定の画素は画面の倍率 `scale` で掛ける。
    #[must_use]
    pub fn new(
        orientation: ScrollOrientation,
        track: PixelRect,
        axis: ScrollAxis,
        scale: DisplayScale,
    ) -> Self {
        Self {
            orientation,
            track,
            axis: axis.with_min_thumb(scale.px(MIN_THUMB)),
            scale,
        }
    }

    /// 溝の幅（倍率を掛けた `TRACK_WIDTH`）。
    #[must_use]
    pub fn track_width(scale: DisplayScale) -> u32 {
        scale.px(TRACK_WIDTH)
    }

    /// 表と溝の間の罫線を含めた、スクロールバーが表から取る幅。
    #[must_use]
    pub fn space(scale: DisplayScale) -> u32 {
        scale.px(TRACK_WIDTH).saturating_add(scale.px(BAR_RULE))
    }

    /// 画面の倍率。
    #[must_use]
    pub fn scale(self) -> DisplayScale {
        self.scale
    }

    /// 向き。
    #[must_use]
    pub fn orientation(self) -> ScrollOrientation {
        self.orientation
    }

    /// 溝の矩形。
    #[must_use]
    pub fn track(self) -> PixelRect {
        self.track
    }

    /// 尺度。
    #[must_use]
    pub fn axis(self) -> ScrollAxis {
        self.axis
    }

    /// 尺度を替えたもの（ドラッグの間は押したときの尺度に固定する）。
    #[must_use]
    pub fn with_axis(self, axis: ScrollAxis) -> Self {
        Self {
            axis: axis.with_min_thumb(self.scale.px(MIN_THUMB)),
            ..self
        }
    }

    /// 溝の長さ（向きに沿って）。
    #[must_use]
    pub fn length(self) -> u32 {
        match self.orientation {
            ScrollOrientation::Vertical => self.track.size().height(),
            ScrollOrientation::Horizontal => self.track.size().width(),
        }
    }

    /// 指 `at` の、溝の始まりからの位置（向きに沿って・溝の外では負や長さより大きくもなる）。
    #[must_use]
    pub fn along(self, at: PointerPoint) -> i64 {
        let origin = self.track.origin();
        match self.orientation {
            ScrollOrientation::Vertical => i64::from(at.y()) - i64::from(origin.y()),
            ScrollOrientation::Horizontal => i64::from(at.x()) - i64::from(origin.x()),
        }
    }

    /// つまみの始まりと長さ（溝の始まりから・向きに沿って）。
    #[must_use]
    pub fn thumb(self) -> (u32, u32) {
        self.axis.thumb(self.length())
    }

    /// つまみの矩形。`wide` なら太く（指が乗った・ドラッグ中）。
    #[must_use]
    pub fn thumb_rect(self, wide: bool) -> PixelRect {
        let (offset, length) = self.thumb();
        let thickness = self.scale.px(if wide { THUMB_WIDE } else { THUMB_THIN });
        let inset = Self::track_width(self.scale).saturating_sub(thickness) / 2;
        self.span((offset, length), (inset, thickness))
    }

    /// 溝の中の、向きに沿って `along`（始まり・長さ）、幅の向きに `across`（始まり・太さ）の矩形。
    #[must_use]
    pub fn span(self, along: (u32, u32), across: (u32, u32)) -> PixelRect {
        let ((offset, length), (inset, thickness)) = (along, across);
        let origin = self.track.origin();
        match self.orientation {
            ScrollOrientation::Vertical => PixelRect::new(
                PixelPoint::new(
                    origin.x().saturating_add(inset),
                    origin.y().saturating_add(offset),
                ),
                PixelSize::new(thickness, length),
            ),
            ScrollOrientation::Horizontal => PixelRect::new(
                PixelPoint::new(
                    origin.x().saturating_add(offset),
                    origin.y().saturating_add(inset),
                ),
                PixelSize::new(length, thickness),
            ),
        }
    }

    /// 窓の中の位置 `at` が溝のどこに当たるか。溝の外なら `None`。当たり判定は溝の幅いっぱい。
    #[must_use]
    pub fn hit(self, at: PixelPoint) -> Option<BarHit> {
        let origin = self.track.origin();
        let x = at.x().checked_sub(origin.x())?;
        let y = at.y().checked_sub(origin.y())?;
        if x >= self.track.size().width() || y >= self.track.size().height() {
            return None;
        }
        let along = match self.orientation {
            ScrollOrientation::Vertical => y,
            ScrollOrientation::Horizontal => x,
        };
        let (offset, length) = self.thumb();
        Some(if along < offset {
            BarHit::Track(TrackSide::Before)
        } else if along >= offset.saturating_add(length) {
            BarHit::Track(TrackSide::After)
        } else {
            BarHit::Thumb {
                grab: i64::from(along - offset),
            }
        })
    }

    /// 溝の位置 `along` がつまみの中か（溝を押し続けたとき、つまみが指まで来たら止める）。
    #[must_use]
    pub fn thumb_covers(self, along: i64) -> bool {
        let (offset, length) = self.thumb();
        along >= i64::from(offset) && along < i64::from(offset) + i64::from(length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vertical() -> ScrollBar {
        // 溝は (300, 25) から 16 × 200。100 行のうち 25 行・先頭から 25 行目。
        ScrollBar::new(
            ScrollOrientation::Vertical,
            PixelRect::new(PixelPoint::new(300, 25), PixelSize::new(16, 200)),
            ScrollAxis::new(100, 25, 25),
            DisplayScale::hundred(),
        )
    }

    #[test]
    fn the_thumb_sits_inside_the_track_and_grows_when_wide() {
        let scrollbar = vertical();
        assert_eq!(scrollbar.thumb(), (50, 50));
        assert_eq!(
            scrollbar.thumb_rect(false),
            PixelRect::new(PixelPoint::new(304, 75), PixelSize::new(8, 50))
        );
        assert_eq!(
            scrollbar.thumb_rect(true),
            PixelRect::new(PixelPoint::new(302, 75), PixelSize::new(12, 50))
        );
        let sideways = ScrollBar::new(
            ScrollOrientation::Horizontal,
            PixelRect::new(PixelPoint::new(40, 300), PixelSize::new(200, 16)),
            ScrollAxis::new(100, 25, 0),
            DisplayScale::hundred(),
        );
        assert_eq!(
            sideways.thumb_rect(false),
            PixelRect::new(PixelPoint::new(40, 304), PixelSize::new(50, 8))
        );
        assert_eq!(sideways.along(PointerPoint::new(30, -5)), -10);
    }

    #[test]
    fn pressing_tells_the_thumb_from_the_track() {
        let scrollbar = vertical();
        assert_eq!(
            scrollbar.hit(PixelPoint::new(300, 30)),
            Some(BarHit::Track(TrackSide::Before))
        );
        assert_eq!(
            scrollbar.hit(PixelPoint::new(315, 80)),
            Some(BarHit::Thumb { grab: 5 })
        );
        assert_eq!(
            scrollbar.hit(PixelPoint::new(310, 125)),
            Some(BarHit::Track(TrackSide::After))
        );
        assert_eq!(scrollbar.hit(PixelPoint::new(316, 80)), None);
        assert_eq!(scrollbar.hit(PixelPoint::new(310, 24)), None);
        assert!(
            scrollbar.thumb_covers(50)
                && scrollbar.thumb_covers(99)
                && !scrollbar.thumb_covers(100)
        );
        assert_eq!(scrollbar.along(PointerPoint::new(0, 30)), 5);
    }
}
