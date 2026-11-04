//! 文字の大きさの変え方。

/// 文字の大きさの変え方（ADR 0020）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomStep {
    /// 1 段大きく（Ctrl+ホイールの上・Ctrl++）。
    In,
    /// 1 段小さく（Ctrl+ホイールの下・Ctrl+-）。
    Out,
    /// 100% に戻す（Ctrl+0）。
    Reset,
}
