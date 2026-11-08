//! 表と詳細ペインの文字の大きさ。

use crate::zoom_step::ZoomStep;

/// 段階（百分率）。Excel とブラウザーに近い（ADR 0020 の決定 1）。
const STEPS: [u32; 12] = [50, 67, 80, 90, 100, 110, 125, 150, 175, 200, 250, 300];
/// 100% の段階の位置。
const HUNDRED: usize = 4;

/// 表と詳細ペインの文字の大きさ（段階の 1 つ）。タイトルバーと状態の行は変えない（ADR 0020）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZoomLevel {
    step: usize,
}

impl ZoomLevel {
    /// 100%。
    #[must_use]
    pub fn hundred() -> Self {
        Self { step: HUNDRED }
    }

    /// 百分率 `percent` の段階。段階に無い値は `None`（設定のファイルで黙って近い段階へ寄せない）。
    #[must_use]
    pub fn from_percent(percent: u32) -> Option<Self> {
        STEPS
            .iter()
            .position(|&step| step == percent)
            .map(|step| Self { step })
    }

    /// 百分率。
    #[must_use]
    pub fn percent(self) -> u32 {
        STEPS.get(self.step).copied().unwrap_or(100)
    }

    /// この大きさに OS の文字の倍率 `percent`（百分率・100 で変えない）を掛けた、いちばん近い段（同じ近さなら小さい
    /// 方・端で止まる・A11Y-03・GitHub Issue #263）。
    #[must_use]
    pub fn scaled(self, percent: u32) -> Self {
        let wanted = u64::from(self.percent()).saturating_mul(u64::from(percent)) / 100;
        let step = STEPS
            .iter()
            .enumerate()
            .min_by_key(|&(_, &step)| u64::from(step).abs_diff(wanted))
            .map_or(HUNDRED, |(index, _)| index);
        Self { step }
    }

    /// `step` で変えた大きさ。端では止まる。
    #[must_use]
    pub fn stepped(self, step: ZoomStep) -> Self {
        let last = STEPS.len().saturating_sub(1);
        let step = match step {
            ZoomStep::In => self.step.saturating_add(1).min(last),
            ZoomStep::Out => self.step.saturating_sub(1),
            ZoomStep::Reset => HUNDRED,
        };
        Self { step }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_stop_at_both_ends_and_reset_to_a_hundred() {
        let hundred = ZoomLevel::hundred();
        assert_eq!(hundred.percent(), 100);
        assert_eq!(hundred.stepped(ZoomStep::In).percent(), 110);
        assert_eq!(hundred.stepped(ZoomStep::Out).percent(), 90);
        let mut largest = hundred;
        for _ in 0..20_u32 {
            largest = largest.stepped(ZoomStep::In);
        }
        assert_eq!(largest.percent(), 300);
        let mut smallest = hundred;
        for _ in 0..20_u32 {
            smallest = smallest.stepped(ZoomStep::Out);
        }
        assert_eq!(smallest.percent(), 50);
        assert_eq!(largest.stepped(ZoomStep::Reset), hundred);
    }

    #[test]
    fn only_listed_percentages_can_be_read_back() {
        assert_eq!(
            ZoomLevel::from_percent(125).map(ZoomLevel::percent),
            Some(125)
        );
        assert_eq!(ZoomLevel::from_percent(123), None);
    }

    /// OS の倍率を掛けて、いちばん近い段に寄せる。同じ近さなら小さい方。端で止まる（GitHub Issue #263）。
    #[test]
    fn os_text_scale_picks_the_nearest_step() {
        let percent = |zoom: ZoomLevel, scale: u32| zoom.scaled(scale).percent();
        let hundred = ZoomLevel::hundred();
        assert_eq!(percent(hundred, 100), 100);
        assert_eq!(percent(hundred, 125), 125);
        assert_eq!(percent(hundred, 140), 150);
        assert_eq!(percent(hundred, 225), 200);
        // 150% × 150% = 225% は 200 と 250 のちょうど真ん中なので、小さい方。
        assert_eq!(percent(ZoomLevel::from_percent(150).unwrap(), 150), 200);
        assert_eq!(percent(ZoomLevel::from_percent(300).unwrap(), 225), 300);
        assert_eq!(percent(ZoomLevel::from_percent(50).unwrap(), 100), 50);
        assert_eq!(percent(hundred, 0), 50);
        // 90% × 125% = 112.5% は 110 と 125 の間で 110 の方が近い。
        assert_eq!(percent(ZoomLevel::from_percent(90).unwrap(), 125), 110);
    }
}
