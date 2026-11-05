//! 編集を確定したときに、差分へすること（ADR 0026 の決定 2）。

use nene_ledger_core::cell_value::CellValue;
use nene_ledger_core::column_id::ColumnId;
use nene_ledger_core::row_id::RowId;

/// 編集を確定したときに、差分（`CellEdits`）へすること。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditCommit {
    /// 見えていた値のまま。差分も Undo の手も作らない。
    Keep,
    /// 原本の値に戻った。差分を消す（保存すればバイト列まで元に戻る）。
    Restore {
        /// 行。
        row: RowId,
        /// 列。
        column: ColumnId,
    },
    /// 差分に入れる。
    Set {
        /// 行。
        row: RowId,
        /// 列。
        column: ColumnId,
        /// 新しい値。
        value: CellValue,
    },
}
