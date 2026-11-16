//! 書き出すときの、引用符の付け方（ADR 0036・SAVE-03・JP-05）。

/// 書き出すときの引用符の付け方。閉じた集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuotePolicy {
    /// 要るときだけ（区切り・引用符・改行を含むセル）。
    AsNeeded,
    /// すべてのセルを囲む（JP-05）。引用を使わない方言では、要るときだけと同じ。
    All,
}
