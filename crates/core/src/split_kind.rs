//! 列の分け方の種類（ADR 0037 の決定 1）。

/// 列の分け方の種類。閉じた集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SplitKind {
    /// 文字どおりの区切り（`-`・空白・`／` など）。
    Literal,
    /// 正規表現に当たった所で分ける。
    Pattern,
    /// 固定幅（`3,5,2` のように字の数を並べる。残りは最後のセル）。
    Widths,
}

impl SplitKind {
    /// 次の種類（段で Tab を押すたびに替わる。最後からは最初へ回る）。
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Literal => Self::Pattern,
            Self::Pattern => Self::Widths,
            Self::Widths => Self::Literal,
        }
    }

    /// 種類の名前。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Literal => "文字どおり",
            Self::Pattern => "正規表現",
            Self::Widths => "固定幅",
        }
    }
}
