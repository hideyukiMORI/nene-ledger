//! 求められた範囲のレコード。

use crate::mark::Mark;
use crate::window_record::WindowRecord;

/// 求められた範囲のレコード。ファイルの終わりを越えた分は無い。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordWindow {
    records: Vec<WindowRecord>,
    last: Option<Mark>,
}

impl RecordWindow {
    /// 範囲を作る。
    #[must_use]
    /// 範囲を作る。`last` は、境目を探して見つけた最後のレコードの番号と始まり（無ければ `None`）。
    pub(crate) fn new(records: Vec<WindowRecord>, last: Option<Mark>) -> Self {
        Self { records, last }
    }

    /// 求められた範囲のレコード（番号の順）。
    #[must_use]
    pub fn records(&self) -> &[WindowRecord] {
        &self.records
    }

    /// 境目を探して見つけた最後のレコードの番号と始まり（次に読むときに目印の代わりに使える・
    /// `windowing::read_records_after`）。番号は境目を数えて決めたもので、読んだレコードの数からは数えない。無ければ `None`。
    #[must_use]
    pub fn last_mark(&self) -> Option<Mark> {
        self.last
    }
}
