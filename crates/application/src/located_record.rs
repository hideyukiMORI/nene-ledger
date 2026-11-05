//! 原本の中の位置が分かった 1 つのレコード。

use alloc::vec::Vec;

use nene_ledger_core::record::Record;

/// 原本の中の位置が分かった 1 つのレコード。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocatedRecord {
    offset: u64,
    bytes: Vec<u8>,
    record: Record,
    whole: bool,
}

impl LocatedRecord {
    /// レコードを作る。`bytes` は原本の `offset` から読んだバイト列、`record` はそれを切った結果（区間は
    /// `bytes` の中の位置）。`whole` は、レコードの全体（終わり方まで）を読んだか。
    #[must_use]
    pub fn new(offset: u64, bytes: Vec<u8>, record: Record, whole: bool) -> Self {
        Self {
            offset,
            bytes,
            record,
            whole,
        }
    }

    /// レコードが始まる、原本の中のバイト位置。
    #[must_use]
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// 読んだバイト列。
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// `bytes` を切った結果。
    #[must_use]
    pub fn record(&self) -> &Record {
        &self.record
    }

    /// 全体（終わり方まで）を読んだか。
    #[must_use]
    pub fn is_whole(&self) -> bool {
        self.whole
    }
}
