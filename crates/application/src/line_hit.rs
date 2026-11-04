//! 行番号・見出し・左上の角の、押した所。

/// 行番号・見出し・左上の角の、押した所（行全体・列全体・すべてを選ぶ）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineHit {
    /// 本体の行の行番号。
    Row(u64),
    /// 列の見出し。
    Column(usize),
    /// 左上の角。
    All,
}
