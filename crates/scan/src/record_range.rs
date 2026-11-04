//! 求めるレコードの範囲。

/// 求めるレコードの範囲。`first` から `count` 件（レコードの番号は 0 始まり）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordRange {
    first: u64,
    count: usize,
}

impl RecordRange {
    /// 範囲を作る。
    #[must_use]
    pub fn new(first: u64, count: usize) -> Self {
        Self { first, count }
    }

    /// 最初のレコードの番号。
    #[must_use]
    pub fn first(self) -> u64 {
        self.first
    }

    /// 件数。
    #[must_use]
    pub fn count(self) -> usize {
        self.count
    }

    /// 先頭から `size` 個ずつに分けた範囲（最後は残りだけ）。`size` が 0 なら 1 個ずつ。
    pub fn chunks(self, size: usize) -> impl Iterator<Item = Self> {
        let size = size.max(1);
        (0..self.count.div_ceil(size)).map(move |index| {
            let done = index.saturating_mul(size);
            let first = self
                .first
                .saturating_add(u64::try_from(done).unwrap_or(u64::MAX));
            Self::new(first, size.min(self.count.saturating_sub(done)))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_range_is_split_into_chunks_with_the_rest_last() {
        let chunks: Vec<RecordRange> = RecordRange::new(10, 7).chunks(3).collect();
        assert_eq!(
            chunks,
            vec![
                RecordRange::new(10, 3),
                RecordRange::new(13, 3),
                RecordRange::new(16, 1)
            ]
        );
        assert_eq!(RecordRange::new(0, 0).chunks(3).count(), 0);
        assert_eq!(RecordRange::new(5, 2).chunks(0).count(), 2);
    }
}
