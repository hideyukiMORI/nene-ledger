//! 行の後ろの余計なセルを消す 1 手（ADR 0024 の追記・GitHub Issue #413）。

use alloc::vec::Vec;

use nene_ledger_core::cell_edits::CellEdits;
use nene_ledger_core::cell_value::CellValue;
use nene_ledger_core::column_id::ColumnId;
use nene_ledger_core::row_id::RowId;

/// 原本の行 `row` の、見出しより後ろのセル（`cells`）を「セルが無い」にした 1 手。戻すために、消す前の差分の値
/// （書き換えていなければ `None`）を持つ。保存すると、残すセルまでで行を書く（`trimmed_row`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrimChange {
    row: RowId,
    cells: Vec<(ColumnId, Option<CellValue>)>,
}

impl TrimChange {
    /// 行 `row` の列 `columns` を消す手。消す前の値は差分 `edits` から覚える。
    #[must_use]
    pub fn new(row: RowId, columns: &[ColumnId], edits: &CellEdits) -> Self {
        Self {
            row,
            cells: columns
                .iter()
                .map(|&column| (column, edits.get(row, column).cloned()))
                .collect(),
        }
    }

    /// 消した行。
    #[must_use]
    pub const fn row(&self) -> RowId {
        self.row
    }

    /// 消したセルの数。
    #[must_use]
    pub fn count(&self) -> usize {
        self.cells.len()
    }

    /// 差分 `edits` で、セルを「セルが無い」にする。
    pub fn redo(&self, edits: &mut CellEdits) {
        for &(column, _) in &self.cells {
            let _previous = edits.set(self.row, column, CellValue::Absent);
        }
    }

    /// 差分 `edits` を、消す前の値に戻す。
    pub fn undo(&self, edits: &mut CellEdits) {
        for (column, before) in &self.cells {
            let _previous = match before {
                Some(value) => edits.set(self.row, *column, value.clone()),
                None => edits.clear(self.row, *column),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    #[test]
    fn trimming_marks_the_cells_missing_and_undo_restores_the_edits() {
        let row = RowId::Source(3);
        let mut edits = CellEdits::empty();
        edits.set(row, ColumnId::Source(4), CellValue::Text(String::from("x")));
        let change = TrimChange::new(row, &[ColumnId::Source(3), ColumnId::Source(4)], &edits);
        change.redo(&mut edits);
        assert_eq!(
            edits.get(row, ColumnId::Source(3)),
            Some(&CellValue::Absent)
        );
        assert_eq!(
            edits.get(row, ColumnId::Source(4)),
            Some(&CellValue::Absent)
        );
        change.undo(&mut edits);
        assert_eq!(edits.get(row, ColumnId::Source(3)), None);
        assert_eq!(
            edits.get(row, ColumnId::Source(4)),
            Some(&CellValue::Text(String::from("x")))
        );
        assert_eq!(change.count(), 2);
    }
}
