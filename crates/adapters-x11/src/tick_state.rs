//! 時計の刻みの状態。

/// 時計の刻みの状態。刻んでいるか・終わるか・まだ受け取られていない刻みの数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TickState {
    running: bool,
    quit: bool,
    ticks: u64,
}

impl TickState {
    /// 止めた状態。
    pub(crate) fn stopped() -> Self {
        Self {
            running: false,
            quit: false,
            ticks: 0,
        }
    }

    /// 刻んでいるか。
    pub(crate) fn running(self) -> bool {
        self.running
    }

    /// 終わるか。
    pub(crate) fn quit(self) -> bool {
        self.quit
    }

    /// 刻み始める。もともと刻んでいたら `false`。
    pub(crate) fn start(&mut self) -> bool {
        let started = !self.running;
        self.running = true;
        started
    }

    /// 止めて、数えた刻みを捨てる。
    pub(crate) fn stop(&mut self) {
        self.running = false;
        self.ticks = 0;
    }

    /// 終わらせる。
    pub(crate) fn finish(&mut self) {
        self.quit = true;
    }

    /// 刻んでいれば 1 つ数える。数えたら `true`。
    pub(crate) fn tick(&mut self) -> bool {
        if self.running {
            self.ticks = self.ticks.saturating_add(1);
        }
        self.running
    }

    /// 数えた刻みを受け取る。
    pub(crate) fn take(&mut self) -> u64 {
        core::mem::replace(&mut self.ticks, 0)
    }
}
