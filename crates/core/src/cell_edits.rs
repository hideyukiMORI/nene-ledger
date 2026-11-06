//! セルの書き換えの差分（ADR 0024 の層 2）。行と列は同一性で指す（ADR 0024 の決定 1）。

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::cell_value::CellValue;
use crate::column_id::ColumnId;
use crate::row_id::RowId;

/// セルの書き換えの疎な写像。記憶は書き換えたセルの数に比例する（BIG-13）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellEdits {
    cells: BTreeMap<(RowId, ColumnId), CellValue>,
}

impl CellEdits {
    /// 書き換えの無い差分。
    #[must_use]
    pub fn empty() -> Self {
        Self {
            cells: BTreeMap::new(),
        }
    }

    /// セルを書き換える。前の書き換えを返す（Undo の逆の操作に使う）。
    pub fn set(&mut self, row: RowId, column: ColumnId, value: CellValue) -> Option<CellValue> {
        self.cells.insert((row, column), value)
    }

    /// セルの書き換えを外す（原本の値に戻す）。外した書き換えを返す。
    pub fn clear(&mut self, row: RowId, column: ColumnId) -> Option<CellValue> {
        self.cells.remove(&(row, column))
    }

    /// セルの書き換え。無ければ `None`（原本の値のまま）。
    #[must_use]
    pub fn get(&self, row: RowId, column: ColumnId) -> Option<&CellValue> {
        self.cells.get(&(row, column))
    }

    /// 行 `row` のセルを 1 つでも書き換えているか。
    #[must_use]
    pub fn touches_row(&self, row: RowId) -> bool {
        self.cells
            .range((row, ColumnId::Source(0))..)
            .next()
            .is_some_and(|(&(found, _), _)| found == row)
    }

    /// 書き換えが 1 つも無いか。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// 書き換えた行ごとの、列と値の列。行の昇順（原本の行が先、足した行が後）、行の中は列の昇順。
    #[must_use]
    pub fn rows(&self) -> Vec<(RowId, Vec<(ColumnId, &CellValue)>)> {
        let mut rows: Vec<(RowId, Vec<(ColumnId, &CellValue)>)> = Vec::new();
        for (&(row, column), value) in &self.cells {
            match rows.last_mut() {
                Some((last, cells)) if *last == row => cells.push((column, value)),
                Some(_) | None => rows.push((row, alloc::vec![(column, value)])),
            }
        }
        rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    #[test]
    fn edits_are_grouped_by_row_in_order_and_can_be_undone() {
        let mut edits = CellEdits::empty();
        assert!(edits.is_empty());
        let text = |value: &str| CellValue::Text(String::from(value));
        assert_eq!(
            edits.set(RowId::Source(5), ColumnId::Source(2), text("b")),
            None
        );
        edits.set(RowId::Source(1), ColumnId::Source(0), text("a"));
        edits.set(RowId::Added(0), ColumnId::Source(0), CellValue::Absent);
        edits.set(RowId::Source(5), ColumnId::Source(0), text("c"));
        assert_eq!(
            edits.set(RowId::Source(5), ColumnId::Source(2), text("d")),
            Some(text("b"))
        );
        let rows = edits.rows();
        let order: Vec<RowId> = rows.iter().map(|(row, _)| *row).collect();
        assert_eq!(order, [RowId::Source(1), RowId::Source(5), RowId::Added(0)]);
        let columns: Vec<ColumnId> = rows[1].1.iter().map(|(column, _)| *column).collect();
        assert_eq!(columns, [ColumnId::Source(0), ColumnId::Source(2)]);
        assert_eq!(
            edits.clear(RowId::Source(1), ColumnId::Source(0)),
            Some(text("a"))
        );
        assert_eq!(edits.get(RowId::Source(1), ColumnId::Source(0)), None);
    }
}
