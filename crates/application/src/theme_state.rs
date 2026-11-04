//! 今の配色。

use nene_ledger_core::palette::Palette;
use nene_ledger_core::theme_catalog::{COUNT, at, position_of};
use nene_ledger_core::theme_derivation::derive;
use nene_ledger_core::theme_seed::ThemeSeed;

use crate::theme_step::ThemeStep;

/// 今の配色（組み込みの表の中の位置）と、そこから導いた画面の色。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeState {
    position: usize,
    palette: Palette,
}

impl ThemeState {
    /// 名前 `name` の配色。表に無ければ `None`。
    #[must_use]
    pub fn named(name: &str) -> Option<Self> {
        let canonical: alloc::string::String = name
            .chars()
            .map(|c| if c == '_' { '-' } else { c })
            .collect();
        position_of(&canonical).map(Self::at)
    }

    /// 配色の元 `seed` の配色。
    #[must_use]
    pub fn of(seed: &ThemeSeed) -> Self {
        Self::at(position_of(seed.name()).unwrap_or(0))
    }

    /// 今の配色の元（設定に残す名前）。
    #[must_use]
    pub fn seed(self) -> &'static ThemeSeed {
        at(self.position)
    }

    /// 表の `position` 番目（端で巡る）。
    #[must_use]
    pub fn at(position: usize) -> Self {
        let position = position % COUNT;
        Self {
            position,
            palette: derive(at(position)),
        }
    }

    /// 画面の色。
    #[must_use]
    pub fn palette(self) -> Palette {
        self.palette
    }

    /// 配色の名前。
    #[must_use]
    pub fn name(self) -> &'static str {
        at(self.position).name()
    }

    /// 表の中の位置（0 始まり）。
    #[must_use]
    pub fn position(self) -> usize {
        self.position
    }

    /// `step` の向きに 1 つ替えた配色（端で巡る）。
    #[must_use]
    pub fn stepped(self, step: ThemeStep) -> Self {
        let position = match step {
            ThemeStep::Next => self.position.saturating_add(1),
            ThemeStep::Previous => self.position.checked_sub(1).unwrap_or(COUNT - 1),
        };
        Self::at(position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn themes_are_found_by_name_and_cycle_at_both_ends() {
        let light = ThemeState::named("neutral-light").unwrap();
        assert_eq!(light.position(), 1);
        assert_eq!(ThemeState::named("neutral_light"), Some(light));
        assert_eq!(ThemeState::named("no-such-theme"), None);
        let first = light.stepped(ThemeStep::Previous);
        assert_eq!(first.name(), "ubuntu-aubergine");
        assert_eq!(first.stepped(ThemeStep::Previous).position(), COUNT - 1);
        assert_eq!(
            ThemeState::at(COUNT - 1)
                .stepped(ThemeStep::Next)
                .position(),
            0
        );
        assert_eq!(light.palette(), derive(at(1)));
        assert_eq!(ThemeState::of(light.seed()), light);
    }
}
