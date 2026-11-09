//! パレットの欄に重ねる知らせ（GitHub Issue #305）。

use crate::note_tone::NoteTone;

/// パレットの欄に重ねる知らせ：1 行目の短い知らせ `text` とその種類 `tone`、条件の札の代わりの内訳の行 `lines`
/// （注意は `true`・GitHub Issue #289）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuideNote<'note> {
    /// 1 行目の短い知らせ（置換の段）か、欄の右端の知らせ（探す段）。
    pub text: Option<&'note str>,
    /// 知らせの種類（色）。
    pub tone: NoteTone,
    /// 全部の置換の内訳の行。
    pub lines: &'note [(alloc::string::String, bool)],
}
