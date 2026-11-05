//! 列の並び（ADR 0024 の決定 2・EDIT-03）。行の並び（`RowOrder`）と同じ piece table で、同一性だけを `ColumnId` にする。
//!
//! 位置は表の列の順（0 始まり）。原本の列の数は行ごとに違ってよく、読むほど増える（OPEN-14）ので、最後の区間は
//! 「原本の終わりまで」のまま持つ。

use alloc::vec::Vec;

use crate::column_id::ColumnId;
use crate::order_run::OrderRun;
use crate::row_id::RowId;
use crate::row_order::RowOrder;

/// 列の並び。中身は行の並びと同じ区間の列（`OrderRun` の `Source` は原本の列、`Added` は足した列）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnOrder {
    runs: RowOrder,
}

impl ColumnOrder {
    /// 開いた直後の並び（原本の列の頭から終わりまで）。
    #[must_use]
    pub fn whole() -> Self {
        Self {
            runs: RowOrder::whole(),
        }
    }

    /// 開いた直後のままか。
    #[must_use]
    pub fn is_whole(&self) -> bool {
        self.runs.is_whole()
    }

    /// 区間の列。
    #[must_use]
    pub fn runs(&self) -> &[OrderRun] {
        self.runs.runs()
    }

    /// 列の数。原本の列の数 `source_columns` から数える。
    #[must_use]
    pub fn len(&self, source_columns: usize) -> usize {
        let source = u64::try_from(source_columns).unwrap_or(u64::MAX);
        usize::try_from(self.runs.len(source)).unwrap_or(usize::MAX)
    }

    /// `index` 番目の列（終わりまでの区間に落ちれば、原本の終わりを確かめずに返す）。
    #[must_use]
    pub fn column_at(&self, index: usize) -> Option<ColumnId> {
        self.runs.row_at(widen(index)).map(column_of)
    }

    /// 列 `column` の位置。並びに無ければ `None`。
    #[must_use]
    pub fn position_of(&self, column: ColumnId) -> Option<usize> {
        let position = self.runs.position_of(row_of(column))?;
        usize::try_from(position).ok()
    }

    /// `index` 番目の前に `count` 列を足し、足した列を返す。
    pub fn insert(&mut self, index: usize, count: usize) -> Vec<ColumnId> {
        self.runs
            .insert(widen(index), widen(count))
            .into_iter()
            .map(column_of)
            .collect()
    }

    /// `index` 番目から `count` 列を消し、消した区間を返す（[`Self::restore`] で戻せる）。
    pub fn remove(&mut self, index: usize, count: usize) -> Vec<OrderRun> {
        self.runs.remove(widen(index), widen(count))
    }

    /// 消した区間 `runs` を `index` 番目の前に戻す。
    pub fn restore(&mut self, index: usize, runs: Vec<OrderRun>) {
        self.runs.restore(widen(index), runs);
    }

    /// 頭から `count` 列（表の幅）の同一性。
    #[must_use]
    pub fn columns(&self, count: usize) -> Vec<ColumnId> {
        (0..count)
            .filter_map(|index| self.column_at(index))
            .collect()
    }
}

/// 行の同一性の形で持った列を、列の同一性にする。
fn column_of(row: RowId) -> ColumnId {
    match row {
        RowId::Source(column) => ColumnId::Source(usize::try_from(column).unwrap_or(usize::MAX)),
        RowId::Added(added) => ColumnId::Added(added),
    }
}

/// 列の同一性を、行の並びの区間で持つ形にする。
fn row_of(column: ColumnId) -> RowId {
    match column {
        ColumnId::Source(column) => RowId::Source(widen(column)),
        ColumnId::Added(added) => RowId::Added(added),
    }
}

/// `usize` を `u64` に広げる。
fn widen(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn columns_are_inserted_removed_and_restored_like_rows() {
        let mut order = ColumnOrder::whole();
        assert!(order.is_whole());
        let added = order.insert(1, 1);
        assert_eq!(added, [ColumnId::Added(0)]);
        assert_eq!(
            order.columns(4),
            [
                ColumnId::Source(0),
                ColumnId::Added(0),
                ColumnId::Source(1),
                ColumnId::Source(2)
            ]
        );
        assert_eq!(order.len(3), 4);
        assert_eq!(order.position_of(ColumnId::Source(2)), Some(3));
        let removed = order.remove(2, 1);
        assert_eq!(order.column_at(2), Some(ColumnId::Source(2)));
        order.restore(2, removed);
        let _gone = order.remove(1, 1);
        assert!(order.is_whole());
    }
}
