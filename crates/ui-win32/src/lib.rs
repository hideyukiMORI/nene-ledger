//! `NeNe Ledger` の画面（Windows）。Win32 の窓に Direct2D / DirectWrite で描画命令を描き、キーとマウスを意図にする。
//!
//! 画面の状態は描く相手（`Session`）が持ち、ここは描いて起こされたら取り込むだけ（ARC-011）。X11 版（ui-x11）と同じ口の `Ui`。
//! unsafe を許す 2 クレートの 1 つ（ADR 0002 の 4）。Win32 を呼ぶ小さな関数ごとに `#[expect(unsafe_code, …)]` と `// SAFETY:`。
//! 🔴 Windows の実機ではまだ動かしていない（QLT-013・Issue #7）。キーとマウスの割り当てと当たり判定の写しは、どの OS でもテストする。

#[cfg(any(windows, test))]
mod composition;
#[cfg(any(windows, test))]
mod hit_code;
#[cfg(any(windows, test))]
mod key_intent;
#[cfg(any(windows, test))]
mod pixels;

#[cfg(windows)]
mod clipboard;
#[cfg(windows)]
pub mod desktop;
#[cfg(windows)]
mod dropped_files;
#[cfg(windows)]
mod ime;
#[cfg(windows)]
mod pen;
#[cfg(windows)]
mod timer_ticker;
#[cfg(windows)]
pub mod ui;
#[cfg(windows)]
pub mod win32_failure;
#[cfg(windows)]
mod win32_wakeup;
#[cfg(windows)]
mod window_state;
