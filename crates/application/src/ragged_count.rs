//! ファイル全体の、列の数が見出しと違う行の数の状態（GitHub Issue #391）。

/// ファイル全体の、列の数が見出しと違う行の数の状態。索引を作り終えた後に、裏で 1 度だけ数える。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaggedCount {
    /// まだ数え始めていない。
    Unasked,
    /// 裏で数えている（数えられなかったときもここに留まり、何も出さない）。
    Counting,
    /// 数えた行の数。
    Counted(u64),
}
