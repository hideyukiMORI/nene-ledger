//! 表の行の描き方。

/// 表の行の描き方。見出しの行は窓の上に固定し、本体の行と色で分ける（VIEW-01）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RowStyle {
    /// 固定した見出しの行。
    Header,
    /// 見出しの行が無い表の、列の番号の行（Issue #48）。行番号の列は空ける。
    Numbers,
    /// スクロールする本体の、何番目かの行（選択の判定に使う）。
    Body(u64),
}
