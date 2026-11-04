//! 設定を残せなかった理由。

/// 設定を残せなかった理由。閉じた集合（ARC-010）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsFailure {
    /// 設定の置き場に書けなかった。
    Unwritable,
}
