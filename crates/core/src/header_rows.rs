//! 見出しの行の範囲（ADR 0034）。

use core::ops::Range;

/// 見出しの行の範囲。行の並びの位置（またはレコードの番号）の `first` から続く `count` 行（1〜[`HeaderRows::MAX`]）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HeaderRows {
    first: u64,
    count: u64,
}

impl HeaderRows {
    /// 見出しにできる行の数の上限。
    pub const MAX: u64 = 3;

    /// `first` から `count` 行。`count` が 1〜[`Self::MAX`] でないか、終わりが `u64` を越えれば `None`。
    #[must_use]
    pub fn new(first: u64, count: u64) -> Option<Self> {
        if count == 0 || count > Self::MAX {
            return None;
        }
        first.checked_add(count)?;
        Some(Self { first, count })
    }

    /// 1 行の見出し。
    #[must_use]
    pub const fn single(first: u64) -> Self {
        Self { first, count: 1 }
    }

    /// 最初の行。
    #[must_use]
    pub const fn first(self) -> u64 {
        self.first
    }

    /// 行の数（1〜[`Self::MAX`]）。
    #[must_use]
    pub const fn count(self) -> u64 {
        self.count
    }

    /// 最後の行の次。
    #[must_use]
    pub const fn end(self) -> u64 {
        self.first.saturating_add(self.count)
    }

    /// 行の範囲。
    #[must_use]
    pub const fn lines(self) -> Range<u64> {
        self.first..self.end()
    }

    /// 行 `line` が見出しか。
    #[must_use]
    pub const fn contains(self, line: u64) -> bool {
        line >= self.first && line < self.end()
    }

    /// 最初の行を `first` にした同じ数の見出し（行を足し消しして見出しが動いたとき）。終わりが `u64` を越えれば `None`。
    #[must_use]
    pub fn at(self, first: u64) -> Option<Self> {
        Self::new(first, self.count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn only_one_to_three_rows_make_a_header() {
        assert_eq!(HeaderRows::new(2, 0), None);
        assert_eq!(HeaderRows::new(2, 4), None);
        assert_eq!(HeaderRows::new(u64::MAX, 1), None);
        assert_eq!(HeaderRows::single(7), HeaderRows::new(7, 1).unwrap());
    }

    #[test]
    fn a_header_knows_its_lines() {
        let header = HeaderRows::new(2, 3).unwrap();
        assert_eq!((header.first(), header.count(), header.end()), (2, 3, 5));
        assert_eq!(header.lines(), 2..5);
        assert_eq!(
            (0..7).map(|line| header.contains(line)).collect::<Vec<_>>(),
            [false, false, true, true, true, false, false]
        );
        assert_eq!(header.at(9).map(HeaderRows::lines), Some(9..12));
        assert_eq!(header.at(u64::MAX - 1), None);
    }
}
