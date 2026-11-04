//! まばらな索引の目印。

/// 目印。あるレコードの番号と、そのレコードが始まるバイト位置（ADR 0017 の決定 2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mark {
    record: u64,
    offset: u64,
}

impl Mark {
    /// 目印を作る。
    #[must_use]
    pub(crate) fn new(record: u64, offset: u64) -> Self {
        Self { record, offset }
    }

    /// レコードの番号（0 始まり）。
    #[must_use]
    pub fn record(self) -> u64 {
        self.record
    }

    /// レコードが始まるバイト位置。
    #[must_use]
    pub fn offset(self) -> u64 {
        self.offset
    }
}
