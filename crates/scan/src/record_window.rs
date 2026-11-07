//! 求められた範囲のレコード。

use crate::mark::Mark;
use crate::window_record::WindowRecord;

/// 求められた範囲のレコード。ファイルの終わりを越えた分は無い。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordWindow {
    records: Vec<WindowRecord>,
}

impl RecordWindow {
    /// 範囲を作る。
    #[must_use]
    pub(crate) fn new(records: Vec<WindowRecord>) -> Self {
        Self { records }
    }

    /// 求められた範囲のレコード（番号の順）。
    #[must_use]
    pub fn records(&self) -> &[WindowRecord] {
        &self.records
    }

    /// 最初のレコードの番号が `first` のとき、最後のレコードの番号と始まり（次に読むときに目印の代わりに使える・
    /// `windowing::read_records_after`）。レコードが無ければ `None`。
    #[must_use]
    pub fn last_mark(&self, first: u64) -> Option<Mark> {
        let last = self.records.last()?;
        let before = u64::try_from(self.records.len().checked_sub(1)?).ok()?;
        Some(Mark::new(first.checked_add(before)?, last.offset()))
    }
}
