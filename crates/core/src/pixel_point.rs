//! 物理画素の位置。

/// 窓の左上を原点とする物理画素の位置。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PixelPoint {
    x: u32,
    y: u32,
}

impl PixelPoint {
    /// 位置を作る。
    #[must_use]
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    /// 横の位置。
    #[must_use]
    pub fn x(self) -> u32 {
        self.x
    }

    /// 縦の位置。
    #[must_use]
    pub fn y(self) -> u32 {
        self.y
    }
}
