//! 未保存のまま閉じるときの選び方（ADR 0026 の決定 7）。

/// 尋ねる画面の 3 つのボタン。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseChoice {
    /// 保存して閉じる（S・Enter）。保存が終わってから閉じ、失敗したら閉じない。
    Save,
    /// 保存しないで閉じる（D）。書き換えを捨てる。
    Discard,
    /// やめる（Esc）。閉じない。
    Cancel,
}

impl CloseChoice {
    /// 左からの並び。
    pub const ALL: [Self; 3] = [Self::Save, Self::Discard, Self::Cancel];

    /// ボタンの文言（キーを添える）。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Save => "保存して閉じる (S)",
            Self::Discard => "保存しないで閉じる (D)",
            Self::Cancel => "やめる (Esc)",
        }
    }
}
