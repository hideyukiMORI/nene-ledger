//! 編集中のセルの描き方（ADR 0026 の決定 3）。

use alloc::string::String;

/// 編集中のセルに描くもの。位置は描く文字列の字の番目で、画素には描く側が、UI が測った字の送り幅で直す（GitHub
/// Issue #505。前は半角 1・全角 2 の見積もりで、比例の字体ではカーソルの棒と選択の地がずれた）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EditingCell {
    text: String,
    caret: u32,
    underline: Option<(u32, u32)>,
    /// 選んだ字の範囲（始まりと終わりの字の番目・無ければ `None`・GitHub Issue #496）。
    selection: Option<(u32, u32)>,
    /// カーソルの棒を描くか（詳細ペインで打っている間は、表のセルには描かない・GitHub Issue #458）。
    caret_shown: bool,
}

impl EditingCell {
    /// 1 行に直した文字列 `text`・カーソルの左の字の数 `caret`・確定前の文字列の範囲 `underline`（始まりと終わりの字の番目）。
    pub(crate) const fn new(text: String, caret: u32, underline: Option<(u32, u32)>) -> Self {
        Self {
            text,
            caret,
            underline,
            selection: None,
            caret_shown: true,
        }
    }

    /// 選んだ字の範囲 `selection`（始まりと終わりの字の番目）を付ける。
    pub(crate) fn with_selection(self, selection: Option<(u32, u32)>) -> Self {
        Self { selection, ..self }
    }

    /// 選んだ字の範囲（字の番目）。
    pub(crate) const fn selection(&self) -> Option<(u32, u32)> {
        self.selection
    }

    /// カーソルの棒を描かない（詳細ペインで打っている間・GitHub Issue #458）。
    pub(crate) fn without_caret(self) -> Self {
        Self {
            caret_shown: false,
            ..self
        }
    }

    /// カーソルの棒を描くか。
    pub(crate) const fn caret_shown(&self) -> bool {
        self.caret_shown
    }

    /// 描く文字列（改行は印に直してある）。
    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    /// カーソルの左の字の数。
    pub(crate) const fn caret(&self) -> u32 {
        self.caret
    }

    /// 確定前の文字列の範囲（字の番目）。
    pub(crate) const fn underline(&self) -> Option<(u32, u32)> {
        self.underline
    }
}
