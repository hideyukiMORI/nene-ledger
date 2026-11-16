//! 作り終えた索引のまとめ。

use nene_ledger_core::line_ending_counts::LineEndingCounts;

/// 作り終えた索引のまとめ。レコードの数と、閉じなかった引用符の位置（ADR 0018 の決定 1）と、ファイル全体の
/// レコードの改行の内訳（OPEN-08・数えていなければ `None`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexSummary {
    records: u64,
    unclosed_quote: Option<u64>,
    /// 閉じなかった引用符のある行（索引のまとめではレコードの番号、状態の行に見せるまとめでは行の並びの位置・0 始まり・
    /// GitHub Issue #519）。分からなければ `None`。
    unclosed_record: Option<u64>,
    line_endings: Option<LineEndingCounts>,
}

impl IndexSummary {
    /// まとめを作る。
    #[must_use]
    pub fn new(records: u64, unclosed_quote: Option<u64>) -> Self {
        Self {
            records,
            unclosed_quote,
            unclosed_record: None,
            line_endings: None,
        }
    }

    /// 閉じなかった引用符のある行 `record`（0 始まり）を添える（壊れた所を行で言い、そこへ飛ぶ・ADR 0018 の決定 2）。
    #[must_use]
    pub fn with_unclosed_record(self, record: Option<u64>) -> Self {
        Self {
            unclosed_record: record,
            ..self
        }
    }

    /// 閉じなかった引用符のある行（0 始まり・分からなければ `None`）。
    #[must_use]
    pub fn unclosed_record(self) -> Option<u64> {
        self.unclosed_record
    }

    /// ファイル全体で数えたレコードの改行の内訳を添える（OPEN-08）。
    #[must_use]
    pub fn with_line_endings(self, line_endings: LineEndingCounts) -> Self {
        Self {
            line_endings: Some(line_endings),
            ..self
        }
    }

    /// ファイル全体のレコードの改行の内訳。数えていなければ `None`。
    #[must_use]
    pub fn line_endings(self) -> Option<LineEndingCounts> {
        self.line_endings
    }

    /// レコードの数。
    #[must_use]
    pub fn records(self) -> u64 {
        self.records
    }

    /// 入力の終わりまで閉じなかった引用符の位置（あれば最初のもの）。
    #[must_use]
    pub fn unclosed_quote(self) -> Option<u64> {
        self.unclosed_quote
    }
}
