//! 列を足す所（ADR 0027 の決定 2）。

/// 列を足す所。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnPlace {
    /// カーソルの列（列全体を選んでいれば選んだ列）の左に、選んだ列の数だけ。
    Left,
    /// 選んだ列の右に、選んだ列の数だけ。
    Right,
}
