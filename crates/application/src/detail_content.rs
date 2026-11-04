//! 詳細ペインに出す中身。

/// 詳細ペインに出す、カーソルのセルの中身。空のセルとセルが無いを区別する（RS-004・KEEP-05）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailContent<'row> {
    /// カーソルの行が見えている範囲の外（読んでいない）。
    OutOfView,
    /// カーソルの行にその列のセルが無い。
    Missing,
    /// セルの値（復号したまま。空のこともある）。
    Value(&'row str),
}
