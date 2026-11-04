//! 原本の中の 1 レコードの位置と長さ。

use crate::record_length::RecordLength;

/// 原本の中の 1 レコードの始まりと長さ。範囲の取り出しが境目を探した結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RecordSpan {
    start: u64,
    length: RecordLength,
}

impl RecordSpan {
    pub(crate) fn new(start: u64, length: RecordLength) -> Self {
        Self { start, length }
    }

    pub(crate) fn start(self) -> u64 {
        self.start
    }

    pub(crate) fn length(self) -> RecordLength {
        self.length
    }
}
