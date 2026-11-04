//! 表示だけの動かし方。

/// 表示だけの動かし方（ホイール）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scroll {
    /// `rows` 行上へ。
    Up {
        /// 動かす行の数。
        rows: u64,
    },
    /// `rows` 行下へ。
    Down {
        /// 動かす行の数。
        rows: u64,
    },
    /// `columns` 列左へ。
    Left {
        /// 動かす列の数。
        columns: usize,
    },
    /// `columns` 列右へ。
    Right {
        /// 動かす列の数。
        columns: usize,
    },
}
