//! 色。

/// 不透明な色（sRGB の 8 ビット × 3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

impl Color {
    /// 色を作る。
    #[must_use]
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    /// 赤。
    #[must_use]
    pub fn red(self) -> u8 {
        self.red
    }

    /// 緑。
    #[must_use]
    pub fn green(self) -> u8 {
        self.green
    }

    /// 青。
    #[must_use]
    pub fn blue(self) -> u8 {
        self.blue
    }
}
