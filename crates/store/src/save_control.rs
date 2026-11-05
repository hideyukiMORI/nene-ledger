//! 裏の保存の進み具合と中止の印（SAVE-02・ADR 0025 の施主の判断 4）。

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// 裏の保存の進み具合と中止の印。画面のスレッドと保存のスレッドで分け合う。
#[derive(Debug)]
pub(crate) struct SaveControl {
    cancelled: AtomicBool,
    written: AtomicU64,
}

impl SaveControl {
    pub(crate) fn new() -> Self {
        Self {
            cancelled: AtomicBool::new(false),
            written: AtomicU64::new(0),
        }
    }

    /// 中止を求める。
    pub(crate) fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    /// 中止を求められたか。
    pub(crate) fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    /// 書いたバイト数（文字列は符号化の前の UTF-8 の長さで数える・目安）を足す。
    pub(crate) fn add(&self, bytes: u64) {
        self.written.fetch_add(bytes, Ordering::Relaxed);
    }

    /// ここまでに書いたバイト数（目安）。
    pub(crate) fn written(&self) -> u64 {
        self.written.load(Ordering::Relaxed)
    }
}
