//! `NeNe Ledger` の X11 の画面。
//!
//! `x11rb` の窓に、`cairo` の画面外の面（`ImageSurface`）で描き終えたフレームを転送する（ADR 0002 の決定 3・ADR 0016）。
//! 🔴 配置も表示の文字列も判断しない。application が作ったフレームを順に描くだけ（ARC-011）。

mod chrome_action;
mod cursor_shape;
mod frame_surface;
mod incoming_paste;
mod key_intent;
mod key_map;
mod paste_receipt;
mod pixels;
mod pointer;
mod pointer_at;
mod press;
mod screen;
mod step;
pub mod ui;
pub mod ui_failure;
mod viewport;
mod window_atoms;
mod x11_wakeup;
mod x11_window;
