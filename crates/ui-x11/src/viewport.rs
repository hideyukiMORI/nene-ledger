//! 今の窓の大きさと字の寸法。

use nene_ledger_core::chrome_hit::ChromeHit;
use nene_ledger_core::glyph_metrics::GlyphMetrics;
use nene_ledger_core::pixel_point::PixelPoint;
use nene_ledger_core::pixel_size::PixelSize;
use nene_ledger_core::window_chrome::WindowChrome;
use nene_ledger_core::window_placement::WindowPlacement;

/// 今の窓の大きさ・字の寸法・置かれ方。描くとき・意図を渡すとき・当たり判定に使う。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Viewport {
    size: PixelSize,
    metrics: GlyphMetrics,
    placement: WindowPlacement,
}

impl Viewport {
    pub(crate) fn new(size: PixelSize, metrics: GlyphMetrics, placement: WindowPlacement) -> Self {
        Self {
            size,
            metrics,
            placement,
        }
    }

    pub(crate) fn placement(self) -> WindowPlacement {
        self.placement
    }

    /// 置かれ方を替えた窓。
    pub(crate) fn placed(self, placement: WindowPlacement) -> Self {
        Self { placement, ..self }
    }

    /// 窓の中の位置 `(x, y)` が当たる所（core の当たり判定）。窓の外（負の位置）は中身とみなす。
    pub(crate) fn hit(self, x: i16, y: i16) -> ChromeHit {
        let (Ok(x), Ok(y)) = (u32::try_from(x), u32::try_from(y)) else {
            return ChromeHit::Client;
        };
        WindowChrome::new(self.size, self.metrics, self.placement).hit(PixelPoint::new(x, y))
    }

    pub(crate) fn size(self) -> PixelSize {
        self.size
    }

    pub(crate) fn metrics(self) -> GlyphMetrics {
        self.metrics
    }

    /// 大きさを替えた窓。
    pub(crate) fn resized(self, size: PixelSize) -> Self {
        Self { size, ..self }
    }
}
