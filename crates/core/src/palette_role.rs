//! 画面の色の役割。

/// 画面の色の役割。閉じた集合（RS-002）。配色はすべての役割に色を持ち、導出（`theme_derivation`）が決める（ADR 0012）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaletteRole {
    /// 背景（表のセル・詳細ペインの本文）。
    Background,
    /// 本文の字。
    Text,
    /// 罫線。
    Grid,
    /// 面（行番号の列・見出しの行・セルが無い・状態の行・ペインの見出し）。
    Gutter,
    /// 補助の字（行番号・状態の行・ペインの補助の情報）。
    GutterText,
    /// タイトルバーの背景。
    TitleBar,
    /// タイトルバーの字とボタンの印。
    TitleText,
    /// 指の乗ったボタンの背景（閉じる以外）。
    ButtonHover,
    /// 指の乗った閉じるボタンの背景。
    CloseHover,
    /// 指の乗った閉じるボタンの印。
    CloseHoverText,
    /// 窓の縁とペインの境の線。
    Border,
    /// 選んだセルの背景。
    Selection,
    /// カーソルの枠・選んだ範囲の外周・ペインの帯。
    Cursor,
    /// タイトルバーと中身の境の線。
    ChromeRule,
    /// 選んだ範囲と同じ行と列の帯。
    Band,
    /// スクロールバーのつまみ（普段）。
    ScrollThumb,
    /// 検索で当たったセルの背景（GitHub Issue #189）。
    Found,
    /// 検索で当たった字の背景（GitHub Issue #189）。
    FoundText,
    /// 検索で当たった字（当たった字の背景の上・GitHub Issue #189）。
    FoundInk,
}

impl PaletteRole {
    /// 役割の数。
    pub const COUNT: usize = 19;

    /// すべての役割（並びは配色が色を持つ順）。
    pub const ALL: [Self; Self::COUNT] = [
        Self::Background,
        Self::Text,
        Self::Grid,
        Self::Gutter,
        Self::GutterText,
        Self::TitleBar,
        Self::TitleText,
        Self::ButtonHover,
        Self::CloseHover,
        Self::CloseHoverText,
        Self::Border,
        Self::Selection,
        Self::Cursor,
        Self::ChromeRule,
        Self::Band,
        Self::ScrollThumb,
        Self::Found,
        Self::FoundText,
        Self::FoundInk,
    ];

    /// 配色の中の位置。
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Background => 0,
            Self::Text => 1,
            Self::Grid => 2,
            Self::Gutter => 3,
            Self::GutterText => 4,
            Self::TitleBar => 5,
            Self::TitleText => 6,
            Self::ButtonHover => 7,
            Self::CloseHover => 8,
            Self::CloseHoverText => 9,
            Self::Border => 10,
            Self::Selection => 11,
            Self::Cursor => 12,
            Self::ChromeRule => 13,
            Self::Band => 14,
            Self::ScrollThumb => 15,
            Self::Found => 16,
            Self::FoundText => 17,
            Self::FoundInk => 18,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_role_has_its_own_place_in_order() {
        for (place, role) in PaletteRole::ALL.into_iter().enumerate() {
            assert_eq!(role.index(), place);
        }
    }
}
