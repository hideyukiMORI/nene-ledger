//! 方言を作れなかった理由。

/// 方言（区切り・引用）として受け付けられない理由。閉じた集合（ARC-010）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialectFailure {
    /// 区切りが改行（CR / LF）である。
    LineBreakDelimiter,
    /// 区切りが ASCII でない（多バイトの文字の途中で切れる）。
    NonAsciiDelimiter,
    /// 引用符が改行（CR / LF）である。
    LineBreakQuote,
    /// 引用符が ASCII でない。
    NonAsciiQuote,
    /// 引用符が区切りと同じである。
    QuoteEqualsDelimiter,
    /// 引用符の中をバックスラッシュで逃がす流儀で、引用符がバックスラッシュである。
    BackslashQuote,
}
