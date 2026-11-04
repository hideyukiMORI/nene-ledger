//! 裏のスレッドで索引を作る仕事。

use std::sync::Arc;
use std::thread::{Builder, JoinHandle};

use nene_ledger_application::wakeup::Wakeup;
use nene_ledger_core::text_format::TextFormat;

use crate::block_size::BlockSize;
use crate::index_outcome::IndexOutcome;
use crate::indexing::build_index;
use crate::job_control::JobControl;
use crate::progress::Progress;
use crate::scan_failure::ScanFailure;
use crate::source::Source;

/// 作業のスレッドの名前。
const THREAD_NAME: &str = "nene-ledger-index";

/// 作業のスレッドの結果。
type Outcome = Result<IndexOutcome, ScanFailure>;

/// 裏のスレッドで索引を作る仕事（ADR 0014 の決定 2〜4）。塊ごとに進み具合を残し、`Wakeup` で画面のスレッドを起こす。
///
/// 🔴 仕事は窓が開いている間ずっと生きるので `std::thread::scope` には入れられない。代わりに、手放すと中止して
/// 合流する（`Drop`）。スレッドが持ち主より長生きしないことは scope と同じ。
#[derive(Debug)]
pub struct IndexJob {
    control: Arc<JobControl>,
    worker: Option<JoinHandle<Outcome>>,
}

impl IndexJob {
    /// `source` を `format` で切る索引を、裏のスレッドで作り始める。
    ///
    /// # Errors
    ///
    /// 作業のスレッドを作れないとき。
    pub fn spawn(
        source: Source,
        format: TextFormat,
        size: BlockSize,
        wakeup: Arc<dyn Wakeup>,
    ) -> Result<Self, ScanFailure> {
        let control = Arc::new(JobControl::new(source.len()));
        let shared = Arc::clone(&control);
        let worker = Builder::new()
            .name(String::from(THREAD_NAME))
            .spawn(move || {
                let outcome = build_index(&source, format, size, |progress| {
                    let flow = shared.report(progress);
                    wakeup.wake();
                    flow
                });
                wakeup.wake();
                outcome
            })
            .map_err(|error| ScanFailure::Spawn(error.kind()))?;
        Ok(Self {
            control,
            worker: Some(worker),
        })
    }

    /// 最後に知らされた進み具合。
    #[must_use]
    pub fn progress(&self) -> Progress {
        self.control.progress()
    }

    /// 中止を求める。作業のスレッドは次の塊の終わりで止まる。
    pub fn cancel(&self) {
        self.control.cancel();
    }

    /// 終わっていれば結果を受け取る（1 度だけ）。まだ走っていれば `None`。待たない。
    pub fn take_result(&mut self) -> Option<Outcome> {
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
    pub fn wait(mut self) -> Outcome {
        self.worker
            .take()
            .map_or(Err(ScanFailure::WorkerPanicked), join)
    }
}

impl Drop for IndexJob {
    fn drop(&mut self) {
        self.control.cancel();
        if let Some(worker) = self.worker.take() {
            // 結果は捨てる（中止した処理の途中の結果は使わない・ADR 0014 の決定 4）。
            let _discarded = join(worker);
        }
    }
}

/// 作業のスレッドと合流する。panic で落ちていたら失敗として受け止める。
fn join(worker: JoinHandle<Outcome>) -> Outcome {
    worker.join().unwrap_or(Err(ScanFailure::WorkerPanicked))
}
