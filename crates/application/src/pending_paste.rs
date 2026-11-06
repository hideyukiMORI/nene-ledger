//! 原本の行を読むのを待っている貼り付け（ADR 0029 の決定 1・2）。

use alloc::string::String;
use alloc::vec::Vec;

/// 貼る束と、貼る所（本体の行・表示の列）と大きさ。束が 1 つのセルで範囲が広ければ、範囲の全部に同じ値を書く。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingPaste {
    cells: Vec<Vec<String>>,
    top: u64,
    left: usize,
    rows: u64,
    columns: usize,
    fill: bool,
}

impl PendingPaste {
    /// 束 `cells` を本体の `top` 番目・表示の `left` 列目から、そのままの形で貼る。
    #[must_use]
    pub fn new(cells: Vec<Vec<String>>, (top, left): (u64, usize)) -> Self {
        let rows = u64::try_from(cells.len()).unwrap_or(u64::MAX);
        let columns = cells.iter().map(Vec::len).max().unwrap_or(0);
        Self {
            cells,
            top,
            left,
            rows,
            columns,
            fill: false,
        }
    }

    /// 1 つの値 `value` で、本体の `top` 番目・表示の `left` 列目から `rows` 行 × `columns` 列を埋める。
    #[must_use]
    pub fn filling(
        value: String,
        (top, left): (u64, usize),
        (rows, columns): (u64, usize),
    ) -> Self {
        Self {
            cells: alloc::vec![alloc::vec![value]],
            top,
            left,
            rows,
            columns,
            fill: true,
        }
    }

    /// 貼る最初の行（本体の番目）。
    #[must_use]
    pub const fn top(&self) -> u64 {
        self.top
    }

    /// 貼る最初の列（表示の列）。
    #[must_use]
    pub const fn left(&self) -> usize {
        self.left
    }

    /// 貼る行の数。
    #[must_use]
    pub const fn rows(&self) -> u64 {
        self.rows
    }

    /// 貼る列の数。
    #[must_use]
    pub const fn columns(&self) -> usize {
        self.columns
    }

    /// 貼る最後の行（本体の番目）。
    #[must_use]
    pub fn bottom(&self) -> u64 {
        self.top.saturating_add(self.rows.saturating_sub(1))
    }

    /// 貼るセルの数（行 × 列）。
    #[must_use]
    pub fn cell_count(&self) -> u128 {
        u128::from(self.rows) * u128::try_from(self.columns).unwrap_or(u128::MAX)
    }

    /// 束の `row` 行目・`column` 列目に書く値。束の行が短くて、その列のセルが無ければ `None`（書かない）。
    #[must_use]
    pub fn value(&self, row: u64, column: usize) -> Option<&str> {
        if self.fill {
            return self.cells.first()?.first().map(String::as_str);
        }
        let row = usize::try_from(row).ok()?;
        self.cells.get(row)?.get(column).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn a_ragged_bundle_writes_only_its_own_cells_and_a_fill_writes_everywhere() {
        let bundle = PendingPaste::new(
            vec![
                vec![String::from("a"), String::from("b")],
                vec![String::from("c")],
            ],
            (3, 1),
        );
        assert_eq!(
            (bundle.rows(), bundle.columns(), bundle.bottom()),
            (2, 2, 4)
        );
        assert_eq!(bundle.cell_count(), 4);
        assert_eq!(bundle.value(1, 0), Some("c"));
        assert_eq!(bundle.value(1, 1), None);
        let fill = PendingPaste::filling(String::from("x"), (0, 0), (3, 2));
        assert_eq!(fill.value(2, 1), Some("x"));
        assert_eq!((fill.rows(), fill.columns()), (3, 2));
    }
}
