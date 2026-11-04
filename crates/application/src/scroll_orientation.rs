//! スクロールバーの向き。

/// スクロールバーの向き。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScrollOrientation {
    /// 縦（表の右端・本体の行）。
    Vertical,
    /// 横（表の下端・列の画素）。
    Horizontal,
}
