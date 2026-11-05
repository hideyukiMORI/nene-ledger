//! 編集の欄の状態。文字列・カーソル・選択・確定前の文字列（ADR 0026 の決定 3・EDIT-01・10・11）。
//!
//! 位置はすべて**文字**（`char`）の添字で、バイトでは持たない。途中のバイトで切る経路が無い。
//! 操作は値を返す純関数で、同じ状態と同じ操作から同じ状態が出る（ARC-007）。
//! セルの値は `\n` で行に分かれてよい（EDIT-10）。Home / End は行の中で、上下は隣の行へ動く。
//! 出発点は `NeNeDocket` の `TextEdit`（同じ施主・同じ規約）。

use alloc::borrow::ToOwned;
use alloc::string::String;

use crate::caret_move::CaretMove;
use crate::preedit::Preedit;
use crate::text_caret::TextCaret;

/// 1 つの編集の欄の、今の文字列とカーソルと選択と確定前の文字列。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    text: String,
    caret: TextCaret,
    preedit: Option<Preedit>,
}

impl TextEdit {
    /// 文字列から始める。カーソルは末尾（F2・ダブルクリックで続けて直す）。
    #[must_use]
    pub fn showing(text: &str) -> Self {
        let end = text.chars().count();
        Self {
            text: text.to_owned(),
            caret: TextCaret::new(end, end),
            preedit: None,
        }
    }

    /// 空から始める（打ち始めで元の値を置き換える）。
    #[must_use]
    pub fn empty() -> Self {
        Self::showing("")
    }

    /// 確定した文字列。確定前の文字列は含まない。
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// カーソルと選択。
    #[must_use]
    pub const fn caret(&self) -> TextCaret {
        self.caret
    }

    /// 確定前の文字列。
    #[must_use]
    pub const fn preedit(&self) -> Option<&Preedit> {
        self.preedit.as_ref()
    }

    /// 確定前の文字列を置き換えた状態。IME が組み立てを始める・変える・やめるたびに呼ぶ。
    #[must_use]
    pub fn with_preedit(&self, preedit: Option<Preedit>) -> Self {
        Self {
            text: self.text.clone(),
            caret: self.caret,
            preedit,
        }
    }

    /// 1 字打った状態。改行の字はセルの中の改行になる（Alt+Enter・EDIT-10）。
    #[must_use]
    pub fn typed(&self, character: char) -> Self {
        let mut addition = String::new();
        addition.push(character);
        self.inserted(&addition)
    }

    /// 文字列を挿した状態。選択があればそれを置き換え、確定前の文字列は消える（IME の確定・貼り付け）。
    /// 貼り付けは先に [`crate::pasted_text::pasted_text`] で整える。
    #[must_use]
    pub fn inserted(&self, addition: &str) -> Self {
        let cleared = self.without_selection();
        let at = cleared.byte_offset(cleared.caret.cursor());
        let (head, tail) = cleared.text.split_at(at);
        let mut text = String::with_capacity(cleared.text.len().saturating_add(addition.len()));
        text.push_str(head);
        text.push_str(addition);
        text.push_str(tail);
        let cursor = cleared
            .caret
            .cursor()
            .saturating_add(addition.chars().count());
        Self {
            text,
            caret: TextCaret::new(cursor, cursor),
            preedit: None,
        }
    }

    /// カーソルを動かした状態。`extend` なら選択を広げ、そうでなければ畳む。
    #[must_use]
    pub fn moved(&self, to: CaretMove, extend: bool) -> Self {
        let caret = self.caret;
        let cursor = match to {
            CaretMove::Left if caret.has_selection() && !extend => caret.start(),
            CaretMove::Right if caret.has_selection() && !extend => caret.end(),
            CaretMove::Left => caret.cursor().saturating_sub(1),
            CaretMove::Right => caret.cursor().saturating_add(1).min(self.char_count()),
            CaretMove::Home => {
                let (line, _) = self.position_of(caret.cursor());
                self.index_of(line, 0)
            }
            CaretMove::End => {
                let (line, _) = self.position_of(caret.cursor());
                self.index_of(line, usize::MAX)
            }
            CaretMove::Up => self.line_neighbour(true),
            CaretMove::Down => self.line_neighbour(false),
        };
        let anchor = if extend { caret.anchor() } else { cursor };
        self.with_caret(TextCaret::new(cursor, anchor))
    }

