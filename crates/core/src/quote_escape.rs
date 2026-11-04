//! 引用符の中での逃がし方。

/// 引用符で囲まれたセルの中で、引用符をどう逃がすか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteEscape {
    /// 引用符を 2 つ重ねる（`""`）。
    Doubled,
    /// バックスラッシュで次の 1 バイトを逃がす（`\"`）。
    Backslash,
}
