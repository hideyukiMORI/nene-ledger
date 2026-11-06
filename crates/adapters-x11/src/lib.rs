//! `NeNe Ledger` の X11（Linux）の OS のポート。
//!
//! 環境とファイルと時刻とスレッドを持てる区画の 1 つ（ARC-007・CNF-012）。取得した値はその場で core と application の型へ変える（ARC-008）。

pub mod adapter_failure;
pub mod appearance;
mod config_directory;
pub mod new_window;
pub mod saved_settings;
pub mod thread_ticker;
mod tick_shared;
mod tick_state;