    /// Backspace。選択があればそれを、無ければカーソルの左の 1 字を消した状態。
    #[must_use]
    pub fn backspaced(&self) -> Self {
        if self.caret.has_selection() {
            return self.without_selection();
        }
        let cursor = self.caret.cursor();
        self.removed(cursor.saturating_sub(1), cursor)
    }

    /// Delete。選択があればそれを、無ければカーソルの右の 1 字を消した状態。
    #[must_use]
    pub fn deleted(&self) -> Self {
        if self.caret.has_selection() {
            return self.without_selection();
        }
        let cursor = self.caret.cursor();
        self.removed(cursor, cursor.saturating_add(1).min(self.char_count()))
    }

    /// すべての字を選んだ状態。
    #[must_use]
    pub fn selected_all(&self) -> Self {
        self.with_caret(TextCaret::new(self.char_count(), 0))
    }

    /// 描く文字列と、その中のカーソル。確定前の文字列をカーソルの位置に差し込み、カーソルはその中へ移る。
    #[must_use]
    pub fn display(&self) -> (String, TextCaret) {
        let Some(preedit) = &self.preedit else {
            return (self.text.clone(), self.caret);
        };
        let cleared = self.without_selection();
        let base = cleared.caret.cursor();
        let shown = cleared.inserted(preedit.text());
        let cursor = base.saturating_add(preedit.cursor());
        let end = base.saturating_add(preedit.char_count());
        (
            shown.text,
            TextCaret::new(cursor, cursor).with_underline(base, end),
        )
    }

    /// 字の数。
    #[must_use]
    pub fn char_count(&self) -> usize {
        self.text.chars().count()
    }

