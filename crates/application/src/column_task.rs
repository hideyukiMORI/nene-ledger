//! 列の分割と結合の段の仕事（ADR 0037）。

use nene_ledger_core::split_kind::SplitKind;

/// 列の分割と結合の段の仕事。欄に打った文字列を、分割なら分け方（種類は `SplitKind`）、結合ならつなぐ区切りとして読む。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColumnTask {
    /// カーソルの列を分ける。
    Split(SplitKind),
    /// 選んだ続いた列をつなぐ。
    Join,
}

impl ColumnTask {
    /// Tab で替えた仕事。分割は分け方の種類を次へ、結合はそのまま。
    #[must_use]
    pub const fn turned(self) -> Self {
        match self {
            Self::Split(kind) => Self::Split(kind.next()),
            Self::Join => Self::Join,
        }
    }

    /// 欄が空のときの案内。
    #[must_use]
    pub const fn placeholder(self) -> &'static str {
        match self {
            Self::Split(SplitKind::Literal) => "分ける区切りの字（例: - や空白）",
            Self::Split(SplitKind::Pattern) => "分ける所の正規表現（例: [-/年月]）",
            Self::Split(SplitKind::Widths) => "字の数をカンマで並べる（例: 3,5,2）",
            Self::Join => "つなぐ区切りの字（空のままでもよい）",
        }
    }

    /// 欄の下の案内の行。
    #[must_use]
    pub const fn hint(self) -> &'static str {
        match self {
            Self::Split(SplitKind::Literal) => {
                "Enter で右に列を足す（元の列は残る）· Tab で分け方：文字どおり"
            }
            Self::Split(SplitKind::Pattern) => {
                "Enter で右に列を足す（元の列は残る）· Tab で分け方：正規表現"
            }
            Self::Split(SplitKind::Widths) => {
                "Enter で右に列を足す（元の列は残る）· Tab で分け方：固定幅"
            }
            Self::Join => "Enter で、選んだ列をつないだ 1 列を右に足す（元の列は残る）",
        }
    }

    /// 見本の行が無いときに、候補の所に出す 1 行。
    #[must_use]
    pub const fn empty_note(self) -> &'static str {
        match self {
            Self::Split(_) => "打つと、見えている行の分け方をここに見せます",
            Self::Join => "区切りを打たなければ、そのままつなぎます",
        }
    }
}
