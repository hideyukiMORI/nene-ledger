//! 保存に渡す、原本への差分の全部（ADR 0024 の層 1 と層 2・ADR 0027）。

use nene_ledger_core::cell_edits::CellEdits;
use nene_ledger_core::column_order::ColumnOrder;
use nene_ledger_core::row_order::RowOrder;

use crate::added_row_shape::AddedRowShape;

/// 保存に渡す差分。行の並び・列の並び・セルの書き換え・足した行の書き方。
#[derive(Debug, Clone, Copy)]
pub struct TableChanges<'changes> {
    order: &'changes RowOrder,
    columns: &'changes ColumnOrder,
    edits: &'changes CellEdits,
    shape: AddedRowShape,
}

impl<'changes> TableChanges<'changes> {
    /// 行の並びと列の並び `(order, columns)`・セルの書き換え `edits`・足した行の書き方 `shape`
    /// （セルの数は表の列の数）。
    #[must_use]
    pub const fn new(
        (order, columns): (&'changes RowOrder, &'changes ColumnOrder),
        edits: &'changes CellEdits,
        shape: AddedRowShape,
    ) -> Self {
        Self {
            order,
            columns,
            edits,
            shape,
        }
    }

    /// 列の並び。
    #[must_use]
    pub const fn columns(&self) -> &'changes ColumnOrder {
        self.columns
    }

    /// 行の並び。
    #[must_use]
    pub const fn order(&self) -> &'changes RowOrder {
        self.order
    }

    /// セルの書き換え。
    #[must_use]
    pub const fn edits(&self) -> &'changes CellEdits {
        self.edits
    }

    /// 足した行の書き方。
    #[must_use]
    pub const fn shape(&self) -> AddedRowShape {
        self.shape
    }
}
