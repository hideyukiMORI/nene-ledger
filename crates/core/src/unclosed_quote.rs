//! 閉じない引用符の読み方。

/// 引用符が入力の終わりまで閉じないときの読み方。閉じた集合（ADR 0018）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnclosedQuote {
    /// 規則の正本。引用符から入力の終わりまでを 1 つのセルにする（既定）。
    ToEnd,
    /// 利用者が選んだとき。閉じなかった引用符を文字とみなし、そのセルを引用の無いセルとして切る。
    Literal,
}
