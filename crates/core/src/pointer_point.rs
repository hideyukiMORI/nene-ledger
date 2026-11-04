//! 窓の外にも出うる指の位置。

/// 窓の左上を原点とする指の位置。ボタンを押したまま窓の外へ出ると負にもなる（ドラッグ）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PointerPoint {
    x: i32,
    y: i32,
}

impl PointerPoint {
    /// 位置 `(x, y)`。
    #[must_use]
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// 横の位置。
    #[must_use]
    pub fn x(self) -> i32 {
        self.x
    }

    /// 縦の位置。
    #[must_use]
    pub fn y(self) -> i32 {
        self.y
    }
}
