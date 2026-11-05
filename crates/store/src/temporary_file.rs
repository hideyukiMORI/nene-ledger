//! 原本と同じフォルダの一時ファイル。

use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// 名前がぶつかったときに番号を変えて試す回数。
const ATTEMPTS: u32 = 100;

/// `target` と同じフォルダに、新しい一時ファイルを作る（既にある名前は使わない）。
///
/// 名前は `.<原本の名前>.nene-ledger-<プロセスの番号>-<n>.tmp`。落ちた後に残っていても、どのファイルの保存の跡かが分かる。
/// 🔴 同じフォルダに置く。ボリュームが違うと置き換え（`rename`・`ReplaceFileW`）が使えない（ADR 0025 の却下）。
///
/// # Errors
///
/// 作れないとき。
pub(crate) fn create_beside(target: &Path) -> Result<(File, PathBuf), ErrorKind> {
    let folder = target.parent().ok_or(ErrorKind::InvalidInput)?;
    let name = target.file_name().ok_or(ErrorKind::InvalidInput)?;
    let process = std::process::id();
    for attempt in 0..ATTEMPTS {
        let mut file_name = std::ffi::OsString::from(".");
        file_name.push(name);
        file_name.push(format!(".nene-ledger-{process}-{attempt}.tmp"));
        let path = folder.join(file_name);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((file, path)),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.kind()),
        }
    }
    Err(ErrorKind::AlreadyExists)
}
