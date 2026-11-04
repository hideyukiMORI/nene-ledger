//! 詳細ペインの開閉。

/// 詳細ペインの開閉（F2 で切り替える）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailPane {
    /// 閉じている。
    Closed,
    /// 表と状態の行の間に開いている。
    Open,
}

impl DetailPane {
    /// 開閉を切り替えた状態。
    #[must_use]
    pub fn toggled(self) -> Self {
        match self {
            Self::Closed => Self::Open,
            Self::Open => Self::Closed,
        }
    }
}
