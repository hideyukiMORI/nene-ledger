//! 物理画素の大きさ。

/// 物理画素の大きさ（幅と高さ）。窓の大きさ・塗る矩形の大きさに使う。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PixelSize {
    width: u32,
    height: u32,
}

impl PixelSize {
    /// 大きさを作る。
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self::new_const(width, height)
    }

    /// 大きさを作る（定数の式で使える形）。
    #[must_use]
    pub const fn new_const(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// 幅。
    #[must_use]
    pub fn width(self) -> u32 {
        self.width
    }

    /// 高さ。
    #[must_use]
    pub fn height(self) -> u32 {
        self.height
    }
}
