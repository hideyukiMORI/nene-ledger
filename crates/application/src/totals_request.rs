//! 選んだ範囲を集計するために、合成ルートがすること。

/// 選んだ範囲を集計するために、合成ルートがすること。`Unchanged` 以外なら、前の裏の計算は捨てる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TotalsRequest {
    /// 範囲が変わっていない（裏の計算があれば続ける）。
    Unchanged,
    /// 数え終えたか、出さない。
    Settled,
    /// レコード `first_record` から `count` 個を読んで `Document::count_totals` に渡す。
    Read {
        /// 最初のレコードの番号。
        first_record: u64,
        /// 読むレコードの数（見出しのレコードを含むことがある）。
        count: u64,
    },
    /// レコード `first_record` から `count` 個を裏で数え、`Document::totals_report` に知らせる。
    Background {
        /// 最初のレコードの番号。
        first_record: u64,
        /// 数えるレコードの数（見出しのレコードを含むことがある）。
        count: u64,
    },
}
