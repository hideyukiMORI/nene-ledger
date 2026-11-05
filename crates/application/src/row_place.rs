//! 行を足す所（ADR 0027 の決定 2）。

/// 行を足す所。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowPlace {
    /// カーソルの行（行全体を選んでいれば選んだ行）の上に、選んだ行の数だけ。
    Above,
    /// 選んだ行の下に、選んだ行の数だけ。
    Below,
    /// 表の末尾に 1 行（索引を作り終えてから）。
    End,
}
