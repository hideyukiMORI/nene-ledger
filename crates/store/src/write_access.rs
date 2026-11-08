//! 開くときに、原本を保存で置き換えられるかを見る（FILE-04・GitHub Issue #240）。

use std::path::Path;

use nene_ledger_application::replace_failure::ReplaceFailure;

/// 原本 `path` を保存で置き換えられないなら、その理由。置き換えられそうなら `None`（保存の前にもう一度確かめる・
/// ADR 0025 の決定 2）。
///
/// - Windows：保存の前と同じ確かめ（`lock_probe`）。ほかのアプリ（Excel など）が共有を許さずに開いていれば
///   [`ReplaceFailure::Locked`]、権限が無ければ [`ReplaceFailure::Denied`]
/// - Unix：書き込みの権限が誰にも無ければ [`ReplaceFailure::Denied`]（開いているほかのプロセスがあっても `rename` で
///   置き換えられるので、ロックは見ない）
#[must_use]
pub fn write_blocker(path: &Path) -> Option<ReplaceFailure> {
    #[cfg(windows)]
    {
        crate::lock_probe::probe(path).err()
    }
    #[cfg(not(windows))]
    {
        std::fs::metadata(path)
            .ok()
            .filter(|metadata| metadata.permissions().readonly())
            .map(|_| ReplaceFailure::Denied)
    }
}
