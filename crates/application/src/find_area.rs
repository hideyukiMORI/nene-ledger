//! 検索と置換で照らす所（表の全部か、選んだ列か、選んだ矩形の範囲）。

use core::ops::Range;

/// 検索と置換で照らす所。欄を開いたときの選び方で決まり、開き直すまで変えない（当たりへ動いても範囲は残る）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FindArea {
    /// 照らす表示の列（列全体か矩形を選んで開いたとき・GitHub Issue #411）。`None` なら全部の列。
    columns: Option<Range<usize>>,
    /// 照らす本体の行（最初と最後・矩形を選んで開いたとき・GitHub Issue #513）。`None` なら全部の行。
    rows: Option<(u64, u64)>,
}

impl FindArea {
    /// 表の全部。
    pub(crate) const fn whole() -> Self {
        Self {
            columns: None,
            rows: None,
        }
    }

    /// 表示の列 `columns` と本体の行 `rows`（どちらも `None` なら限らない）。
    pub(crate) const fn new(columns: Option<Range<usize>>, rows: Option<(u64, u64)>) -> Self {
        Self { columns, rows }
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
