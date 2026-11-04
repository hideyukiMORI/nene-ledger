//! 見出しの行の有無の推定。

/// 先頭の行が見出しかどうかの推定（OPEN-11）。利用者が選び直せる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderGuess {
    /// 見出しがありそう（数の列の先頭が文字列）。
    Present,
    /// 見出しはなさそう（数の列の先頭も数）。
    Absent,
    /// 決め手が無い。
    Unknown,
}
