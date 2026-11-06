//! 進み具合を数え、中止を見る書き先（SAVE-02）。

use std::sync::Arc;

use nene_ledger_application::wakeup::Wakeup;
use nene_ledger_core::plan_sink::PlanSink;

use crate::save_control::SaveControl;

/// 大きな写す区間を分ける大きさ。分けるごとに中止を見て、画面を起こす（進み具合を描き直させる）。
const PIECE: u64 = 16 << 20;

/// 書き先 `inner` に歩を渡しながら、書いたバイト数を数え、中止を求められたら以後の歩を捨てる。
pub(crate) struct WatchedSink<'inner> {
    inner: &'inner mut dyn PlanSink,
    control: Arc<SaveControl>,
    wakeup: Arc<dyn Wakeup>,
    since_wake: u64,
}

impl<'inner> WatchedSink<'inner> {
    pub(crate) fn new(
        inner: &'inner mut dyn PlanSink,
        control: Arc<SaveControl>,
        wakeup: Arc<dyn Wakeup>,
    ) -> Self {
        Self {
            inner,
            control,
            wakeup,
            since_wake: 0,
        }
    }

    /// `bytes` を書いたと数え、[`PIECE`] ごとに画面を起こす。
    fn count(&mut self, bytes: u64) {
        self.control.add(bytes);
        self.since_wake = self.since_wake.saturating_add(bytes);
        if self.since_wake >= PIECE {
            self.since_wake = 0;
            self.wakeup.wake();
        }
    }
}

impl PlanSink for WatchedSink<'_> {
    fn copy(&mut self, mut offset: u64, mut length: u64) {
        while length > 0 && !self.stopped() {
            let piece = length.min(PIECE);
            self.inner.copy(offset, piece);
            self.count(piece);
            offset = offset.saturating_add(piece);
            length = length.saturating_sub(piece);
        }
    }

    fn text(&mut self, text: &str) {
        if self.stopped() {
            return;
        }
        self.inner.text(text);
        self.count(u64::try_from(text.len()).unwrap_or(u64::MAX));
    }

    fn stopped(&self) -> bool {
        self.control.is_cancelled() || self.inner.stopped()
    }
}
