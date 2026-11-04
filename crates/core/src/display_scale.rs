//! 画面の倍率（OS の拡大率・WIN-09）。

/// 画面の倍率（百分率・100 は等倍）。OS の拡大率（125%・150% など）で、窓全体の字と固定の画素に掛ける（ADR 0020 の 5）。
/// 利用者の拡大（`ZoomLevel`）とは別で、こちらは窓の部品も大きくする。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DisplayScale {
    percent: u32,
}

/// 等倍の dpi（Windows の 96）。
const BASE_DPI: u32 = 96;
/// 倍率の下限と上限（百分率）。
const MIN_PERCENT: u32 = 100;
const MAX_PERCENT: u32 = 500;

impl DisplayScale {
    /// 等倍（100%）。
    #[must_use]
    pub const fn hundred() -> Self {
        Self { percent: 100 }
    }

    /// 百分率 `percent` の倍率（100〜500 に収める）。
    #[must_use]
    pub fn of_percent(percent: u32) -> Self {
        Self {
            percent: percent.clamp(MIN_PERCENT, MAX_PERCENT),
        }
    }

    /// dpi `dpi` の倍率（96 で 100%・120 で 125%・144 で 150%）。
    #[must_use]
    pub fn of_dpi(dpi: u32) -> Self {
        let percent = dpi.saturating_mul(100).saturating_add(BASE_DPI / 2) / BASE_DPI;
        Self::of_percent(percent)
    }

    /// 百分率。
    #[must_use]
    pub fn percent(self) -> u32 {
        self.percent
    }

    /// 固定の画素 `pixels` に倍率を掛ける（四捨五入）。1 以上は 1 以上のまま（細い線が消えない）。
    #[must_use]
    pub fn px(self, pixels: u32) -> u32 {
        if pixels == 0 {
            return 0;
        }
        let scaled = u64::from(pixels)
            .saturating_mul(u64::from(self.percent))
            .saturating_add(50)
            / 100;
        u32::try_from(scaled).unwrap_or(u32::MAX).max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_pixels_grow_with_the_scale_and_stay_visible() {
        let hundred = DisplayScale::hundred();
        assert_eq!((hundred.px(1), hundred.px(6), hundred.px(16)), (1, 6, 16));
        let quarter = DisplayScale::of_dpi(120);
        assert_eq!(quarter.percent(), 125);
        assert_eq!((quarter.px(1), quarter.px(6), quarter.px(16)), (1, 8, 20));
        let half = DisplayScale::of_dpi(144);
        assert_eq!(
            (half.percent(), half.px(1), half.px(3), half.px(16)),
            (150, 2, 5, 24)
        );
        assert_eq!(DisplayScale::of_percent(200).px(17), 34);
        assert_eq!(DisplayScale::of_percent(50), hundred);
        assert_eq!(hundred.px(0), 0);
    }
}
