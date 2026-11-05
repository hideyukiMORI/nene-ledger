//! 元に戻す・やり直すの履歴（UNDO-01・ADR 0024 の決定 5）。
//!
//! 1 手は書き換えの束（今は確定 1 回で 1 つのセル。一括の入力・置換・貼り付けでは束が大きくなる）か、行の足し消し
//! （ADR 0027）。数は限らない（FR-006）。
//! 触ったセルの「ファイルの今の値」も覚えておき、戻した値がそれと同じなら差分を消す（保存すればバイト列まで元に戻る）。
//! 保存のときは、覚えたファイルの値を保存した値に進める。これで保存の後も履歴を残せる。

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use nene_ledger_core::cell_change::CellChange;
use nene_ledger_core::cell_edits::CellEdits;
use nene_ledger_core::cell_value::CellValue;
use nene_ledger_core::column_id::ColumnId;
use nene_ledger_core::row_id::RowId;
use nene_ledger_core::row_order::RowOrder;

use crate::edit_step::EditStep;
use crate::row_change::RowChange;

/// 元に戻す・やり直すの履歴と、触ったセルのファイルの値。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditHistory {
    done: Vec<EditStep>,
    undone: Vec<EditStep>,
    sources: BTreeMap<(RowId, ColumnId), String>,
}

