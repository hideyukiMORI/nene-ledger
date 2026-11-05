//! 編集中のセルの状態（ADR 0026 の決定 1〜3・EDIT-01・10・11）。
//!
//! `Document` が 1 つだけ持つ。欄の規則は core の `TextEdit`。ここは、入り方・状態・矢印の扱い・確定の判定だけを持つ。

use alloc::borrow::ToOwned;
use alloc::string::String;

use nene_ledger_core::caret_move::CaretMove;
use nene_ledger_core::cell_value::CellValue;
use nene_ledger_core::column_id::ColumnId;
use nene_ledger_core::pasted_text::pasted_text;
use nene_ledger_core::preedit::Preedit;
use nene_ledger_core::row_id::RowId;
use nene_ledger_core::text_edit::TextEdit;

use crate::cursor_step::CursorStep;
use crate::edit_commit::EditCommit;
use crate::edit_mode::EditMode;

/// 編集中のセル。どのセルか・見えていた値・原本の値・状態・欄。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellEditor {
    row: RowId,
    column: ColumnId,
    shown: String,
    source: Option<String>,
    mode: EditMode,
    field: TextEdit,
}

impl CellEditor {
    /// 打ち始めで入る（入力の状態・欄は空）。`shown` は見えていた値（差分があれば差分の値）、
    /// `source` は原本のセルの値（原本にセルが無ければ `None`）。
    #[must_use]
    pub fn entering(cell: (RowId, ColumnId), shown: &str, source: Option<&str>) -> Self {
        Self::started(cell, (shown, source), EditMode::Entering)
    }

    /// F2・ダブルクリックで入る（編集の状態・欄は見えていた値・カーソルは末尾）。
    #[must_use]
    pub fn editing(cell: (RowId, ColumnId), shown: &str, source: Option<&str>) -> Self {
        Self::started(cell, (shown, source), EditMode::Editing)
    }

    fn started(
        (row, column): (RowId, ColumnId),
        (shown, source): (&str, Option<&str>),
        mode: EditMode,
    ) -> Self {
        let field = match mode {
            EditMode::Entering => TextEdit::empty(),
            EditMode::Editing => TextEdit::showing(shown),
        };
        Self {
            row,
            column,
            shown: shown.to_owned(),
            source: source.map(ToOwned::to_owned),
            mode,
            field,
        }
    }

    /// 編集中のセルの行。
    #[must_use]
    pub const fn row(&self) -> RowId {
        self.row
    }

    /// 編集中のセルの列。
    #[must_use]
    pub const fn column(&self) -> ColumnId {
        self.column
    }

    /// 今の状態。
    #[must_use]
    pub const fn mode(&self) -> EditMode {
        self.mode
    }

    /// 欄（描く文字列とカーソルは `TextEdit::display`）。
    #[must_use]
    pub const fn field(&self) -> &TextEdit {
        &self.field
    }

    /// 1 字打つ。
    pub fn type_character(&mut self, character: char) {
        self.field = self.field.typed(character);
    }

    /// セルの中の改行（Alt+Enter・EDIT-10）。
    pub fn break_line(&mut self) {
        self.field = self.field.typed('\n');
    }

    /// 貼り付け・IME の確定。改行を揃え、制御文字を捨ててから挿す。
    pub fn insert(&mut self, text: &str) {
        self.field = self.field.inserted(&pasted_text(text));
    }

    /// 確定前の文字列を置き換える（IME）。
    pub fn compose(&mut self, preedit: Option<Preedit>) {
        self.field = self.field.with_preedit(preedit);
    }

    /// Backspace。
    pub fn erase_before(&mut self) {
        self.field = self.field.backspaced();
    }

    /// Delete。
    pub fn erase_after(&mut self) {
        self.field = self.field.deleted();
    }

