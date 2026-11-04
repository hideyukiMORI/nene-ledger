//! 物理画素の矩形。

use crate::pixel_point::PixelPoint;
use crate::pixel_size::PixelSize;

/// 物理画素の矩形（左上と大きさ）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PixelRect {
    origin: PixelPoint,
    size: PixelSize,
}

impl PixelRect {
    /// 矩形を作る。
    #[must_use]
    pub fn new(origin: PixelPoint, size: PixelSize) -> Self {
        Self { origin, size }
    }

    /// 左上。
    #[must_use]
    pub fn origin(self) -> PixelPoint {
        self.origin
    }

    /// 大きさ。
    #[must_use]
    pub fn size(self) -> PixelSize {
        self.size
    }
}
