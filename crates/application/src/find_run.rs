//! 進んでいる検索（ADR 0030 の決定 1・4）。

use crate::find_pass::FindPass;
use crate::find_scope::FindScope;

/// 進んでいる検索。探す問い（`scope`）と向き、始めたセル（行の並びの位置・表示の列）、反対の端から続けているか、
/// 裏で探している 1 回分（合成ルートが取りに来るまで `pending`）、進み具合。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FindRun {
    scope: FindScope,
    forward: bool,
    start: (u64, usize),
    wrapped: bool,
    pending: Option<FindPass>,
    progress: Option<(u64, u64)>,
}

impl FindRun {
    /// `start` のセルから `forward` の向きに `scope` を探し始める。
    pub(crate) const fn new(scope: FindScope, forward: bool, start: (u64, usize)) -> Self {
        Self {
            scope,
            forward,
            start,
            wrapped: false,
            pending: None,
            progress: None,
        }
    }

    pub(crate) const fn scope(&self) -> &FindScope {
        &self.scope
    }

    pub(crate) const fn forward(&self) -> bool {
        self.forward
    }

    pub(crate) const fn start(&self) -> (u64, usize) {
        self.start
    }

    pub(crate) const fn wrapped(&self) -> bool {
        self.wrapped
    }

    /// 反対の端から続ける。
    pub(crate) fn wrap(&mut self) {
        self.wrapped = true;
        self.progress = None;
    }

    /// 裏で探す 1 回分を置く。
    pub(crate) fn set_pending(&mut self, pass: FindPass) {
        self.pending = Some(pass);
    }

    /// 裏で探す 1 回分を取り出す（合成ルートが裏の仕事を始める）。
    pub(crate) fn take_pending(&mut self) -> Option<FindPass> {
        self.pending.take()
    }

    /// 進み具合（読んだレコードの数・読む見込み）。
    pub(crate) fn set_progress(&mut self, done: u64, total: u64) {
        self.progress = Some((done, total));
    }

    pub(crate) const fn progress(&self) -> Option<(u64, u64)> {
        self.progress
    }
}
