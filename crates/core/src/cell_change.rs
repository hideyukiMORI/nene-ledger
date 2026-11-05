//! 1 つのセルの書き換え（元に戻す・やり直すの単位・ADR 0024 の決定 5・UNDO-01）。

use alloc::string::String;

use crate::column_id::ColumnId;
use crate::row_id::RowId;

/// 1 つのセルを、値 `before` から値 `after` に書き換えたこと。値は引用を外した文字列。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellChange {
    row: RowId,
    column: ColumnId,
    before: String,
    after: String,
}

impl CellChange {
    /// セル（`row`・`column`）を `before` から `after` に書き換えた。
    #[must_use]
    pub const fn new(row: RowId, column: ColumnId, before: String, after: String) -> Self {
        Self {
            row,
            column,
            before,
            after,
        }
    }

    /// 行。
    #[must_use]
    pub const fn row(&self) -> RowId {
        self.row
    }

    /// 列。
    #[must_use]
    pub const fn column(&self) -> ColumnId {
        self.column
    }

    /// 書き換える前の値。
    #[must_use]
    pub fn before(&self) -> &str {
        &self.before
    }

    /// 書き換えた後の値。
    #[must_use]
    pub fn after(&self) -> &str {
        &self.after
    }

    /// 逆の書き換え（元に戻す）。
    #[must_use]
    pub fn reversed(&self) -> Self {
        Self {
            row: self.row,
            column: self.column,
            before: self.after.clone(),
            after: self.before.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reversing_twice_gives_the_same_change() {
        let change = CellChange::new(
            RowId::Source(4),
            ColumnId::Source(1),
            String::from("0012"),
            String::from("13"),
        );
        let back = change.reversed();
        assert_eq!(back.before(), "13");
        assert_eq!(back.after(), "0012");
        assert_eq!(back.row(), RowId::Source(4));
        assert_eq!(back.column(), ColumnId::Source(1));
        assert_eq!(back.reversed(), change);
    }
}