    /// F2。入力と編集の状態を替える。欄の中身はそのまま。
    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            EditMode::Entering => EditMode::Editing,
            EditMode::Editing => EditMode::Entering,
        };
    }

    /// カーソルを動かすキー。欄の中で動かしたら `true`。
    /// `false` なら、確定してその向きへ表のカーソルを動かす（入力の状態の矢印・Home・End と、両方の状態の `PageUp` など）。
    pub fn arrow(&mut self, step: CursorStep, extend: bool) -> bool {
        let inside = match step {
            CursorStep::Up => CaretMove::Up,
            CursorStep::Down => CaretMove::Down,
            CursorStep::Left => CaretMove::Left,
            CursorStep::Right => CaretMove::Right,
            CursorStep::RowStart => CaretMove::Home,
            CursorStep::RowEnd => CaretMove::End,
            CursorStep::PageUp | CursorStep::PageDown | CursorStep::Top | CursorStep::Bottom => {
                return false;
            }
        };
        match self.mode {
            EditMode::Entering => false,
            EditMode::Editing => {
                self.field = self.field.moved(inside, extend);
                true
            }
        }
    }

    /// 確定する。確定前の文字列は捨てる（IME が確定を送ってから確定する）。
    #[must_use]
    pub fn commit(&self) -> EditCommit {
        let value = self.field.text();
        if value == self.shown {
            EditCommit::Keep
        } else if self.source.as_deref() == Some(value) {
            EditCommit::Restore {
                row: self.row,
                column: self.column,
            }
        } else {
            EditCommit::Set {
                row: self.row,
                column: self.column,
                value: CellValue::Text(value.to_owned()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CELL: (RowId, ColumnId) = (RowId::Source(3), ColumnId::Source(1));

    fn typed(editor: &mut CellEditor, text: &str) {
        for character in text.chars() {
            editor.type_character(character);
        }
    }

    #[test]
    fn typing_replaces_the_shown_value() {
        let mut editor = CellEditor::entering(CELL, "0012", Some("0012"));
        assert_eq!(editor.mode(), EditMode::Entering);
        assert_eq!(editor.field().text(), "");
        typed(&mut editor, "東京");
        assert_eq!(
            editor.commit(),
            EditCommit::Set {
                row: RowId::Source(3),
                column: ColumnId::Source(1),
                value: CellValue::Text(String::from("東京")),
            }
        );
    }

    #[test]
    fn f2_continues_from_the_shown_value_with_the_cursor_at_the_end() {
        let mut editor = CellEditor::editing(CELL, "abc", Some("abc"));
        assert_eq!(editor.field().caret().cursor(), 3);
        typed(&mut editor, "d");
        assert_eq!(editor.field().text(), "abcd");
    }

    #[test]
    fn arrows_commit_while_entering() {
        let mut entering = CellEditor::entering(CELL, "", None);
        typed(&mut entering, "ab");
        for step in [
            CursorStep::Left,
            CursorStep::Right,
            CursorStep::Up,
            CursorStep::Down,
            CursorStep::RowStart,
            CursorStep::RowEnd,
        ] {
            assert!(!entering.arrow(step, false), "{step:?}");
        }
        assert_eq!(entering.field().text(), "ab");
    }

    #[test]
    fn arrows_move_the_caret_while_editing_but_page_keys_commit() {
        let mut editing = CellEditor::editing(CELL, "ab", None);
        assert!(editing.arrow(CursorStep::Left, false));
        assert_eq!(editing.field().caret().cursor(), 1);
        assert!(editing.arrow(CursorStep::RowStart, false));
        assert_eq!(editing.field().caret().cursor(), 0);
        assert!(editing.arrow(CursorStep::RowEnd, true));
        assert!(editing.field().caret().has_selection());
        for step in [
            CursorStep::PageUp,
            CursorStep::PageDown,
            CursorStep::Top,
            CursorStep::Bottom,
        ] {
            assert!(!editing.arrow(step, false), "{step:?}");
        }
    }

    #[test]
    fn f2_switches_between_entering_and_editing() {
        let mut editor = CellEditor::entering(CELL, "x", None);
        typed(&mut editor, "ab");
        editor.toggle_mode();
        assert_eq!(editor.mode(), EditMode::Editing);
        assert!(editor.arrow(CursorStep::Left, false));
        typed(&mut editor, "-");
        assert_eq!(editor.field().text(), "a-b");
        editor.toggle_mode();
        assert_eq!(editor.mode(), EditMode::Entering);
        assert!(!editor.arrow(CursorStep::Left, false));
    }

    #[test]
    fn an_unchanged_value_keeps_the_edits_and_the_source_value_restores_them() {
        assert_eq!(
            CellEditor::editing(CELL, "same", Some("orig")).commit(),
            EditCommit::Keep
        );
        let mut back = CellEditor::entering(CELL, "edited", Some("orig"));
        typed(&mut back, "orig");
        assert_eq!(
            back.commit(),
            EditCommit::Restore {
                row: RowId::Source(3),
                column: ColumnId::Source(1),
            }
        );
        let mut absent = CellEditor::entering(CELL, "", None);
        assert_eq!(absent.commit(), EditCommit::Keep);
        typed(&mut absent, "new");
        assert!(matches!(absent.commit(), EditCommit::Set { .. }));
    }

    #[test]
    fn pasted_text_is_tidied_and_line_breaks_stay_inside_the_cell() {
        let mut editor = CellEditor::entering(CELL, "", None);
        editor.insert("一\r\n二\u{7}\t三");
        editor.break_line();
        assert_eq!(editor.field().text(), "一\n二\t三\n");
        editor.erase_before();
        editor.erase_after();
        assert_eq!(editor.field().text(), "一\n二\t三");
    }

    #[test]
    fn the_preedit_is_not_committed_until_the_ime_sends_the_result() {
        let mut editor = CellEditor::entering(CELL, "", None);
        editor.compose(Some(Preedit::new("にほん", 3)));
        assert_eq!(editor.field().display().0, "にほん");
        assert_eq!(editor.commit(), EditCommit::Keep);
        editor.insert("日本");
        assert_eq!(editor.field().preedit(), None);
        assert_eq!(editor.field().text(), "日本");
        assert_eq!(editor.row(), RowId::Source(3));
        assert_eq!(editor.column(), ColumnId::Source(1));
    }
}
