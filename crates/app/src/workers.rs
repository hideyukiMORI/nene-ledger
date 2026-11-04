//! 画面のスレッドの外で動くもの。

use nene_ledger_application::ticker::Ticker;
use nene_ledger_scan::index_job::IndexJob;

/// 画面のスレッドの外で動くもの。索引を作る仕事と、押し続けとドラッグの時計。
#[derive(Debug)]
pub(crate) struct Workers {
    index: IndexJob,
    ticker: Box<dyn Ticker>,
}

impl Workers {
    /// 索引を作る仕事 `index` と時計 `ticker`。
    pub(crate) fn new(index: IndexJob, ticker: Box<dyn Ticker>) -> Self {
        Self { index, ticker }
    }

    /// 分ける（`Viewer` がそれぞれを持つ）。
    pub(crate) fn into_parts(self) -> (IndexJob, Box<dyn Ticker>) {
        (self.index, self.ticker)
    }
}
