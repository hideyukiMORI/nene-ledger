//! 窓の中の位置が当たる所。

use crate::chrome_button::ChromeButton;
use crate::resize_edge::ResizeEdge;

/// 窓の中の位置が当たる所。UI はこれを OS の領域（X11 はウィンドウマネージャーへの頼み）に写すだけ（ADR 0013 の決定 3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChromeHit {
    /// 中身（表と状態の行）。
    Client,
    /// タイトルバーのボタンの無い所。押すと移動、ダブルクリックで最大化。
    Caption,
    /// タイトルバーのボタン。
    Button(ChromeButton),
    /// 見えない縁。押すとリサイズ。
    Edge(ResizeEdge),
}
