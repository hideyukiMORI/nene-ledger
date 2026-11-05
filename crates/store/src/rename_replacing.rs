//! `rename` による置き換え（Linux・ADR 0025 の決定 4）。

use std::io::ErrorKind;
use std::path::Path;

use nene_ledger_application::replace_failure::ReplaceFailure;
use nene_ledger_application::replacing::Replacing;

/// `rename` で置き換え、Unix ではフォルダも同期する。同じフォルダの中の `rename` は原子的で、途中で落ちても
/// 元の名前には原本か新しい内容のどちらかがある。🔴 Windows では使わない（ACL と名前つきストリームを引き継がない・
/// 施主の判断 1。合成ルートが adapters-windows の `ReplaceFileW` を選ぶ）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenameReplacing;

impl Replacing for RenameReplacing {
    type Location = Path;

    fn replace(&self, replacement: &Path, target: &Path) -> Result<(), ReplaceFailure> {
        std::fs::rename(replacement, target).map_err(|error| failure_of(error.kind()))?;
        sync_folder(target);
        Ok(())
    }
}

/// `rename` の失敗の種類。
fn failure_of(kind: ErrorKind) -> ReplaceFailure {
    if kind == ErrorKind::PermissionDenied || kind == ErrorKind::ReadOnlyFilesystem {
        ReplaceFailure::Denied
    } else if kind == ErrorKind::ResourceBusy {
        ReplaceFailure::Locked
    } else {
        ReplaceFailure::Other
    }
}

/// 名前の変更を記憶装置に届ける（Unix）。失敗しても、置き換えそのものは済んでいるので知らせない。
#[cfg(unix)]
fn sync_folder(target: &Path) {
    if let Some(folder) = target.parent()
        && let Ok(handle) = std::fs::File::open(folder)
    {
        drop(handle.sync_all());
    }
}

/// Unix の外ではフォルダを開いて同期できない（この経路は Windows では使わない）。
#[cfg(not(unix))]
fn sync_folder(_target: &Path) {}
