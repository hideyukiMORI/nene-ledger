//! 1 つのセルの位置。

use crate::byte_range::ByteRange;

/// 1 つのセルの、原本の中の位置。値は文字列のまま（KEEP-01）で、引用符とエスケープを含む元のバイト列を指す。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Field {
    raw: ByteRange,
    quoted: bool,
}

impl Field {
    /// セルを作る。`raw` は引用符を含む元の区間。
    #[must_use]
    pub fn new(raw: ByteRange, quoted: bool) -> Self {
        Self { raw, quoted }
    }

    /// 引用符とエスケープを含む元の区間。
    #[must_use]
    pub fn raw(self) -> ByteRange {
        self.raw
    }

    /// 引用符で始まったか（元の書き方を保つために覚える）。
    #[must_use]
    pub fn quoted(self) -> bool {
        self.quoted
    }
}
