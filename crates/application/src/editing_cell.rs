//! 編集中のセルの描き方（ADR 0026 の決定 3）。

use alloc::string::String;

/// 編集中のセルに描くもの。位置は表示の字数（半角 1・全角 2）で、画素には描く側が直す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EditingCell {
    text: String,
    caret: u32,
    underline: Option<(u32, u32)>,
}

impl EditingCell {
    /// 1 行に直した文字列 `text`・カーソルの左の字数 `caret`・確定前の文字列の範囲 `underline`（始まりと終わりの字数）。
    pub(crate) const fn new(text: String, caret: u32, underline: Option<(u32, u32)>) -> Self {
        Self {
            text,
            caret,
            underline,
        }
    }

    /// 描く文字列（改行は印に直してある）。
    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    /// カーソルの左の字数。
    pub(crate) const fn caret(&self) -> u32 {
        self.caret
    }

    /// 確定前の文字列の範囲（字数）。
    pub(crate) const fn underline(&self) -> Option<(u32, u32)> {
        self.underline
    }
}
