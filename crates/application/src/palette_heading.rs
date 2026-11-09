//! 何も打っていない操作の一覧の見出し（GitHub Issue #420）。

/// 何も打っていない操作の一覧の見出し。選べない行で、↑↓ は飛ばす。打って絞っている間は出さない。
/// 初めて使う人が「何ができるアプリか」を、分類の見出しで一目に分かるように（業界のペルソナと専門家の相談・2026-11-09）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteHeading {
    /// 最近使った操作。
    Recent,
    /// よく使う操作（固定）。
    Frequent,
    /// ファイル。
    File,
    /// 探す・置き換える。
    Find,
    /// 編集。
    Edit,
    /// 行と列。
    RowsColumns,
    /// 移動と選択。
    MoveSelect,
    /// 読み方・文字コード。
    Reading,
    /// 表示。
    View,
    /// ヘルプ。
    Help,
}

impl PaletteHeading {
    /// 分類の見出し（`Recent`・`Frequent` を除く）を、一覧に並べる順に。
    pub const SECTIONS: [Self; 8] = [
        Self::File,
        Self::Find,
        Self::Edit,
        Self::RowsColumns,
        Self::MoveSelect,
        Self::Reading,
        Self::View,
        Self::Help,
    ];

    /// 見出しの言葉。🔴 表示言語の文言の表（ADR 0011）を入れるまでの仮の日本語。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Recent => "最近使った操作",
            Self::Frequent => "よく使う",
            Self::File => "ファイル",
            Self::Find => "探す・置き換える",
            Self::Edit => "編集",
            Self::RowsColumns => "行と列",
            Self::MoveSelect => "移動と選択",
            Self::Reading => "読み方・文字コード（文字化けしたとき）",
            Self::View => "表示",
            Self::Help => "ヘルプ",
        }
    }
}
