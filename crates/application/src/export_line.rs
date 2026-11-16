//! 書き出しの段の 1 行（ADR 0036 の決定 1）。

use crate::export_aspect::ExportAspect;

/// 書き出しの段の 1 行。形式の面（Enter か左右で値を替える）か、書き出す行（Enter で書き出す先を選ぶ）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportLine {
    /// 形式の面。
    Aspect(ExportAspect),
    /// この形式で書き出す。
    Write,
}

impl ExportLine {
    /// 全部の行（段に並べる順）。
    pub const ALL: [Self; 5] = [
        Self::Aspect(ExportAspect::Encoding),
        Self::Aspect(ExportAspect::Delimiter),
        Self::Aspect(ExportAspect::LineBreak),
        Self::Aspect(ExportAspect::Quotes),
        Self::Write,
    ];

    /// 行の名前。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Aspect(aspect) => aspect.name(),
            Self::Write => "この形式で書き出す…",
        }
    }
}
