//! クリップボードに置く文字列。

use alloc::string::String;

/// クリップボードに置く文字列（タブ区切り・CRLF・UTF-8）。UI は持ち主になって渡すだけ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardText(String);

impl ClipboardText {
    /// 文字列から作る。
    #[must_use]
    pub fn new(text: String) -> Self {
        Self(text)
    }

    /// 中身。
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
