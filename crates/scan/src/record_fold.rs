//! 範囲のレコードを 1 つずつ受け取る畳み込み。

use crate::window_record::WindowRecord;

/// 範囲のレコードを 1 つずつ受け取る畳み込み（裏の仕事 `RangeJob` に渡す）。何を数えるかは呼ぶ側が決める。
pub trait RecordFold: Send {
    /// レコードの番号 `number` の `record` を取り込む。範囲の先頭から順に、1 度ずつ呼ばれる。
    fn record(&mut self, number: u64, record: &WindowRecord);
}
