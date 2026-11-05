//! 書く前に、原本を置き換えられるかを確かめる（FILE-04・ADR 0025 の決定 2）。

use std::path::Path;

use nene_ledger_application::replace_failure::ReplaceFailure;

/// Windows: 原本を削除の権限（`DELETE`）とすべての共有の指定で開いてみる。ほかのアプリ（Excel など）が共有を許さずに
/// 開いていれば共有違反になるので、10 GB を書く前に知らせる。開いたハンドルはすぐ閉じる。
///
/// # Errors
///
/// 共有違反なら [`ReplaceFailure::Locked`]、権限が無ければ [`ReplaceFailure::Denied`]。
#[cfg(windows)]
pub(crate) fn probe(target: &Path) -> Result<(), ReplaceFailure> {
    use std::os::windows::fs::OpenOptionsExt;
    /// `DELETE`。
    const DELETE: u32 = 0x0001_0000;
    /// `FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE`。
    const SHARE_ALL: u32 = 0x0000_0001 | 0x0000_0002 | 0x0000_0004;
    /// `ERROR_SHARING_VIOLATION`・`ERROR_LOCK_VIOLATION`。
    const LOCKED: [i32; 2] = [32, 33];
    match std::fs::OpenOptions::new()
        .access_mode(DELETE)
        .share_mode(SHARE_ALL)
        .open(target)
    {
        Ok(handle) => {
            drop(handle);
            Ok(())
        }
        Err(error)
            if error
                .raw_os_error()
                .is_some_and(|code| LOCKED.contains(&code)) =>
        {
            Err(ReplaceFailure::Locked)
        }
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            Err(ReplaceFailure::Denied)
        }
        Err(_) => Ok(()),
    }
}
