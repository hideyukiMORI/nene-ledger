//! 切り取りの枠（ADR 0029 の決定 5）。

use alloc::string::String;
use alloc::vec::Vec;

use nene_ledger_core::cell_change::CellChange;

use crate::selection::Selection;

/// 切り取った範囲と、この窓で貼ったときに元を空にする書き換え（空でないセルだけ・それぞれのファイルの今の値と一緒）。
/// クリップボードに置いた文字列と、切り取ったときの履歴の進み（`EditHistory::revision`）を覚え、どちらかが違えば使わない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CutMark {
    range: Selection,
    clears: Vec<(CellChange, String)>,
    text: String,
    revision: u64,
}

impl CutMark {
    /// 範囲 `range` を、空にする書き換え `clears`・置いた文字列 `text`・履歴の進み `revision` で覚える。
    #[must_use]
    pub fn new(
        range: Selection,
        clears: Vec<(CellChange, String)>,
        text: String,
        revision: u64,
    ) -> Self {
        Self {
            range,
            clears,
            text,
            revision,
        }
    }

    /// 切り取った範囲（点線の枠で示す）。
    #[must_use]
    pub const fn range(&self) -> Selection {
        self.range
    }

    /// 履歴の進みが `revision` のときに、まだ使えるか（その間に表が変わっていない）。
    #[must_use]
    pub const fn is_current(&self, revision: u64) -> bool {
        self.revision == revision
    }

    /// 貼る文字列 `text` が、この切り取りで置いたものか。
    #[must_use]
    pub fn placed(&self, text: &str) -> bool {
        self.text == text
    }

    /// 元を空にする書き換え（取り出す）。
    #[must_use]
    pub fn into_clears(self) -> Vec<(CellChange, String)> {
        self.clears
    }
}
