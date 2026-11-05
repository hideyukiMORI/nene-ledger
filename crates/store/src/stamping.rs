//! 原本の印を取る（FILE-01・ADR 0025 の決定 2）。

use std::io::ErrorKind;
use std::path::Path;
use std::time::UNIX_EPOCH;

use nene_ledger_application::file_stamp::FileStamp;

/// `path`（シンボリックリンクなら行き先）の今の印。開く前に取り、保存の前と置き換えの直前に同じ関数で取り直して比べる。
///
/// # Errors
///
/// 原本の情報を読めないとき。
pub fn stamp_of(path: &Path) -> Result<FileStamp, ErrorKind> {
    let metadata = std::fs::metadata(path).map_err(|error| error.kind())?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|since| (since.as_secs(), since.subsec_nanos()));
    #[cfg(unix)]
    let identity = Some(unix_identity(&metadata));
    // 🔴 Windows のファイル ID は std では読めない（`windows_by_handle` は不安定）。大きさと更新日時だけで比べる。
    #[cfg(not(unix))]
    let identity = None;
    Ok(FileStamp::new(metadata.len(), modified, identity))
}

/// Unix の dev と inode。
#[cfg(unix)]
fn unix_identity(metadata: &std::fs::Metadata) -> (u64, u64) {
    use std::os::unix::fs::MetadataExt;
    (metadata.dev(), metadata.ino())
}
