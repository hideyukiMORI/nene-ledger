//! 詳細ペインのキーの種類（GitHub Issue #478）。

/// 詳細ペインのキーがすること。閉じた集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailKey {
    /// ペインを開閉する（Ctrl+Shift+U）。
    Toggle,
    /// 字を打つ先を表のセルと詳細ペインで切り替える（F6）。直していなければペインで直し始める（ペインが閉じていれば
    /// 開く）。確定はしない。
    SwitchTyping,
}
