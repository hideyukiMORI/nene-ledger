//! 一度に読む塊の大きさ。

use core::num::NonZeroUsize;

/// 一度に読む塊のバイト数。0 にはならない（ADR 0017 の決定 1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockSize(NonZeroUsize);

impl BlockSize {
    /// 初期値の 1 MiB（ADR 0017 の決定 1）。
    pub const MEBIBYTE: Self = Self(NonZeroUsize::MIN.saturating_add(1_048_575_usize));

    /// 64 KiB。範囲の取り出しが境目を探す最初の塊（窓の 1,024 行の多くはこれに収まる・GitHub Issue #30）。
    pub const SIXTY_FOUR_KIBIBYTES: Self = Self(NonZeroUsize::MIN.saturating_add(65_535_usize));

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

    /// 倍の大きさ。`most` を超えない。
    #[must_use]
    pub fn doubled_within(self, most: Self) -> Self {
        Self(
            self.0
                .saturating_mul(NonZeroUsize::MIN.saturating_add(1))
                .min(most.0),
        )
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
        assert_eq!(BlockSize::SIXTY_FOUR_KIBIBYTES.get(), 64 * 1024);
    }

    #[test]
    fn doubles_up_to_the_ceiling() {
        let doubled = BlockSize::SIXTY_FOUR_KIBIBYTES.doubled_within(BlockSize::MEBIBYTE);
        assert_eq!(doubled.get(), 128 * 1024);
        let capped = BlockSize::MEBIBYTE.doubled_within(BlockSize::MEBIBYTE);
        assert_eq!(capped, BlockSize::MEBIBYTE);
    }
}
