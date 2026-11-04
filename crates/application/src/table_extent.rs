//! 表の大きさ。

/// 表の大きさ。本体の行の数（索引を作り終えるまで分からない）と列の数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableExtent {
    rows: Option<u64>,
    columns: usize,
}

impl TableExtent {
    /// 大きさを作る。
    #[must_use]
    pub fn new(rows: Option<u64>, columns: usize) -> Self {
        Self { rows, columns }
    }

    /// 行の数が分かっている大きさを作る（定数の式で使える形）。
    #[must_use]
    pub const fn new_const(rows: u64, columns: usize) -> Self {
        Self {
            rows: Some(rows),
            columns,
        }
    }

    /// 本体の行の数（分かっていれば）。
    #[must_use]
    pub fn rows(self) -> Option<u64> {
        self.rows
    }

    /// 列の数。
    #[must_use]
    pub fn columns(self) -> usize {
        self.columns
    }
}
