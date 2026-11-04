//! 文字の種類。

/// 文字の種類。どちらの字の大きさで書くかを UI に伝える（ADR 0020）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextRole {
    /// 窓の部品（タイトルバー・状態の行）。拡大では変えない。
    Chrome,
    /// 中身（表・詳細ペイン）。利用者の拡大に従う。
    Content,
}
