//! パレットの欄の短い知らせの種類（色を分ける・GitHub Issue #305）。

/// パレットの欄の短い知らせの種類。橙（強調）はフォーカスと確かめ、緑は済んだこと、赤は注意と誤り（専門家ペルソナの
/// 指摘「橙が確かめ・注意・完了・0 件のすべてに使われている」・#285）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteTone {
    /// 確かめ・進み具合・知らせ（強調の色）。
    Prompt,
    /// 済んだこと（緑）。
    Done,
    /// 注意と誤り（赤）。
    Problem,
}

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
