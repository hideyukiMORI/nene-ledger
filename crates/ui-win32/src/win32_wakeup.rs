//! 作業のスレッドから画面のスレッドを起こす口（`PostMessageW`）。

use nene_ledger_application::wakeup::Wakeup;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_APP};

/// 起こす合図のメッセージ。
pub(crate) const WM_WAKE: u32 = WM_APP + 1;

/// 作業のスレッドから画面のスレッドを起こす口。窓に合図を積むだけで、状態は渡さない（ADR 0014 の決定 3）。
/// 窓の取っ手はスレッドをまたいで持てる数として持つ（`HWND` そのものは送れない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Win32Wakeup {
    window: isize,
}

impl Win32Wakeup {
    /// 窓 `window` を起こす口。
    pub(crate) fn new(window: HWND) -> Self {
        Self {
            window: window.0.expose_provenance().cast_signed(),
        }
    }
}

impl Wakeup for Win32Wakeup {
    #[expect(
        unsafe_code,
        reason = "RS-010: Win32 の PostMessageW を呼ぶ（ADR 0002 の 4）"
    )]
    fn wake(&self) {
        let window = HWND(core::ptr::with_exposed_provenance_mut(
            self.window.cast_unsigned(),
        ));
        // SAFETY: 窓の取っ手と値だけを渡す。窓が無くなっていれば失敗するだけ（合図は捨ててよい）。
        let _posted: Result<(), _> =
            unsafe { PostMessageW(Some(window), WM_WAKE, WPARAM(0), LPARAM(0)) };
    }
}
