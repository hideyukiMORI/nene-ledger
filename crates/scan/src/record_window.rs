//! 求められた範囲のレコード。

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
}