    /// 行の数（空でも 1）。
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.text
            .chars()
            .filter(|character| *character == '\n')
            .count()
            .saturating_add(1)
    }

    /// 文字の添字がある行（0 から）と、行の中の桁（字の数）。
    #[must_use]
    pub fn position_of(&self, index: usize) -> (usize, usize) {
        let mut line = 0_usize;
        let mut column = 0_usize;
        for character in self.text.chars().take(index) {
            if character == '\n' {
                line = line.saturating_add(1);
                column = 0;
            } else {
                column = column.saturating_add(1);
            }
        }
        (line, column)
    }

    /// 行と桁の文字の添字。行が無ければ末尾、桁が行より長ければ行の末尾。
    #[must_use]
    pub fn index_of(&self, line: usize, column: usize) -> usize {
        let mut index = 0_usize;
        let mut current = 0_usize;
        for character in self.text.chars() {
            if current == line {
                break;
            }
            if character == '\n' {
                current = current.saturating_add(1);
            }
            index = index.saturating_add(1);
        }
        let length = self
            .text
            .chars()
            .skip(index)
            .take_while(|character| *character != '\n')
            .count();
        index.saturating_add(column.min(length))
    }

    fn with_caret(&self, caret: TextCaret) -> Self {
        Self {
            text: self.text.clone(),
            caret,
            preedit: self.preedit.clone(),
        }
    }

    /// 隣の行の同じ桁。最初の行から上は先頭、最後の行から下は末尾。
    fn line_neighbour(&self, upward: bool) -> usize {
        let (line, column) = self.position_of(self.caret.cursor());
        if upward {
            line.checked_sub(1)
                .map_or(0, |above| self.index_of(above, column))
        } else if line.saturating_add(1) < self.line_count() {
            self.index_of(line.saturating_add(1), column)
        } else {
            self.char_count()
        }
    }

    fn without_selection(&self) -> Self {
        if self.caret.has_selection() {
            self.removed(self.caret.start(), self.caret.end())
        } else {
            self.clone()
        }
    }

    /// 文字の添字 `start..end` を消し、カーソルを `start` に置いた状態。
    fn removed(&self, start: usize, end: usize) -> Self {
        let mut text = self.text.clone();
        text.replace_range(self.byte_offset(start)..self.byte_offset(end), "");
        Self {
            text,
            caret: TextCaret::new(start, start),
            preedit: self.preedit.clone(),
        }
    }

    /// 文字の添字をバイトの位置に直す。末尾を超えていれば末尾。
    fn byte_offset(&self, char_index: usize) -> usize {
        self.text
            .char_indices()
            .nth(char_index)
            .map_or(self.text.len(), |(offset, _)| offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed(edit: &TextEdit, text: &str) -> TextEdit {
        text.chars()
            .fold(edit.clone(), |edit, character| edit.typed(character))
    }

    #[test]
    fn typing_inserts_at_the_cursor_in_characters_not_bytes() {
        let edit = typed(&TextEdit::empty(), "日本x");
        assert_eq!(edit.text(), "日本x");
        assert_eq!(edit.caret(), TextCaret::new(3, 3));
        let middle = edit
            .moved(CaretMove::Left, false)
            .moved(CaretMove::Left, false);
        assert_eq!(typed(&middle, "語").text(), "日語本x");
        assert_eq!(edit.char_count(), 3);
    }

    #[test]
    fn backspace_and_delete_remove_one_character_or_the_selection() {
        let edit = TextEdit::showing("あいう");
        assert_eq!(edit.backspaced().text(), "あい");
        assert_eq!(edit.deleted().text(), "あいう");
        let home = edit.moved(CaretMove::Home, false);
        assert_eq!(home.deleted().text(), "いう");
        assert_eq!(home.backspaced().text(), "あいう");
        let selected = home
            .moved(CaretMove::Right, true)
            .moved(CaretMove::Right, true);
        assert_eq!(selected.caret(), TextCaret::new(2, 0));
        assert_eq!(selected.backspaced().text(), "う");
        assert_eq!(selected.deleted().text(), "う");
        assert_eq!(typed(&selected, "x").text(), "xう");
    }

    #[test]
    fn moving_collapses_a_selection_toward_the_moved_side() {
        let all = TextEdit::showing("abcd").selected_all();
        assert_eq!(all.caret(), TextCaret::new(4, 0));
        assert_eq!(
            all.moved(CaretMove::Left, false).caret(),
            TextCaret::new(0, 0)
        );
        assert_eq!(
            all.moved(CaretMove::Right, false).caret(),
            TextCaret::new(4, 4)
        );
        assert_eq!(
            all.moved(CaretMove::Left, true).caret(),
            TextCaret::new(3, 0)
        );
        assert_eq!(
            all.moved(CaretMove::End, false).caret(),
            TextCaret::new(4, 4)
        );
        let start = all.moved(CaretMove::Home, false);
        assert_eq!(
            start.moved(CaretMove::Left, false).caret(),
            TextCaret::new(0, 0)
        );
        assert_eq!(
            all.moved(CaretMove::Right, true).caret(),
            TextCaret::new(4, 0)
        );
    }

    #[test]
    fn the_preedit_is_shown_at_the_cursor_and_replaced_by_the_commit() {
        let edit = TextEdit::showing("ab").moved(CaretMove::Left, false);
        let composing = edit.with_preedit(Some(Preedit::new("にほ", 1)));
        let (shown, caret) = composing.display();
        assert_eq!(shown, "aにほb");
        assert_eq!(caret, TextCaret::new(2, 2).with_underline(1, 3));
        assert_eq!(composing.text(), "ab");
        assert_eq!(composing.preedit().map(Preedit::text), Some("にほ"));
        let committed = composing.inserted("日本");
        assert_eq!(committed.text(), "a日本b");
        assert_eq!(committed.preedit(), None);
        assert_eq!(committed.caret(), TextCaret::new(3, 3));
        assert_eq!(edit.display(), ("ab".to_owned(), TextCaret::new(1, 1)));
        assert_eq!(composing.with_preedit(None), edit);
    }

    #[test]
    fn the_preedit_replaces_the_selection_when_committed() {
        let selected = TextEdit::showing("abc").selected_all();
        let composing = selected.with_preedit(Some(Preedit::new("x", 1)));
        assert_eq!(composing.display().0, "x");
        assert_eq!(composing.inserted("y").text(), "y");
    }

    #[test]
    fn a_newline_character_is_a_line_break_inside_the_cell() {
        let edit = TextEdit::showing("ab").typed('\n').typed('c');
        assert_eq!(edit.text(), "ab\nc");
        assert_eq!(edit.line_count(), 2);
    }

    #[test]
    fn lines_and_columns_map_both_ways() {
        let edit = TextEdit::showing("ab\ncde\nf");
        assert_eq!(edit.line_count(), 3);
        assert_eq!(edit.position_of(edit.caret().cursor()), (2, 1));
        assert_eq!(edit.index_of(1, 1), 4);
        assert_eq!(edit.index_of(1, 9), 6);
        assert_eq!(edit.index_of(7, 0), 8);
        assert_eq!(TextEdit::empty().line_count(), 1);
    }

    #[test]
    fn home_and_end_move_within_the_current_line() {
        let up = TextEdit::showing("ab\ncde\nf").moved(CaretMove::Up, false);
        assert_eq!(up.caret(), TextCaret::new(4, 4));
        assert_eq!(
            up.moved(CaretMove::Home, false).caret(),
            TextCaret::new(3, 3)
        );
        assert_eq!(up.moved(CaretMove::End, true).caret(), TextCaret::new(6, 4));
    }

    #[test]
    fn walking_past_the_first_or_last_line_lands_on_the_ends() {
        let top = TextEdit::showing("ab\ncde\nf")
            .moved(CaretMove::Up, false)
            .moved(CaretMove::Up, false)
            .moved(CaretMove::Up, false);
        assert_eq!(top.caret(), TextCaret::new(0, 0));
        let bottom = top
            .moved(CaretMove::Down, false)
            .moved(CaretMove::Down, false)
            .moved(CaretMove::Down, false);
        assert_eq!(bottom.caret(), TextCaret::new(8, 8));
        let extended = top.moved(CaretMove::Down, true);
        assert_eq!(extended.caret(), TextCaret::new(3, 0));
    }

    #[test]
    fn a_line_break_inserted_at_the_start_of_a_line_keeps_the_rest() {
        let home = TextEdit::showing("ab\ncde\nf")
            .moved(CaretMove::Up, false)
            .moved(CaretMove::Home, false);
        let broken = home.typed('\n');
        assert_eq!(broken.text(), "ab\n\ncde\nf");
        assert_eq!(broken.caret(), TextCaret::new(4, 4));
        assert_eq!(broken.typed('x').text(), "ab\n\nxcde\nf");
    }

    /// 無作為の操作の列で、カーソルがいつも字の数の内にあり、バイトの途中で切れない（`replace_range` が落ちない）。
    #[test]
    fn random_operations_keep_the_caret_inside_the_text() {
        let mut state = 0x2545_f491_u32;
        let mut edit = TextEdit::showing("日本\nab");
        for _ in 0_u32..5_000 {
            state ^= state << 13_u32;
            state ^= state >> 17_u32;
            state ^= state << 5_u32;
            edit = match state % 11 {
                0 => edit.typed('語'),
                1 => edit.typed('\n'),
                2 => edit.backspaced(),
                3 => edit.deleted(),
                4 => edit.moved(CaretMove::Left, state & 0x100 != 0),
                5 => edit.moved(CaretMove::Right, state & 0x100 != 0),
                6 => edit.moved(CaretMove::Up, state & 0x100 != 0),
                7 => edit.moved(CaretMove::Down, state & 0x100 != 0),
                8 => edit.moved(CaretMove::Home, state & 0x100 != 0),
                9 => edit.with_preedit(Some(Preedit::new("か", 1))),
                _ => edit.inserted("x"),
            };
            let caret = edit.caret();
            assert!(caret.cursor() <= edit.char_count());
            assert!(caret.anchor() <= edit.char_count());
            // 確定前の文字列は選択を置き換えて描く（確定と同じ見え）。
            let selected = caret.end().saturating_sub(caret.start());
            let composing = edit.preedit().map_or(0, Preedit::char_count);
            let expected = if composing == 0 {
                edit.char_count()
            } else {
                edit.char_count() - selected + composing
            };
            let (shown, shown_caret) = edit.display();
            assert_eq!(shown.chars().count(), expected);
            assert!(shown_caret.cursor() <= expected);
        }
    }
}
