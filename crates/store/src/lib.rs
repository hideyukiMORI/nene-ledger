//! `NeNe Ledger` の書き出し。保存の計画（core の `SavePlan`）を、同じフォルダの一時ファイルに書き、同期してから
//! 原本の名前に置き換える（ADR 0005 の 6・ADR 0025）。
//!
//! 🔴 原本はポート（`SourceBytes`）越しに読む。scan には依存しない（`PROJECT_LAYOUT` 第 2 節）。
//! ファイルとプロセスの番号を読める区画の 1 つ（ARC-007・CNF-012）。

pub mod encoding;
#[cfg(windows)]
mod lock_probe;
pub mod rename_replacing;
pub mod save_failure;
pub mod save_job;
pub mod saving;
pub mod stamping;
mod temporary_file;
