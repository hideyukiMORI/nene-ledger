//! 裏の処理と画面のスレッドの間の、中止の印と進み具合。

use core::ops::ControlFlow;
use core::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};

use crate::progress::Progress;

/// 裏の処理と画面のスレッドが共有する、中止の印と進み具合（ADR 0014 の決定 3・4）。
///
/// 作業のスレッドは塊ごとに `report` を呼び、画面のスレッドは `progress` を読み、`cancel` で止める。
#[derive(Debug)]
pub struct JobControl {
    cancelled: AtomicBool,
    latest: Mutex<Progress>,
}

impl JobControl {
    /// 中止されていない、進み具合 0 / `total` の印を作る。`total` は開いたときの大きさ。
    #[must_use]
    pub fn new(total: u64) -> Self {
        Self {
            cancelled: AtomicBool::new(false),
            latest: Mutex::new(Progress::new(0, total, None)),
        }
    }

    /// 中止を求める。作業のスレッドは次の塊の終わりで止まる。
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    /// 作業のスレッドが進み具合を知らせる。中止を求められていたら `Break`。
    pub fn report(&self, progress: Progress) -> ControlFlow<()> {
        // 進み具合は値を丸ごと置き換えるだけなので、別のスレッドが途中で落ちても中身は壊れていない。
        *self.latest.lock().unwrap_or_else(PoisonError::into_inner) = progress;
        if self.cancelled.load(Ordering::Acquire) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    /// 最後に知らされた進み具合。
    #[must_use]
    pub fn progress(&self) -> Progress {
        *self.latest.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
