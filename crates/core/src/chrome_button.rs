//! タイトルバーのボタン。

/// タイトルバーのボタン。右から閉じる・最大化・最小化の順に並ぶ（ADR 0013）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChromeButton {
    /// 最小化。
    Minimize,
    /// 最大化 / 元に戻す。
    Maximize,
    /// 閉じる。
    Close,
}

impl ChromeButton {
    /// 左から並ぶ順。
    pub const ALL: [Self; 3] = [Self::Minimize, Self::Maximize, Self::Close];
}
