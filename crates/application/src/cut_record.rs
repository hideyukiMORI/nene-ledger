//! 上限で切り詰めて、先頭の部分だけを読んだレコード（ADR 0018 の決定 2・GitHub Issue #36）。

use alloc::format;
use alloc::string::String;

use crate::status_line::group_digits;

/// 1 MiB（1,048,576 バイト）。範囲の取り出しの上限（scan の `RECORD_READ_LIMIT`）と同じ値で、文言に使う。
const MEBIBYTE: u64 = 1_048_576;

/// 範囲の取り出しの上限を超えて、先頭の部分だけを読んだレコード。見えているセルの値はその部分だけで、原本は変えない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CutRecord {
    record: u64,
    total_bytes: Option<u64>,
}

impl CutRecord {
    /// レコード `record`（0 始まり）を切り詰めた。`total_bytes` は分かればレコードの全体のバイト数（索引が終わりを
    /// 知らなければ `None`）。
    #[must_use]
    pub fn new(record: u64, total_bytes: Option<u64>) -> Self {
        Self {
            record,
            total_bytes,
        }
    }

    /// レコードの番号（0 始まり）。
    #[must_use]
    pub fn record(self) -> u64 {
        self.record
    }

    /// 全体のバイト数（分からなければ `None`）。
    #[must_use]
    pub fn total_bytes(self) -> Option<u64> {
        self.total_bytes
    }

    /// 詳細ペインの見出しの行に出す文言（「1 MiB で切り詰め（行の全体 2.0 MiB）」）。
    #[must_use]
    pub fn describe(self) -> String {
        match self.total_bytes {
            Some(total) => format!("1 MiB で切り詰め（行の全体 {}）", mebibytes(total)),
            None => String::from("1 MiB で切り詰め（行は 1 MiB を超える）"),
        }
    }
}

/// バイト数を、小数 1 桁の MiB にする（切り捨て）。
fn mebibytes(bytes: u64) -> String {
    let tenths = bytes.saturating_mul(10) / MEBIBYTE;
    format!("{}.{} MiB", group_digits(tenths / 10), tenths % 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cut_is_described_with_the_whole_size_when_known() {
        let known = CutRecord::new(3, Some(MEBIBYTE + MEBIBYTE / 2));
        assert_eq!(known.describe(), "1 MiB で切り詰め（行の全体 1.5 MiB）");
        assert_eq!(
            CutRecord::new(3, Some(1_500 * MEBIBYTE)).describe(),
            "1 MiB で切り詰め（行の全体 1,500.0 MiB）"
        );
        assert_eq!(
            CutRecord::new(3, None).describe(),
            "1 MiB で切り詰め（行は 1 MiB を超える）"
        );
        assert_eq!((known.record(), known.total_bytes()), (3, Some(1_572_864)));
    }
}
