//! 溝に描く印の位置。

use alloc::vec::Vec;

/// 溝に描く印の位置（尺度の単位・縦は本体の行、横は列の画素）。選んだ範囲ごとの始まりと終わり（終わりは含まない）と、
/// カーソルの位置 `cursor`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarMarks {
    ranges: Vec<(u64, u64)>,
    cursor: u64,
}

impl BarMarks {
    /// 選んだ範囲ごとの `ranges`（始まり・終わりは含まない）とカーソルの位置 `cursor`。
    #[must_use]
    pub fn new(ranges: Vec<(u64, u64)>, cursor: u64) -> Self {
        Self { ranges, cursor }
    }

    /// 選んだ範囲ごとの始まりと終わり。
    #[must_use]
    pub fn ranges(&self) -> &[(u64, u64)] {
        &self.ranges
    }

    /// カーソルの位置。
    #[must_use]
    pub fn cursor(&self) -> u64 {
        self.cursor
    }
}
