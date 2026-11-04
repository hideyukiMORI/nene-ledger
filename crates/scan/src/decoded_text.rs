//! 表示のために復号した文字列。

use std::string::String;

/// 表示のために復号した文字列。原本のバイト列は変えない（ADR 0015 の決定 3）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedText {
    text: String,
    malformed: bool,
}

impl DecodedText {
    /// 復号の結果を作る。
    #[must_use]
    pub(crate) fn new(text: String, malformed: bool) -> Self {
        Self { text, malformed }
    }

    /// 復号した文字列。読めないバイトは U+FFFD に置き換わっている。
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// 読めないバイトがあったか。あれば、文字コードを選び直す手がかりとして示す（OPEN-10・FR-023）。
    #[must_use]
    pub fn malformed(&self) -> bool {
        self.malformed
    }
}
