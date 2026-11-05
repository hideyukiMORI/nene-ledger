//! 書き換えた行を組み立てられない理由。

/// 書き換えた行を組み立てられない理由。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowWriteFailure {
    /// セルの番号が昇順でない（同じ番号が 2 度ある）。
    Unordered(usize),
}
