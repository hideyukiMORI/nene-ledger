//! いくつかの部分に分けて並列に畳み込める畳み込み（ADR 0030 の決定 4・QLT-008）。

use crate::record_fold::RecordFold;

/// 範囲をいくつかの部分に分けて、それぞれの写しで並列に畳み込み、前の部分から順に合わせられる畳み込み。
/// 合わせた結果は、分け方とスレッドの数に依らず、1 つのスレッドで先頭から畳み込んだのと同じでなければならない
/// （QLT-008）。
pub trait SplitFold: RecordFold + Clone + 'static {
    /// 前の部分の結果 `self` に、すぐ後ろの部分の結果 `later` を合わせる。
    #[must_use]
    fn merged(self, later: Self) -> Self;
}
