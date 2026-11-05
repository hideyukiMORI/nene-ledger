//! レコードの長さ。

/// 範囲の取り出しで分かったレコードの長さ（改行を含む）。閉じた集合（RS-004: 分からないことを `None` にしない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordLength {
    /// 終わりまで見つけた。
    Exact(u64),
    /// 上限（`RECORD_READ_LIMIT`）を超え、索引がその後ろのレコードを数えていないので、終わりを探さなかった
    /// （閉じない引用符・作り終える前の索引での巨大なセル）。
    MoreThan(u64),
}
