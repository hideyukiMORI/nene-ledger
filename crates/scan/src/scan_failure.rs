//! 原本を読めなかった理由。

use core::num::TryFromIntError;
use std::io::ErrorKind;

/// 原本を読めなかった理由。閉じた集合（ARC-010）。中止は失敗ではない（`IndexOutcome`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanFailure {
    /// 開けなかった（無い・権限が無い・ロックされている）。
    Open(ErrorKind),
    /// `offset` から読めなかった。
    Read {
        /// 読もうとしたバイト位置。
        offset: u64,
        /// OS が返した理由。
        kind: ErrorKind,
    },
    /// バイト位置が `usize` と `u64` の間で収まらなかった。
    OutOfRange(TryFromIntError),
    /// 作業のスレッドを作れなかった。
    Spawn(ErrorKind),
    /// 作業のスレッドが panic で落ちた。全体は止めず、失敗として受け止める（ADR 0014 の残る穴）。
    WorkerPanicked,
}

impl From<TryFromIntError> for ScanFailure {
    fn from(error: TryFromIntError) -> Self {
        Self::OutOfRange(error)
    }
}
