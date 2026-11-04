//! 組み込みの配色 1 つの元: 名前・明暗・本文の 3 色・出典。台帳の色は `theme_derivation` が導く。

use crate::appearance::Appearance;
use crate::color::Color;

/// 名前（小文字とハイフン）・明暗・背景・前景・強調・作者・許諾・出典の URL。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeSeed {
    name: &'static str,
    appearance: Appearance,
    background: Color,
    foreground: Color,
    accent: Color,
    author: &'static str,
    license: &'static str,
    url: &'static str,
}

impl ThemeSeed {
    /// 表の 1 行。
    #[must_use]
    pub const fn new(
        name: &'static str,
        appearance: Appearance,
        (background, foreground, accent): (Color, Color, Color),
        (author, license, url): (&'static str, &'static str, &'static str),
    ) -> Self {
        Self {
            name,
            appearance,
            background,
            foreground,
            accent,
            author,
            license,
            url,
        }
    }

    /// 名前。
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// 明暗。
    #[must_use]
    pub const fn appearance(&self) -> Appearance {
        self.appearance
    }

    /// 背景。
    #[must_use]
    pub const fn background(&self) -> Color {
        self.background
    }

    /// 前景（字）。
    #[must_use]
    pub const fn foreground(&self) -> Color {
        self.foreground
    }

    /// 強調。
    #[must_use]
    pub const fn accent(&self) -> Color {
        self.accent
    }

    /// 作者。
    #[must_use]
    pub const fn author(&self) -> &'static str {
        self.author
    }

    /// 許諾。
    #[must_use]
    pub const fn license(&self) -> &'static str {
        self.license
    }

    /// 出典の URL。アプリは読みに行かない。
    #[must_use]
    pub const fn url(&self) -> &'static str {
        self.url
    }
}
