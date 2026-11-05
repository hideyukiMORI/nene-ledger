//! 書き換えた行を組み立てられない理由。

/// 書き換えた行を組み立てられない理由。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowWriteFailure {
    /// その番号のセルが原本の行に無い（列の数が足りない行・OPEN-14）。
    NoSuchCell(usize),
    /// セルの番号が昇順でない（同じ番号が 2 度ある）。
    Unordered(usize),
}
