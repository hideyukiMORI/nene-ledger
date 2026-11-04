//! 作り終えた索引のまとめ。

/// 作り終えた索引のまとめ。レコードの数と、閉じなかった引用符の位置（ADR 0018 の決定 1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexSummary {
    records: u64,
    unclosed_quote: Option<u64>,
}

impl IndexSummary {
    /// まとめを作る。
    #[must_use]
    pub fn new(records: u64, unclosed_quote: Option<u64>) -> Self {
        Self {
            records,
            unclosed_quote,
        }
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
