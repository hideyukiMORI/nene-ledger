//! 見えの明暗。OS の設定と配色の表が持つ（ADR 0012）。

/// ライトかダークか。閉じた集合（RS-002）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Appearance {
    /// 明るい背景に暗い字。
    Light,
    /// 暗い背景に明るい字。
    Dark,
}
