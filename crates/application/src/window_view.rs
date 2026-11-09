//! 今の窓の見え。

use nene_ledger_core::char_advances::CharAdvances;
use nene_ledger_core::chrome_button::ChromeButton;
use nene_ledger_core::glyph_metrics::GlyphMetrics;
use nene_ledger_core::pixel_size::PixelSize;
use nene_ledger_core::window_chrome::WindowChrome;
use nene_ledger_core::window_placement::WindowPlacement;

/// 今の窓の見え。大きさ・窓の部品と中身の字の寸法・置かれ方・指の乗ったボタン。UI が OS から受け取って毎回渡す。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowView {
    size: PixelSize,
    metrics: GlyphMetrics,
    content: GlyphMetrics,
    /// 中身の字の送り幅（UI が測った値・GitHub Issue #462）。
    advances: CharAdvances,
    placement: WindowPlacement,
    hovered: Option<ChromeButton>,
}

impl WindowView {
    /// 見えを作る。
    #[must_use]
    pub fn new(
        size: PixelSize,
        metrics: GlyphMetrics,
        placement: WindowPlacement,
        hovered: Option<ChromeButton>,
    ) -> Self {
        Self {
            size,
            metrics,
            content: metrics,
            advances: CharAdvances::uniform(metrics.half_width()),
            placement,
            hovered,
        }
    }

    /// 中身（表・詳細ペイン）の字の寸法を替えた見え（利用者の拡大・ADR 0020）。作った直後は窓の部品と同じ。
    #[must_use]
    /// 送り幅は測っていない値（半角は `0` の幅・全角はその 2 倍）に戻る。測った値は [`Self::with_advances`] で渡す。
    pub fn with_content_metrics(self, content: GlyphMetrics) -> Self {
        Self {
            content,
            advances: CharAdvances::uniform(content.half_width()),
            ..self
        }
    }

    /// 中身の字体から UI が測った字の送り幅を持たせた見え（詳細ペインの押した所・カーソルの棒・選択を実際の字に合わせる・
    /// GitHub Issue #462）。
    #[must_use]
    pub fn with_advances(self, advances: CharAdvances) -> Self {
        Self { advances, ..self }
    }

    /// 中身の字の送り幅。
    #[must_use]
    pub fn advances(self) -> CharAdvances {
        self.advances
    }

    /// 中身（表・詳細ペイン）の字の寸法。
    #[must_use]
    pub fn content_metrics(self) -> GlyphMetrics {
        self.content
    }

    /// 窓の大きさ。
    #[must_use]
    pub fn size(self) -> PixelSize {
        self.size
    }

    /// 窓の部品（タイトルバー・状態の行）の字の寸法（UI が字体から測った値）。
    #[must_use]
    pub fn metrics(self) -> GlyphMetrics {
        self.metrics
    }

    /// 指の乗ったボタン。
    #[must_use]
    pub fn hovered(self) -> Option<ChromeButton> {
        self.hovered
    }

    /// 窓の飾りの配置と当たり判定（core）。
    #[must_use]
    pub fn chrome(self) -> WindowChrome {
        WindowChrome::new(self.size, self.metrics, self.placement)
    }
}
