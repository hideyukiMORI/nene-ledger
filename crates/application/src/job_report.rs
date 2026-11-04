//! 一括処理からの知らせ。

use crate::index_summary::IndexSummary;
use crate::job_progress::JobProgress;

/// 作業のスレッドから届く知らせ。`JobState::apply` が受け取る（ADR 0014 の決定 1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobReport {
    /// 進んだ。
    Progress(JobProgress),
    /// 終わった。
    Finished(IndexSummary),
    /// 失敗した（読めなかった・作業のスレッドが落ちた）。
    Failed,
    /// 中止した。
    Cancelled,
}
