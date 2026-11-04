//! 範囲の取り出しで返す、1 つのレコード。

use nene_ledger_core::record::Record;

use crate::block_size::BlockSize;
use crate::record_length::RecordLength;

/// 1 つのレコードについて読む量の上限。超えた分は読まない（BIG-04）。
pub const RECORD_READ_LIMIT: BlockSize = BlockSize::MEBIBYTE;

/// 範囲の取り出しで返す、1 つのレコード。原本の位置・全体の長さ・読んだバイト列・切った結果。
///
/// 🔴 全体が `RECORD_READ_LIMIT` を超えると先頭の上限分だけを読む（`is_truncated`）。索引が後ろのレコードを
/// 数えていれば終わりまで探して長さは `RecordLength::Exact`（GitHub Issue #13）、数えていなければ
/// `RecordLength::MoreThan` で、終わりは探していない（公開前の Issue #23）。どちらでも `record` は
/// 先頭の分だけを切ったもので、終わり方と形（閉じない引用符か）は全体のものではない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowRecord {
    offset: u64,
    length: RecordLength,
    bytes: Vec<u8>,
    record: Record,
}

impl WindowRecord {
    /// レコードを作る。`record` の区間は `bytes` の中の位置。
    #[must_use]
    pub(crate) fn new(offset: u64, length: RecordLength, bytes: Vec<u8>, record: Record) -> Self {
        Self {
            offset,
            length,
            bytes,
            record,
        }
    }

    /// レコードが始まる、原本の中のバイト位置。
    #[must_use]
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// 改行を含む、原本の中の全体の長さ（上限を超えたら「それより長い」）。
    #[must_use]
    pub fn length(&self) -> RecordLength {
        self.length
    }

    /// 読んだバイト列（改行を含む）。切り詰めたときは先頭の分だけ。
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// `bytes` を切った結果。区間は `bytes` の中の位置。
    #[must_use]
    pub fn record(&self) -> &Record {
        &self.record
    }

    /// 上限を超えたので、先頭の分だけを読んだか。
    #[must_use]
    pub fn is_truncated(&self) -> bool {
        match self.length {
            RecordLength::Exact(length) => {
                u64::try_from(self.bytes.len()).map_or(true, |read| read < length)
            }
            RecordLength::MoreThan(_) => true,
        }
    }
}
