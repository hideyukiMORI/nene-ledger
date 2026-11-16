//! 書き出しの形式のうち、選ぶ面（ADR 0036 の決定 1）。

/// 書き出しの形式のうち、選ぶ面。閉じた集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportAspect {
    /// 文字コード（と BOM）。
    Encoding,
    /// 区切り。
    Delimiter,
    /// 行の終わり方。
    LineBreak,
    /// 引用符の付け方。
    Quotes,
}

impl ExportAspect {
    /// 全部の面（段に並べる順）。
    pub const ALL: [Self; 4] = [
        Self::Encoding,
        Self::Delimiter,
        Self::LineBreak,
        Self::Quotes,
    ];

    /// 面の名前。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Encoding => "文字コード",
            Self::Delimiter => "区切り",
            Self::LineBreak => "改行",
            Self::Quotes => "引用符",
        }
    }
}
