//! 字の寸法。

use crate::display_scale::DisplayScale;

/// UI が字体から測って値で渡す字の寸法（物理画素）と、測ったときの画面の倍率。core は字体を知らない（ADR 0002 の決定 2）。
/// 固定の画素は `px` だけを通して倍率を掛ける（ADR 0020 の 5・Issue #58）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphMetrics {
    half_width: u32,
    line_height: u32,
    ascent: u32,
    scale: DisplayScale,
}

impl GlyphMetrics {
    /// 寸法を作る。`half_width` は半角 1 字の幅、`ascent` は行の上端から基準線までの高さ。
    #[must_use]
    pub fn new(half_width: u32, line_height: u32, ascent: u32) -> Self {
        Self {
            half_width,
            line_height,
            ascent,
            scale: DisplayScale::hundred(),
        }
    }

    /// 画面の倍率 `scale` で測った寸法にする（字の寸法は UI が倍率を掛けた字で測る）。
    #[must_use]
    pub fn with_scale(self, scale: DisplayScale) -> Self {
        Self { scale, ..self }
    }

    /// 画面の倍率。
    #[must_use]
    pub fn scale(self) -> DisplayScale {
        self.scale
    }

    /// 固定の画素 `pixels` に画面の倍率を掛ける（窓の縁・罫線・帯・余白などの唯一の経路）。
    #[must_use]
    pub fn px(self, pixels: u32) -> u32 {
        self.scale.px(pixels)
    }

    /// 半角 1 字の幅（全角はその 2 倍とみなす）。
    #[must_use]
    pub fn half_width(self) -> u32 {
        self.half_width
    }

    /// 1 行の高さ。
    #[must_use]
    pub fn line_height(self) -> u32 {
        self.line_height
    }

    /// 行の上端から基準線までの高さ。
    #[must_use]
    pub fn ascent(self) -> u32 {
        self.ascent
    }
}
