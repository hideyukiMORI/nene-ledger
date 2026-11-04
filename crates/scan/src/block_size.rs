//! 一度に読む塊の大きさ。

use core::num::NonZeroUsize;

/// 一度に読む塊のバイト数。0 にはならない（ADR 0017 の決定 1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockSize(NonZeroUsize);

impl BlockSize {
    /// 初期値の 1 MiB（ADR 0017 の決定 1）。
    pub const MEBIBYTE: Self = Self(NonZeroUsize::MIN.saturating_add(1_048_575_usize));

    /// 塊の大きさを作る。0 なら `None`。
    #[must_use]
    pub fn new(bytes: usize) -> Option<Self> {
        NonZeroUsize::new(bytes).map(Self)
    }

    /// バイト数。
    #[must_use]
    pub fn get(self) -> usize {
        self.0.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_zero_and_defaults_to_one_mebibyte() {
        assert_eq!(BlockSize::new(0), None);
        assert_eq!(BlockSize::new(7).map(BlockSize::get), Some(7));
        assert_eq!(BlockSize::MEBIBYTE.get(), 1024 * 1024);
    }
}
