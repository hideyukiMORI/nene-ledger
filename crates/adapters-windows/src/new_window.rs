//! 新しい表・落としたファイルを新しい窓で開く（ADR 0028 の決定 1・GitHub Issue #243）。自分の exe を起こし直す。

use std::path::Path;
use std::process::Command;

use crate::adapter_failure::AdapterFailure;

/// 自分の exe を `--new` で起こす（std の `Command` は `CreateProcessW`）。今の窓は残る（Excel・メモ帳と同じ）。
/// 子のハンドルは閉じるだけで待たない（Windows ではゾンビは残らない）。
///
/// # Errors
///
/// 自分の exe の道筋を取れない・起こせないとき。
pub fn open_new_window() -> Result<(), AdapterFailure> {
    let exe = std::env::current_exe()?;
    let _child = Command::new(exe).arg("--new").spawn()?;
    Ok(())
}

/// 自分の exe をファイル `path` を付けて起こす（落としたファイルを新しい窓で開く・CLIP-04・GitHub Issue #243）。
///
/// # Errors
///
/// 自分の exe の道筋を取れない・起こせないとき。
pub fn open_file_window(path: &Path) -> Result<(), AdapterFailure> {
    let exe = std::env::current_exe()?;
    let _child = Command::new(exe).arg(path).spawn()?;
    Ok(())
}
