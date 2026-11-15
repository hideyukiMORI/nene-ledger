//! 続けた押しの何度目か（GitHub Issue #468）。

/// 左のボタンを続けて押した何度目か。決めるのは UI（OS のダブルクリックの間と距離）。閉じた集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickCount {
    /// 2 度目（ダブルクリック）。本体のセルなら直し始める・詳細ペインの本文なら語を選ぶ。
    Second,
    /// 3 度目（トリプルクリック）。詳細ペインの本文なら行を選ぶ。それ以外は 1 度押したのと同じ。
    Third,
}
