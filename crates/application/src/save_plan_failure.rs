//! 保存の計画を作れない理由。

/// 保存の計画を作れない理由。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SavePlanFailure<F> {
    /// 原本を読めなかった。
    Lookup(F),
    /// 書き換えた行が原本に無い（外から短くされた）。
    Missing(u64),
    /// 書き換えた行が長すぎて、全体を読んでいない（1 MiB を超える・GitHub Issue #36）。
    TooLong(u64),
    /// その方言では書けない値（`Plain` の方言で区切りか改行を含む）。
    Unwritable {
        /// 原本のレコードの番号。
        row: u64,
        /// セルの番号。
        column: usize,
    },
    /// 書き換えたセルの番号が昇順でない（差分の持ち方の誤り）。
    Unordered {
        /// 原本のレコードの番号。
        row: u64,
        /// セルの番号。
        column: usize,
    },
    /// 足した行の値が、その方言では書けない（`Plain` の方言で区切りか改行を含む）。
    UnwritableAdded {
        /// セルの番号。
        column: usize,
    },
    /// この版ではまだ保存できない書き換え（足した行と列・「セルが無い」への書き換え。ADR 0024 の層 1）。
    Unsupported,
}
