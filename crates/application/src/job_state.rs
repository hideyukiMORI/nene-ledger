//! 一括処理の状態。

use crate::index_summary::IndexSummary;
use crate::job_progress::JobProgress;
use crate::job_report::JobReport;

/// 一括処理の状態。閉じた集合で、遷移は純関数（ADR 0014 の決定 1）。UI は描くだけ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    /// まだ知らせが無い。
    Waiting,
    /// 走っている。
    Running(JobProgress),
    /// 中止を求めた（作業のスレッドが次の塊で止まるのを待っている）。
    CancelRequested(JobProgress),
    /// 終わった。
    Finished(IndexSummary),
    /// 失敗した。
    Failed,
    /// 中止した。途中の結果は捨てた（ADR 0014 の決定 4）。
    Cancelled,
}

impl JobState {
    /// 知らせを受け取った後の状態。終わった・失敗した・中止した後の知らせは無視する。
    #[must_use]
    pub fn apply(self, report: JobReport) -> Self {
        if self.is_settled() {
            return self;
        }
        match report {
            JobReport::Progress(progress) => match self {
                Self::CancelRequested(_) => Self::CancelRequested(progress),
                Self::Waiting | Self::Running(_) => Self::Running(progress),
                Self::Finished(_) | Self::Failed | Self::Cancelled => self,
            },
            JobReport::Finished(summary) => Self::Finished(summary),
            JobReport::Failed => Self::Failed,
            JobReport::Cancelled => Self::Cancelled,
        }
    }

    /// 中止を求めた後の状態。走っていなければ変わらない。
    #[must_use]
    pub fn request_cancel(self) -> Self {
        match self {
            Self::Running(progress) => Self::CancelRequested(progress),
            Self::Waiting
            | Self::CancelRequested(_)
            | Self::Finished(_)
            | Self::Failed
            | Self::Cancelled => self,
        }
    }

    /// もう変わらないか（終わった・失敗した・中止した）。
    #[must_use]
    pub fn is_settled(self) -> bool {
        match self {
            Self::Finished(_) | Self::Failed | Self::Cancelled => true,
            Self::Waiting | Self::Running(_) | Self::CancelRequested(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn progress(done: u64) -> JobProgress {
        JobProgress::new(done, 100, None)
    }

    #[test]
    fn runs_then_finishes_and_ignores_later_reports() {
        let state = JobState::Waiting
            .apply(JobReport::Progress(progress(10)))
            .apply(JobReport::Progress(progress(60)));
        assert_eq!(state, JobState::Running(progress(60)));
        let done = state.apply(JobReport::Finished(IndexSummary::new(5, None)));
        assert_eq!(done, JobState::Finished(IndexSummary::new(5, None)));
        assert_eq!(done.apply(JobReport::Progress(progress(70))), done);
        assert_eq!(done.apply(JobReport::Failed), done);
        assert_eq!(done.request_cancel(), done);
    }

    #[test]
    fn a_cancel_request_keeps_counting_until_the_worker_stops() {
        let asked = JobState::Running(progress(20)).request_cancel();
        assert_eq!(asked, JobState::CancelRequested(progress(20)));
        assert_eq!(
            asked.apply(JobReport::Progress(progress(30))),
            JobState::CancelRequested(progress(30))
        );
        assert_eq!(asked.apply(JobReport::Cancelled), JobState::Cancelled);
        assert_eq!(JobState::Waiting.request_cancel(), JobState::Waiting);
        assert!(JobState::Failed.is_settled());
        assert!(!asked.is_settled());
    }

    #[test]
    fn percent_is_clamped() {
        assert_eq!(JobProgress::new(0, 0, None).percent(), 100);
        assert_eq!(JobProgress::new(45, 100, None).percent(), 45);
        assert_eq!(JobProgress::new(150, 100, None).percent(), 100);
        assert_eq!(JobProgress::new(u64::MAX, u64::MAX, None).percent(), 100);
    }
}
