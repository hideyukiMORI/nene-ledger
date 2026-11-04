//! 表の中のセルの位置。

/// 表の中のセルの位置。縦は本体の何番目か（見出しの行は含まない・Issue #26）、横は列の番号（どちらも 0 始まり）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellPosition {
    body: u64,
    column: usize,
}

impl CellPosition {
    /// 位置を作る。
    #[must_use]
    pub fn new(body: u64, column: usize) -> Self {
        Self { body, column }
    }

    /// 本体の何番目の行か。
    #[must_use]
    pub fn body(self) -> u64 {
        self.body
    }

    /// 列の番号。
    #[must_use]
    pub fn column(self) -> usize {
        self.column
    }
}
