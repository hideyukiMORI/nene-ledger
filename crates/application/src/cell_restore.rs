//! ドラッグの前の選択と表示。

use crate::selection::Selection;

/// ドラッグの前の選択と表示（縦の表示の先頭・表示の最初の列）と、前に足した範囲の数。ドラッグ中の Esc で戻す
/// （Ctrl+ドラッグで足しかけた範囲も外す）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellRestore {
    selection: Selection,
    others: usize,
    first_body: u64,
    first_column: usize,
}

impl CellRestore {
    /// 選択 `selection`（前に足した範囲は `others` 個）・縦の表示の先頭 `first_body`・表示の最初の列 `first_column`。
    #[must_use]
    pub fn new(selection: (Selection, usize), first_body: u64, first_column: usize) -> Self {
        Self {
            selection: selection.0,
            others: selection.1,
            first_body,
            first_column,
        }
    }

    /// 前に足した範囲の数。
    #[must_use]
    pub fn others(self) -> usize {
        self.others
    }

    /// 選択。
    #[must_use]
    pub fn selection(self) -> Selection {
        self.selection
    }

    /// 縦の表示の先頭。
    #[must_use]
    pub fn first_body(self) -> u64 {
        self.first_body
    }

    /// 表示の最初の列。
    #[must_use]
    pub fn first_column(self) -> usize {
        self.first_column
    }
}
