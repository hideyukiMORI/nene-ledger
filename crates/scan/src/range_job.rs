//! 裏のスレッドで範囲のレコードを畳み込む仕事。

use core::ops::ControlFlow;
use std::sync::Arc;
use std::thread::{Builder, JoinHandle};

use nene_ledger_application::wakeup::Wakeup;

use crate::indexed_source::IndexedSource;
use crate::job_control::JobControl;
use crate::progress::Progress;
use crate::range_folding::fold_range;
use crate::range_outcome::RangeOutcome;
use crate::record_fold::RecordFold;
use crate::record_range::RecordRange;
use crate::scan_failure::ScanFailure;

/// 作業のスレッドの名前。
const THREAD_NAME: &str = "nene-ledger-range";
/// 一度に読むレコードの数。中止はこの塊の終わりで効く。
const CHUNK_RECORDS: usize = 4_096;

/// 作業のスレッドの結果。
type Outcome<F> = Result<RangeOutcome<F>, ScanFailure>;

/// 裏のスレッドで範囲のレコードを畳み込む仕事（ADR 0014）。塊ごとに進み具合（畳み込んだレコードの数）を残し、
/// `Wakeup` で画面のスレッドを起こす。手放すと中止して合流する（`IndexJob` と同じ）。
#[derive(Debug)]
pub struct RangeJob<F: RecordFold + 'static> {
    control: Arc<JobControl>,
    worker: Option<JoinHandle<Outcome<F>>>,
}

impl<F: RecordFold + 'static> RangeJob<F> {
    /// `from` の `range` を `fold` に畳み込む仕事を、裏のスレッドで始める。
    ///
    /// # Errors
    ///
    /// 作業のスレッドを作れないとき。
    pub fn spawn(
        from: IndexedSource,
        range: RecordRange,
        mut fold: F,
        wakeup: Arc<dyn Wakeup>,
    ) -> Result<Self, ScanFailure> {
        let total = u64::try_from(range.count())?;
        let control = Arc::new(JobControl::new(total));
        let shared = Arc::clone(&control);
        let worker = Builder::new()
            .name(String::from(THREAD_NAME))
            .spawn(move || {
                let flow = fold_range(&from, range.chunks(CHUNK_RECORDS), &mut fold, |done| {
                    let flow = shared.report(Progress::new(done, total, None));
                    wakeup.wake();
                    flow
                });
                wakeup.wake();
                match flow? {
                    ControlFlow::Continue(()) => Ok(RangeOutcome::Finished(fold)),
                    ControlFlow::Break(()) => Ok(RangeOutcome::Cancelled),
                }
            })
            .map_err(|error| ScanFailure::Spawn(error.kind()))?;
        Ok(Self {
            control,
            worker: Some(worker),
        })
    }

    /// 最後に知らされた進み具合（畳み込んだレコードの数 / 範囲のレコードの数）。
    #[must_use]
    pub fn progress(&self) -> Progress {
        self.control.progress()
    }

    /// 中止を求める。作業のスレッドは次の塊の終わりで止まる。
    pub fn cancel(&self) {
        self.control.cancel();
    }

    /// 終わっていれば結果を受け取る（1 度だけ）。まだ走っていれば `None`。待たない。
    pub fn take_result(&mut self) -> Option<Outcome<F>> {
        let finished = self.worker.as_ref().is_some_and(JoinHandle::is_finished);
        if !finished {
            return None;
        }
        self.worker.take().map(join)
    }

    /// 終わるまで待って結果を受け取る。
    ///
    /// # Errors
    ///
    /// 読み出しの失敗・作業のスレッドが落ちた・結果を受け取り済み。
    pub fn wait(mut self) -> Outcome<F> {
        self.worker
            .take()
            .map_or(Err(ScanFailure::WorkerPanicked), join)
    }
}

impl<F: RecordFold + 'static> Drop for RangeJob<F> {
    fn drop(&mut self) {
        self.control.cancel();
        if let Some(worker) = self.worker.take() {
            // 結果は捨てる（中止した処理の途中の結果は使わない・ADR 0014 の決定 4）。
            let _discarded = join(worker);
        }
    }
}

/// 作業のスレッドと合流する。panic で落ちていたら失敗として受け止める。
fn join<F>(worker: JoinHandle<Outcome<F>>) -> Outcome<F> {
    worker.join().unwrap_or(Err(ScanFailure::WorkerPanicked))
}
