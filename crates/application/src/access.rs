//! 開いている表を変えてよいか（EDIT-13・GitHub Issue #238）。

/// 開いている表を変えてよいか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Access {
    /// 直せる（既定）。
    Writable,
    /// 読み取り専用（表の値・行・列を変える操作を止める）。
    ReadOnly,
}

impl Access {
    /// 切り替えた後。
    #[must_use]
    pub(crate) const fn toggled(self) -> Self {
        match self {
            Self::Writable => Self::ReadOnly,
            Self::ReadOnly => Self::Writable,
        }
    }
}
