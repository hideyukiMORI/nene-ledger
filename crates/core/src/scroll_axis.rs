//! 1 本のスクロールバーの尺度と、溝の中のつまみと印の位置。

use core::ops::Range;

/// つまみの最小の長さ（画素）。巨大なファイルでも指で押さえられる長さ（アクセシビリティ・Issue #44）。
pub const MIN_THUMB: u32 = 32;

/// 1 本のスクロールバーの尺度。全体の量・見えている量・見えている最初の位置（縦は本体の行、横は列の画素）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollAxis {
    total: u64,
    visible: u64,
    first: u64,
    min_thumb: u32,
}

impl ScrollAxis {
    /// 全体 `total` のうち、`first` から `visible` だけ見えている。
    #[must_use]
    pub fn new(total: u64, visible: u64, first: u64) -> Self {
        Self {
            total,
            visible,
            first,
            min_thumb: MIN_THUMB,
        }
    }

    /// つまみの最小の長さを `min_thumb` にしたもの（画面の倍率を掛けた `MIN_THUMB`）。
    #[must_use]
    pub fn with_min_thumb(self, min_thumb: u32) -> Self {
        Self { min_thumb, ..self }
    }

    /// 全体の量。
    #[must_use]
    pub fn total(self) -> u64 {
        self.total
    }

    /// 見えている量。
    #[must_use]
    pub fn visible(self) -> u64 {
        self.visible
    }

    /// 見えている最初の位置。
    #[must_use]
    pub fn first(self) -> u64 {
        self.first
    }

    /// 見えていない所があるか（スクロールバーを出すか）。
    #[must_use]
    pub fn needed(self) -> bool {
        self.total > self.visible
    }

    /// 長さ `track` の溝の中の、つまみの始まりと長さ。長さは見えている割合（`MIN_THUMB` より短くしない）。
    /// 始まりは、最初の位置を動ける範囲（全体 − 見えている量）の中の割合で決める。
    #[must_use]
    pub fn thumb(self, track: u32) -> (u32, u32) {
        let length = self.thumb_length(track);
        let room = track.saturating_sub(length);
        let span = self.span();
        if span == 0 {
            return (0, length);
        }
        let offset = scale(u64::from(room), self.first.min(span), span);
        (u32::try_from(offset).unwrap_or(room), length)
    }

    /// つまみの始まりを溝の `offset`（溝の外なら端に寄せる）に置いたときの、最初の位置（つまみのドラッグ）。
    #[must_use]
    pub fn first_at(self, track: u32, offset: i64) -> u64 {
        let room = track.saturating_sub(self.thumb_length(track));
        if room == 0 {
            return 0;
        }
        let offset = u64::try_from(offset.max(0))
            .unwrap_or(0)
            .min(u64::from(room));
        // 四捨五入で、つまみを端まで寄せたら必ず端の位置になる。
        let span = self.span();
        let half = u64::from(room) / 2;
        u64::try_from((u128::from(span) * u128::from(offset) + u128::from(half)) / u128::from(room))
            .unwrap_or(span)
    }

    /// 全体の `range`（終わりは含まない）を、長さ `track` の溝に写した始まりと長さ（印）。短くても `min` にし、溝からはみ出さない。
    #[must_use]
    pub fn mark(self, track: u32, range: Range<u64>, min: u32) -> (u32, u32) {
        let (from, to) = (range.start, range.end);
        if self.total == 0 {
            return (0, min.min(track));
        }
        let at = |position: u64| {
            u32::try_from(scale(
                u64::from(track),
                position.min(self.total),
                self.total,
            ))
            .unwrap_or(track)
        };
        let start = at(from);
        let length = at(to.max(from)).saturating_sub(start).max(min).min(track);
        (start.min(track.saturating_sub(length)), length)
    }

    /// つまみの長さ。
    fn thumb_length(self, track: u32) -> u32 {
        let minimum = self.min_thumb.min(track);
        if self.total == 0 {
            return track;
        }
        let shown = scale(u64::from(track), self.visible.min(self.total), self.total);
        u32::try_from(shown).unwrap_or(track).clamp(minimum, track)
    }

    /// 最初の位置の動ける範囲。
    fn span(self) -> u64 {
        self.total.saturating_sub(self.visible)
    }
}

/// `value` × `numerator` ÷ `denominator`（切り捨て・あふれない）。
fn scale(value: u64, numerator: u64, denominator: u64) -> u64 {
    if denominator == 0 {
        return 0;
    }
    u64::try_from(u128::from(value) * u128::from(numerator) / u128::from(denominator))
        .unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_thumb_shows_the_visible_share_and_the_position() {
        // 100 行のうち 25 行が見えている・溝 200。つまみは 50、動ける所は 150。
        let top = ScrollAxis::new(100, 25, 0);
        assert!(top.needed());
        assert_eq!(top.thumb(200), (0, 50));
        assert_eq!(ScrollAxis::new(100, 25, 75).thumb(200), (150, 50));
        assert_eq!(ScrollAxis::new(100, 25, 300).thumb(200), (150, 50));
        assert_eq!(ScrollAxis::new(100, 25, 25).thumb(200), (50, 50));
        assert!(!ScrollAxis::new(10, 25, 0).needed());
        assert_eq!(ScrollAxis::new(0, 25, 0).thumb(200), (0, 200));
    }

    #[test]
    fn a_huge_total_keeps_the_minimum_thumb() {
        let rows = ScrollAxis::new(100_000_000, 30, 50_000_000);
        assert_eq!(rows.thumb(400), (184, MIN_THUMB));
        assert_eq!(ScrollAxis::new(u64::MAX, 1, u64::MAX).thumb(400).0, 368);
        // 溝が最小より短ければ溝いっぱい。
        assert_eq!(ScrollAxis::new(100, 1, 0).thumb(20), (0, 20));
    }

    #[test]
    fn dragging_the_thumb_gives_back_the_first_position() {
        let axis = ScrollAxis::new(100, 25, 0);
        assert_eq!(axis.first_at(200, 0), 0);
        assert_eq!(axis.first_at(200, 150), 75);
        assert_eq!(axis.first_at(200, 999), 75);
        assert_eq!(axis.first_at(200, -40), 0);
        assert_eq!(axis.first_at(200, 50), 25);
        // 動ける所が無ければ先頭。
        assert_eq!(ScrollAxis::new(100, 1, 0).first_at(20, 10), 0);
        for offset in 0..=150_i64 {
            let first = axis.first_at(200, offset);
            let (back, _) = ScrollAxis::new(100, 25, first).thumb(200);
            assert!(i64::from(back).abs_diff(offset) <= 2);
        }
    }

    #[test]
    fn marks_are_placed_by_share_and_kept_visible() {
        let axis = ScrollAxis::new(1_000, 25, 0);
        assert_eq!(axis.mark(200, 500..600, 4), (100, 20));
        // 1 行は 0.2 画素なので最小の長さ。末尾でも溝からはみ出さない。
        assert_eq!(axis.mark(200, 10..11, 2), (2, 2));
        assert_eq!(axis.mark(200, 999..1_000, 4), (196, 4));
        let (later, earlier) = (600, 500);
        assert_eq!(axis.mark(200, later..earlier, 4), (120, 4));
        assert_eq!(ScrollAxis::new(0, 25, 0).mark(200, 0..1, 4), (0, 4));
    }
}
