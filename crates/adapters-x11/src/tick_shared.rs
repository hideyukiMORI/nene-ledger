//! 画面のスレッドと時計のスレッドが分け合う状態。

use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};

use crate::tick_state::TickState;

/// 画面のスレッドと時計のスレッドが分け合う状態と、変わったことの合図。
#[derive(Debug)]
pub(crate) struct TickShared {
    state: Mutex<TickState>,
    changed: Condvar,
}

impl TickShared {
    /// 止めた状態。
    pub(crate) fn new() -> Self {
        Self {
            state: Mutex::new(TickState::stopped()),
            changed: Condvar::new(),
        }
    }

    /// 状態を手に取る。
    pub(crate) fn lock(&self) -> MutexGuard<'_, TickState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// 変わったことを知らせる。
    pub(crate) fn notify(&self) {
        self.changed.notify_all();
    }

    /// 刻み始めるか終わるまで待つ。終わるなら `false`。
    pub(crate) fn wait_until_running(&self) -> bool {
        let mut state = self.lock();
        while !state.running() && !state.quit() {
            state = self
                .changed
                .wait(state)
                .unwrap_or_else(PoisonError::into_inner);
        }
        !state.quit()
    }
}
