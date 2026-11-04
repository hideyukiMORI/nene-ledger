//! レコードが方言どおりに切れたか。

/// レコードが方言どおりに切れたか。壊れた行も元のバイト列のまま保つ（OPEN-15・FR-023）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RowShape {
    /// 方言どおりに切れた。
    Complete,
    /// 引用符が閉じないまま入力が終わった。レコードは入力の終わりまで続く。
    UnclosedQuote,
    /// 閉じなかった引用符を、利用者の選択で文字として読んだ（ADR 0018・`UnclosedQuote::Literal`）。
    LiteralQuote,
}
