//! 検索と置換で照らす所（表の全部か、選んだ列か、選んだ矩形の範囲）。

use alloc::vec::Vec;
use core::ops::Range;

use nene_ledger_core::order_run::OrderRun;

/// 検索と置換で照らす所。欄を開いたときの選び方で決まり、開き直すまで変えない（当たりへ動いても範囲は残る）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FindArea {
    /// 照らす表示の列（列全体か矩形を選んで開いたとき・GitHub Issue #411）。`None` なら全部の列。
    columns: Option<Range<usize>>,
    /// 照らす本体の行（最初と最後・矩形を選んで開いたとき・GitHub Issue #513）。`None` なら全部の行。
    rows: Option<(u64, u64)>,
    /// 欄を開いたときの表の並び（行の並び・列の並び・表示だけの列の入れ替え）。行と列は番目で持つので、並びが変わったら
    /// 別の行・別の列を指す。変わっていたら限らない（GitHub Issue #536）。
    shape: (Vec<OrderRun>, Vec<OrderRun>, Vec<usize>),
}

impl FindArea {
    /// 表の全部。
    pub(crate) const fn whole() -> Self {
        Self {
            columns: None,
            rows: None,
            shape: (Vec::new(), Vec::new(), Vec::new()),
        }
    }

    /// 表示の列 `columns` と本体の行 `rows`（どちらも `None` なら限らない）を、今の表の並び `shape`（行の並び・列の並び・
    /// 表示だけの列の入れ替え）で覚える。
    pub(crate) const fn new(
        columns: Option<Range<usize>>,
        rows: Option<(u64, u64)>,
        shape: (Vec<OrderRun>, Vec<OrderRun>, Vec<usize>),
    ) -> Self {
        Self {
            columns,
            rows,
            shape,
        }
    }

    /// 表の並びが、欄を開いたときのままか（変わっていたら、覚えた行と列の番目は別の所を指す）。
    pub(crate) fn holds(&self, rows: &[OrderRun], columns: &[OrderRun], order: &[usize]) -> bool {
        self.shape.0 == rows && self.shape.1 == columns && self.shape.2 == order
    }

    /// 照らす表示の列。
    pub(crate) fn columns(&self) -> Option<Range<usize>> {
        self.columns.clone()
    }

    /// 照らす本体の行（最初と最後）。
    pub(crate) const fn rows(&self) -> Option<(u64, u64)> {
        self.rows
    }
}
