//! 範囲を畳み込む仕事の終わり方。

/// 範囲を畳み込む仕事の終わり方。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RangeOutcome<F> {
    /// 範囲の終わり（またはファイルの終わり）まで畳み込んだ。
    Finished(F),
    /// 中止を求められて止まった（途中の結果は返さない・ADR 0014 の決定 4）。
    Cancelled,
}
