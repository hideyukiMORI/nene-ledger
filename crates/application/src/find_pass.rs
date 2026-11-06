//! 裏の検索の 1 回分（ADR 0030 の決定 4）。

use crate::find_bound::FindBound;

/// 原本のレコード `first` から `count` 個を、境 `bound` の向きで探す 1 回分。合成ルートが裏の仕事で探し、当たった
/// レコードの番号と表示の列を `Document::found` に返す。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FindPass {
    first: u64,
    count: u64,
    bound: FindBound,
}

impl FindPass {
    /// レコード `first` から `count` 個を `bound` で探す。
    #[must_use]
    pub const fn new(first: u64, count: u64, bound: FindBound) -> Self {
        Self {
            first,
            count,
            bound,
        }
    }

    /// 最初のレコードの番号。
    #[must_use]
    pub const fn first(self) -> u64 {
        self.first
    }

    /// レコードの数（終わりが分からなければ `u64::MAX`）。
    #[must_use]
    pub const fn count(self) -> u64 {
        self.count
    }

    /// 向きと境。
    #[must_use]
    pub const fn bound(self) -> FindBound {
        self.bound
    }
}
