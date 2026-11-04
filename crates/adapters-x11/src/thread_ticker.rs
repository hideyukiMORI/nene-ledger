//! 裏のスレッドで刻む時計。

use std::io;
use std::sync::Arc;
use std::thread::{Builder, JoinHandle};
use std::time::Duration;

use nene_ledger_application::ticker::Ticker;
use nene_ledger_application::wakeup::Wakeup;

use crate::tick_shared::TickShared;

/// 刻みの間隔（Issue #44。application の繰り返しの待ち `REPEAT_DELAY_TICKS` はこの 10 刻み）。
const TICK: Duration = Duration::from_millis(50);
/// 時計のスレッドの名前。
const THREAD_NAME: &str = "nene-ledger-ticker";

/// 裏のスレッドで、刻み始めてから止めるまで `interval` ごとに刻みを数えて画面のスレッドを起こす時計（`Ticker` の実装）。
/// 止めている間は眠って待つ。手放すと止めて合流する。
#[derive(Debug)]
pub struct ThreadTicker {
    shared: Arc<TickShared>,
    worker: Option<JoinHandle<()>>,
}

impl ThreadTicker {
    /// 50ms ごとに `wakeup` で起こす時計を作る（止めた状態で始まる）。
    ///
    /// # Errors
    ///
    /// スレッドを作れないとき。
    pub fn spawn(wakeup: Arc<dyn Wakeup>) -> io::Result<Self> {
        Self::spawn_every(wakeup, TICK)
    }

    /// `interval` ごとに起こす時計を作る（テストは短い間隔で）。
    fn spawn_every(wakeup: Arc<dyn Wakeup>, interval: Duration) -> io::Result<Self> {
        let shared = Arc::new(TickShared::new());
        let worker_shared = Arc::clone(&shared);
        let worker = Builder::new()
            .name(String::from(THREAD_NAME))
            .spawn(move || run(&worker_shared, wakeup.as_ref(), interval))?;
        Ok(Self {
            shared,
            worker: Some(worker),
        })
    }
}

/// 時計のスレッド。刻んでいる間は `interval` 眠るたびに 1 つ数えて起こし、止めている間は合図を待つ。
fn run(shared: &TickShared, wakeup: &dyn Wakeup, interval: Duration) {
    while shared.wait_until_running() {
        std::thread::sleep(interval);
        let ticked = shared.lock().tick();
        if ticked {
            wakeup.wake();
        }
    }
}

impl Ticker for ThreadTicker {
    fn start(&self) {
        if self.shared.lock().start() {
            self.shared.notify();
        }
    }

    fn stop(&self) {
        self.shared.lock().stop();
    }

    fn take(&self) -> u64 {
        self.shared.lock().take()
    }
}

impl Drop for ThreadTicker {
    fn drop(&mut self) {
        self.shared.lock().finish();
        self.shared.notify();
        if let Some(worker) = self.worker.take() {
            // 時計のスレッドは panic しない（落ちていても捨てるだけ）。
            let _joined = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Debug, Default)]
    struct Counting(AtomicUsize);

    impl Wakeup for Counting {
        fn wake(&self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn ticks_only_while_started_and_counts_what_was_not_taken() {
        let counting = Arc::new(Counting::default());
        let ticker = ThreadTicker::spawn_every(counting.clone(), Duration::from_millis(1)).unwrap();
        std::thread::sleep(Duration::from_millis(20));
        assert_eq!((ticker.take(), counting.0.load(Ordering::SeqCst)), (0, 0));
        ticker.start();
        while counting.0.load(Ordering::SeqCst) < 3 {
            std::thread::yield_now();
        }
        assert!(ticker.take() >= 3);
        ticker.stop();
        assert_eq!(ticker.take(), 0);
        drop(ticker);
    }
}
