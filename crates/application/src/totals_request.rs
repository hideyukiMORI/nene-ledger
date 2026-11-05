//! 選んだ範囲を集計するために、合成ルートがすること。

/// 選んだ範囲を集計するために、合成ルートがすること。`Unchanged` 以外なら、前の裏の計算は捨てる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TotalsRequest {
    /// 範囲が変わっていない（裏の計算があれば続ける）。
    Unchanged,
    /// 数え終えたか、出さない。
    Settled,
    /// 行の並びの位置 `first_line` から `count` 行を読んで `Document::count_totals` に渡す。
    Read {
        /// 最初のレコードの番号。
        first_line: u64,
        /// 読むレコードの数（見出しのレコードを含むことがある）。
        count: u64,
    },
    /// 位置 `first_line` から `count` 行を裏で数え、`Document::totals_report` に知らせる（行を足し消ししていないとき
    /// だけ来るので、位置は原本のレコードの番号と同じ）。
    Background {
        /// 最初のレコードの番号。
        first_line: u64,
        /// 数えるレコードの数（見出しのレコードを含むことがある）。
        count: u64,
    },
}