impl EditHistory {
    /// 空の履歴。
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            done: Vec::new(),
            undone: Vec::new(),
            sources: BTreeMap::new(),
        }
    }

    /// 書き換え `change` を 1 手として積み、差分 `edits` に入れる。`source` はそのセルのファイルの今の値
    /// （初めて触ったときだけ覚える）。やり直しの列は捨てる。
    pub fn record(&mut self, change: CellChange, source: &str, edits: &mut CellEdits) {
        self.sources
            .entry((change.row(), change.column()))
            .or_insert_with(|| String::from(source));
        self.apply(&change, edits);
        self.done.push(EditStep::Cells(vec![change]));
        self.undone.clear();
    }

    /// 行の足し消し `change`（並びにはもう行った）を 1 手として積む。やり直しの列は捨てる。
    pub fn record_rows(&mut self, change: RowChange) {
        self.done.push(EditStep::Rows(change));
        self.undone.clear();
    }

    /// 1 手戻す。戻した手（セルなら戻した後の値への書き換え）を返す。戻すものが無ければ `None`。
    pub fn undo(&mut self, (edits, order): (&mut CellEdits, &mut RowOrder)) -> Option<EditStep> {
        let step = self.done.pop()?;
        let taken = match &step {
            EditStep::Cells(changes) => {
                let reversed: Vec<CellChange> =
                    changes.iter().rev().map(CellChange::reversed).collect();
                for change in &reversed {
                    self.apply(change, edits);
                }
                EditStep::Cells(reversed)
            }
            EditStep::Rows(change) => {
                change.undo(order);
                step.clone()
            }
        };
        self.undone.push(step);
        Some(taken)
    }

    /// 戻した 1 手をやり直す。やり直した手を返す。やり直すものが無ければ `None`。
    pub fn redo(&mut self, (edits, order): (&mut CellEdits, &mut RowOrder)) -> Option<EditStep> {
        let step = self.undone.pop()?;
        match &step {
            EditStep::Cells(changes) => {
                for change in changes {
                    self.apply(change, edits);
                }
            }
            EditStep::Rows(change) => change.redo(order),
        }
        self.done.push(step.clone());
        Some(step)
    }

    /// 行の足し消しの手を持つか。持っていれば、保存の後は原本のレコードの番号が変わるので履歴を残せない。
    #[must_use]
    pub fn has_row_steps(&self) -> bool {
        self.done
            .iter()
            .chain(&self.undone)
            .any(|step| matches!(step, EditStep::Rows(_)))
    }

    /// 保存した。触ったセルのファイルの値を、保存した値（差分があればその値）に進める。差分は呼ぶ側が空にする。
    pub fn saved(&mut self, edits: &CellEdits) {
        for ((row, column), source) in &mut self.sources {
            if let Some(CellValue::Text(value)) = edits.get(*row, *column) {
                source.clone_from(value);
            }
        }
    }

    /// 戻せる手の数。
    #[must_use]
    pub fn undo_count(&self) -> usize {
        self.done.len()
    }

    /// やり直せる手の数。
    #[must_use]
    pub fn redo_count(&self) -> usize {
        self.undone.len()
    }

    /// `change` の後の値を差分に入れる。ファイルの値と同じなら差分を消す。
    fn apply(&self, change: &CellChange, edits: &mut CellEdits) {
        let (row, column) = (change.row(), change.column());
        let same_as_file = self
            .sources
            .get(&(row, column))
            .is_some_and(|source| source == change.after());
        let _previous = if same_as_file {
            edits.clear(row, column)
        } else {
            edits.set(row, column, CellValue::Text(String::from(change.after())))
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn change(row: u64, before: &str, after: &str) -> CellChange {
        CellChange::new(
            RowId::Source(row),
            ColumnId::Source(0),
            String::from(before),
            String::from(after),
        )
    }

    fn value(edits: &CellEdits, row: u64) -> Option<&CellValue> {
        edits.get(RowId::Source(row), ColumnId::Source(0))
    }

    fn text(value: &str) -> CellValue {
        CellValue::Text(String::from(value))
    }

    #[test]
    fn undo_walks_back_and_undoing_to_the_file_value_removes_the_edit() {
        let mut history = EditHistory::empty();
        let mut edits = CellEdits::empty();
        history.record(change(1, "a", "b"), "a", &mut edits);
        history.record(change(1, "b", "c"), "a", &mut edits);
        assert_eq!(value(&edits, 1), Some(&text("c")));
        assert_eq!(
            history.undo((&mut edits, &mut RowOrder::whole())),
            Some(EditStep::Cells(vec![change(1, "c", "b")]))
        );
        assert_eq!(value(&edits, 1), Some(&text("b")));
        history.undo((&mut edits, &mut RowOrder::whole()));
        assert!(edits.is_empty());
        assert_eq!(history.undo((&mut edits, &mut RowOrder::whole())), None);
        assert_eq!(history.redo_count(), 2);
    }

    #[test]
    fn redo_walks_forward_until_a_new_edit_drops_the_rest() {
        let mut history = EditHistory::empty();
        let mut edits = CellEdits::empty();
        history.record(change(1, "a", "b"), "a", &mut edits);
        history.undo((&mut edits, &mut RowOrder::whole()));
        history.redo((&mut edits, &mut RowOrder::whole()));
        assert_eq!(value(&edits, 1), Some(&text("b")));
        history.undo((&mut edits, &mut RowOrder::whole()));
        // 新しく書き換えると、やり直しの列は消える。
        history.record(change(2, "x", "y"), "x", &mut edits);
        assert_eq!(history.redo_count(), 0);
        assert_eq!(history.redo((&mut edits, &mut RowOrder::whole())), None);
        assert_eq!(history.undo_count(), 1);
    }

    #[test]
    fn after_saving_undo_writes_the_old_value_and_back_to_the_saved_value_removes_it() {
        let mut history = EditHistory::empty();
        let mut before_saving = CellEdits::empty();
        history.record(change(1, "a", "b"), "a", &mut before_saving);
        // 保存した（ファイルの値は b になり、差分は空になった）。
        history.saved(&before_saving);
        let mut edits = CellEdits::empty();
        history.undo((&mut edits, &mut RowOrder::whole()));
        assert_eq!(value(&edits, 1), Some(&text("a")));
        history.redo((&mut edits, &mut RowOrder::whole()));
        assert!(edits.is_empty());
    }

    /// 無作為な数 `state` を進める（xorshift）。
    fn next(state: &mut u32) -> u32 {
        *state ^= *state << 13_u32;
        *state ^= *state >> 17_u32;
        *state ^= *state << 5_u32;
        *state
    }

    /// 無作為の書き換えの列を積んだ履歴と差分。
    fn random_history(state: &mut u32) -> (EditHistory, CellEdits) {
        let mut history = EditHistory::empty();
        let mut edits = CellEdits::empty();
        let mut shown: BTreeMap<u64, String> = BTreeMap::new();
        let steps = 1 + next(state) % 20;
        for _ in 0..steps {
            let roll = next(state);
            let row = u64::from(roll % 4);
            let file = alloc::format!("f{row}");
            let before = shown.get(&row).cloned().unwrap_or_else(|| file.clone());
            // 3 回に 1 回はファイルの値に戻す。
            let after = match roll % 3 {
                0 => file.clone(),
                other => alloc::format!("v{other}"),
            };
            if before != after {
                history.record(change(row, &before, &after), &file, &mut edits);
                shown.insert(row, after);
            }
        }
        (history, edits)
    }

    /// 無作為の書き換えの列を、全部戻すと差分が空に、全部やり直すと元の差分に戻る（ADR 0024 の強制）。
    #[test]
    fn undoing_everything_empties_the_edits_and_redoing_restores_them() {
        let mut state = 0x9e37_79b9_u32;
        for _ in 0_u32..500 {
            let (mut history, mut edits) = random_history(&mut state);
            let finished = edits.clone();
            while history.undo((&mut edits, &mut RowOrder::whole())).is_some() {}
            assert!(edits.is_empty(), "{edits:?}");
            while history.redo((&mut edits, &mut RowOrder::whole())).is_some() {}
            assert_eq!(edits, finished);
        }
    }
}
