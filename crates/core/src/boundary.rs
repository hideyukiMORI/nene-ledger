//! セルの後ろに何が来たか。

use crate::line_ending::LineEnding;

/// セルの終わりに来たもの。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Boundary {
    /// 区切り。次のセルが続く。
    Delimiter,
    /// 改行。レコードが終わる。
    Terminator(LineEnding),
    /// 入力の終わり。
    End,
}
