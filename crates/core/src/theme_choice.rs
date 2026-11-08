//! 設定に残す配色の希望（ADR 0012）。

use crate::appearance::Appearance;
use crate::theme_catalog;
use crate::theme_seed::ThemeSeed;

/// `system` の語。
const SYSTEM: &str = "system";

/// 設定に残す配色の希望。`system`（OS の明暗に従う）か、組み込みの名前。表示する配色は観測した明暗と合わせて解決する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeChoice {
    /// OS の明暗に従い、その既定の配色を使う。
    System,
    /// 組み込みの配色。
    Named(&'static ThemeSeed),
}

impl ThemeChoice {
    /// 設定の語から。`system` か組み込みの名前（`_` も受ける）。知らない名前は `None`（黙って既定値へ落とさない）。
    #[must_use]
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        if keyword == SYSTEM {
            return Some(Self::System);
        }
        theme_catalog::named(keyword).map(Self::Named)
    }

    /// 設定に書く語。
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::System => SYSTEM,
            Self::Named(seed) => seed.name(),
        }
    }

    /// 観測した OS の明暗（分からなければライト）と合わせて、表示する配色の元にする。OS の高コントラストが有効なら
    /// （`high_contrast` がその明暗）、希望より優先して高コントラストの配色にする（A11Y-02）。
    #[must_use]
    pub fn resolve(
        self,
        observed: Option<Appearance>,
        high_contrast: Option<Appearance>,
    ) -> &'static ThemeSeed {
        if let Some(appearance) = high_contrast {
            return theme_catalog::high_contrast_for(appearance);
        }
        match self {
            Self::System => theme_catalog::default_for(observed.unwrap_or(Appearance::Light)),
            Self::Named(seed) => seed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords_round_trip_and_system_follows_the_observed_appearance() {
        assert_eq!(
            ThemeChoice::from_keyword("system"),
            Some(ThemeChoice::System)
        );
        let solarized = ThemeChoice::from_keyword("solarized_dark").unwrap();
        assert_eq!(solarized.keyword(), "solarized-dark");
        assert_eq!(
            ThemeChoice::from_keyword(solarized.keyword()),
            Some(solarized)
        );
        assert_eq!(ThemeChoice::from_keyword("no-such-theme"), None);
        assert_eq!(ThemeChoice::System.keyword(), "system");
        assert_eq!(
            ThemeChoice::System.resolve(None, None).name(),
            "neutral-light"
        );
        assert_eq!(
            ThemeChoice::System
                .resolve(Some(Appearance::Dark), None)
                .name(),
            "ubuntu-aubergine"
        );
        assert_eq!(
            solarized.resolve(Some(Appearance::Light), None).name(),
            "solarized-dark"
        );
    }

    #[test]
    fn the_os_high_contrast_wins_over_the_saved_choice() {
        let solarized = ThemeChoice::from_keyword("solarized-dark").unwrap();
        for choice in [ThemeChoice::System, solarized] {
            assert_eq!(
                choice
                    .resolve(Some(Appearance::Light), Some(Appearance::Dark))
                    .name(),
                "vscode-high-contrast"
            );
            assert_eq!(
                choice
                    .resolve(Some(Appearance::Dark), Some(Appearance::Light))
                    .name(),
                "vscode-high-contrast-light"
            );
        }
    }
}
