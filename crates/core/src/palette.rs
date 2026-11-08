//! 画面の色（配色）。

use crate::color::Color;
use crate::palette_role::PaletteRole;

/// 画面の色。役割（`PaletteRole`）ごとに 1 色。製品では導出（`theme_derivation::derive`）が作る（ADR 0012）。
/// UI と application に色のリテラルは無く、役割から色を引く（ARC-011）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Palette {
    colors: [Color; PaletteRole::COUNT],
}

impl Palette {
    /// 役割ごとの色 `color_of` から作る。
    #[must_use]
    pub fn from_roles<F: Fn(PaletteRole) -> Color>(color_of: F) -> Self {
        Self {
            colors: PaletteRole::ALL.map(color_of),
        }
    }

    /// 役割 `role` の色。
    #[must_use]
    pub fn color(self, role: PaletteRole) -> Color {
        self.colors
            .get(role.index())
            .copied()
            .unwrap_or(Color::new(0, 0, 0))
    }

    /// 背景。
    #[must_use]
    pub fn background(self) -> Color {
        self.color(PaletteRole::Background)
    }

    /// セルの文字。
    #[must_use]
    pub fn text(self) -> Color {
        self.color(PaletteRole::Text)
    }

    /// 罫線。
    #[must_use]
    pub fn grid(self) -> Color {
        self.color(PaletteRole::Grid)
    }

    /// 行番号の列の背景。
    #[must_use]
    pub fn gutter(self) -> Color {
        self.color(PaletteRole::Gutter)
    }

    /// 行番号の文字。
    #[must_use]
    pub fn gutter_text(self) -> Color {
        self.color(PaletteRole::GutterText)
    }

    /// タイトルバーの背景。
    #[must_use]
    pub fn title_bar(self) -> Color {
        self.color(PaletteRole::TitleBar)
    }

    /// タイトルバーの文字とボタンの印。
    #[must_use]
    pub fn title_text(self) -> Color {
        self.color(PaletteRole::TitleText)
    }

    /// 指の乗ったボタンの背景（閉じる以外）。
    #[must_use]
    pub fn button_hover(self) -> Color {
        self.color(PaletteRole::ButtonHover)
    }

    /// 指の乗った閉じるボタンの背景。
    #[must_use]
    pub fn close_hover(self) -> Color {
        self.color(PaletteRole::CloseHover)
    }

    /// 指の乗った閉じるボタンの印。
    #[must_use]
    pub fn close_hover_text(self) -> Color {
        self.color(PaletteRole::CloseHoverText)
    }

    /// 窓の縁の線。
    #[must_use]
    pub fn border(self) -> Color {
        self.color(PaletteRole::Border)
    }

    /// 選択したセルの背景。
    #[must_use]
    pub fn selection(self) -> Color {
        self.color(PaletteRole::Selection)
    }

    /// カーソルのセルの枠。
    #[must_use]
    pub fn cursor(self) -> Color {
        self.color(PaletteRole::Cursor)
    }

    /// タイトルバーと中身の境の線。
    #[must_use]
    pub fn chrome_rule(self) -> Color {
        self.color(PaletteRole::ChromeRule)
    }

    /// 選んだ範囲と同じ行と列の、選択の外のセルの背景（ごく薄い帯）。
    #[must_use]
    pub fn band(self) -> Color {
        self.color(PaletteRole::Band)
    }

    /// スクロールバーのつまみ（普段。指が乗った間とドラッグ中は補助の字の色）。
    #[must_use]
    pub fn scroll_thumb(self) -> Color {
        self.color(PaletteRole::ScrollThumb)
    }

    /// 検索で当たったセルの背景（選んでいないセル・GitHub Issue #189）。
    #[must_use]
    pub fn found(self) -> Color {
        self.color(PaletteRole::Found)
    }

    /// 検索で当たった字の背景（GitHub Issue #189）。
    #[must_use]
    pub fn found_text(self) -> Color {
        self.color(PaletteRole::FoundText)
    }

    /// 検索で当たった字（当たった字の背景の上で読める濃い字・GitHub Issue #189）。
    #[must_use]
    pub fn found_ink(self) -> Color {
        self.color(PaletteRole::FoundInk)
    }

    /// 済んだことの字（緑・パレットの面と背景の上で読める・GitHub Issue #305）。
    #[must_use]
    pub fn success(self) -> Color {
        self.color(PaletteRole::Success)
    }

    /// 注意と誤りの字（赤・パレットの面と背景の上で読める・GitHub Issue #305）。
    #[must_use]
    pub fn danger(self) -> Color {
        self.color(PaletteRole::Danger)
    }
}
