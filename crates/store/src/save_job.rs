//! 裏のスレッドで保存する仕事（ADR 0026 の決定 6・ADR 0014）。
//!
//! 計画（`SavePlan`）は呼ぶ側が作って渡す。ここは書いて置き換える手順（`saving::save`）を裏で回すだけ。

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread::{Builder, JoinHandle};

use nene_ledger_application::replacing::Replacing;
use nene_ledger_application::save_target::SaveTarget;
use nene_ledger_application::source_bytes::SourceBytes;
use nene_ledger_application::wakeup::Wakeup;
use nene_ledger_core::plan_sink::PlanSink;
use nene_ledger_core::save_plan::SavePlan;
use nene_ledger_core::text_encoding::TextEncoding;

use crate::save_failure::SaveFailure;
use crate::saving::{save, save_streamed};

/// 裏のスレッドの名前。
const THREAD_NAME: &str = "nene-ledger-save";

type Outcome = Result<(), SaveFailure>;

/// 裏で回っている保存。
///
/// 🔴 途中で止めない。捨てるとき（窓を閉じたとき）は、終わるまで待つ（置き換えの途中で手を離さない）。
#[derive(Debug)]
pub struct SaveJob {
    worker: Option<JoinHandle<Outcome>>,
}

impl SaveJob {
    /// `target` へ、計画 `plan` を文字コード `encoding` で書き、保存先が書く前の様子 `expected` のままか確かめてから
    /// 置き換える（まだ無ければ作る）仕事を始める。
    /// 原本は `source` で読み、置き換えは `replacing`。終わったら `wakeup` で窓を起こす。
    ///
    /// # Errors
    ///
    /// スレッドを作れないとき（`SaveFailure::Spawn`）。
    pub fn spawn<S, R>(
        target: PathBuf,
        (plan, encoding, expected): (SavePlan, TextEncoding, SaveTarget),
        (mut source, replacing): (S, R),
        wakeup: Arc<dyn Wakeup>,
    ) -> Result<Self, SaveFailure>
    where
        S: SourceBytes<Failure = ErrorKind> + Send + 'static,
        R: Replacing<Location = Path> + Send + 'static,
    {
        let worker = Builder::new()
            .name(String::from(THREAD_NAME))
            .spawn(move || {
                let outcome = save(
                    &target,
                    (&plan, encoding, expected),
                    &mut source,
                    &replacing,
                );
                wakeup.wake();
                outcome
            })
            .map_err(|error| SaveFailure::Spawn(error.kind()))?;
        Ok(Self {
            worker: Some(worker),
        })
    }

    /// 計画を記憶に持たずに書く仕事を始める（GitHub Issue #130）。計画を作ること（`produce`）も裏のスレッドで行い、
    /// 歩は一時ファイルに直に書く（`saving::save_streamed`）。画面のスレッドは止まらない。
    ///
    /// # Errors
    ///
    /// スレッドを作れないとき（`SaveFailure::Spawn`）。
    pub fn spawn_streamed<S, R, P>(
        target: PathBuf,
        (encoding, expected): (TextEncoding, SaveTarget),
        (mut source, replacing, produce): (S, R, P),
        wakeup: Arc<dyn Wakeup>,
    ) -> Result<Self, SaveFailure>
    where
        S: SourceBytes<Failure = ErrorKind> + Send + 'static,
        R: Replacing<Location = Path> + Send + 'static,
        P: FnOnce(&mut dyn PlanSink) -> Result<(), SaveFailure> + Send + 'static,
    {
        let worker = Builder::new()
            .name(String::from(THREAD_NAME))
            .spawn(move || {
                let outcome = save_streamed(
                    &target,
                    (encoding, expected),
                    (&mut source, &replacing),
                    produce,
                );
                wakeup.wake();
                outcome
            })
            .map_err(|error| SaveFailure::Spawn(error.kind()))?;
        Ok(Self {
            worker: Some(worker),
        })
    }

    /// 終わっていれば結果（1 度だけ）。まだなら `None`。
    pub fn take_result(&mut self) -> Option<Outcome> {
        let finished = self.worker.as_ref().is_some_and(JoinHandle::is_finished);
        if !finished {
            return None;
        }
        self.worker.take().map(join)
    }

    /// 終わるまで待って結果を返す。
    ///
    /// # Errors
    ///
    /// 保存できなかったとき。
    pub fn wait(mut self) -> Outcome {
        self.worker.take().map_or(Err(SaveFailure::Panicked), join)
    }
}

impl Drop for SaveJob {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.take() {
            // 結果は捨てるが、置き換えが終わるまでは待つ。
            let _finished = join(worker);
        }
    }
}

fn join(worker: JoinHandle<Outcome>) -> Outcome {
    worker.join().unwrap_or(Err(SaveFailure::Panicked))
}
