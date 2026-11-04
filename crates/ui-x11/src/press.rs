//! 左のボタンで押している所。

use nene_ledger_core::chrome_button::ChromeButton;

use crate::pointer_at::PointerAt;

/// 押している所。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Press {
    /// 押していない。
    Idle,
    /// タイトルバーを押した。
    Caption(PointerAt),
    /// ボタンを押した（離したときに同じボタンの上なら働く）。
    Button(ChromeButton),
}
