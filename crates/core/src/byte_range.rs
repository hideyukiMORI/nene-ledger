//! 入力の中の半開区間。

/// 入力のバイト列の中の半開区間 `[start, end)`。`start <= end` を常に満たす。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteRange {
    start: usize,
    end: usize,
}

impl ByteRange {
    /// 区間を作る。`start > end` なら `None`。
    #[must_use]
    pub fn new(start: usize, end: usize) -> Option<Self> {
        (start <= end).then_some(Self { start, end })
    }

    /// 2 つの位置から区間を作る。小さいほうを始まりにする（呼び出し側で順序が保証されている所でだけ使う）。
    #[must_use]
    pub(crate) fn between(first: usize, second: usize) -> Self {
        Self {
            start: first.min(second),
            end: first.max(second),
        }
    }

    /// 始まりの位置。
    #[must_use]
    pub fn start(self) -> usize {
        self.start
    }

    /// 終わりの位置（含まない）。
    #[must_use]
    pub fn end(self) -> usize {
        self.end
    }

    /// 区間の長さ。
    #[must_use]
    pub fn len(self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// 空の区間か。
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// 入力からこの区間を切り出す。入力の外なら `None`。
    #[must_use]
    pub fn slice(self, input: &[u8]) -> Option<&[u8]> {
        input.get(self.start..self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_reversed_bounds() {
        assert!(ByteRange::new(3, 2).is_none());
        assert_eq!(ByteRange::new(2, 2).map(ByteRange::is_empty), Some(true));
    }

    #[test]
    fn between_orders_the_bounds() {
        let range = ByteRange::between(5, 1);
        assert_eq!((range.start(), range.end(), range.len()), (1, 5, 4));
    }

    #[test]
    fn slices_inside_and_refuses_outside() {
        let input = b"abcdef";
        assert_eq!(
            ByteRange::new(1, 3).and_then(|r| r.slice(input)),
            Some(&b"bc"[..])
        );
        assert_eq!(ByteRange::new(4, 9).and_then(|r| r.slice(input)), None);
    }
}
