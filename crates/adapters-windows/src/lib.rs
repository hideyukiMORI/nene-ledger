//! `NeNe Ledger` の Windows の OS のポート。
//!
//! 環境とファイルと時刻とスレッドを持てる区画の 1 つ（ARC-007・CNF-012）。取得した値はその場で core と application の型へ変える（ARC-008）。
//! X11 版（adapters-x11）と同じ名前の口を持ち、合成ルートは OS ごとに取り替えるだけ（Issue #55）。
//! 🔴 Windows でだけ中身がある（他の OS では空のクレート）。

#![cfg(windows)]

pub mod adapter_failure;
pub mod appearance;
mod config_directory;
pub mod file_replacing;
pub mod new_window;
pub mod saved_settings;
pub mod text_scale;
