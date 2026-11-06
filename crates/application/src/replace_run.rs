//! 進んでいる全部の置換（ADR 0030 の決定 6）。

use alloc::string::String;
use alloc::vec::Vec;

use nene_ledger_core::cell_change::CellChange;

use crate::find_scope::FindScope;

/// 進んでいる全部の置換。探す問い（`scope`）と置き換える文字、数え始めたときの表の版、裏で数える仕事を合成ルートが
/// 取ったか、進み具合、数え終えた書き換え（セルの書き換えと、そのセルのファイルの今の値）と飛ばしたレコードの数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReplaceRun {
    scope: FindScope,
    replacement: String,
    revision: u64,
    taken: bool,
    progress: Option<(u64, u64)>,
    counted: Option<(Vec<(CellChange, String)>, u64)>,
}

impl ReplaceRun {
    /// 表の版 `revision` で、`scope` の当たりを `replacement` に置き換える数えを始める。
    pub(crate) const fn new(scope: FindScope, replacement: String, revision: u64) -> Self {
        Self {
            scope,
            replacement,
            revision,
            taken: false,
            progress: None,
            counted: None,
        }
    }

    pub(crate) const fn scope(&self) -> &FindScope {
        &self.scope
    }

    pub(crate) fn replacement(&self) -> &str {
        &self.replacement
    }

    pub(crate) const fn revision(&self) -> u64 {
        self.revision
    }

    /// 裏の仕事を合成ルートに渡す（1 度だけ・渡したら `true` を返さない）。
    pub(crate) fn take(&mut self) -> bool {
        let first = !self.taken;
        self.taken = true;
        first
    }

    pub(crate) const fn progress(&self) -> Option<(u64, u64)> {
        self.progress
    }

    pub(crate) fn set_progress(&mut self, done: u64, total: u64) {
        self.progress = Some((done, total));
    }

    /// 数え終えた書き換えと、飛ばしたレコードの数（件数を見せている間）。
    pub(crate) fn counted(&self) -> Option<&(Vec<(CellChange, String)>, u64)> {
        self.counted.as_ref()
    }

    pub(crate) fn set_counted(&mut self, changes: Vec<(CellChange, String)>, skipped: u64) {
        self.counted = Some((changes, skipped));
    }

    /// 数え終えた書き換えを取り出す（置き換えるとき）。
    pub(crate) fn take_counted(&mut self) -> Option<(Vec<(CellChange, String)>, u64)> {
        self.counted.take()
    }
}
