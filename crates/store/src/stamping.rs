//! 原本の印を取る（FILE-01・ADR 0025 の決定 2）。

use std::io::ErrorKind;
use std::path::Path;
use std::time::UNIX_EPOCH;

use nene_ledger_application::file_stamp::FileStamp;
use nene_ledger_application::save_target::SaveTarget;

/// 名前を付けて保存の保存先 `path` の今の様子（ADR 0028 の決定 5）。無ければ [`SaveTarget::Absent`]、あれば今の印。
///
/// # Errors
///
/// 情報を読めないとき。フォルダなら `ErrorKind::IsADirectory`（ファイルで置き換えない）。
pub fn target_state(path: &Path) -> Result<SaveTarget, ErrorKind> {
    match std::fs::metadata(path) {
        Ok(metadata) if metadata.is_dir() => Err(ErrorKind::IsADirectory),
        Ok(_) => stamp_of(path).map(SaveTarget::Existing),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(SaveTarget::Absent),
        Err(error) => Err(error.kind()),
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    /// 保存先の様子：無ければ `Absent`、ファイルなら今の印、フォルダは拒む（ADR 0028 の決定 5）。
    #[test]
    fn the_target_state_tells_absent_existing_and_folders_apart() {
        let folder =
            std::env::temp_dir().join(format!("nene-ledger-target-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let file = folder.join("t.csv");
        drop(std::fs::remove_file(&file));
        assert_eq!(target_state(&file), Ok(SaveTarget::Absent));
        std::fs::write(&file, b"a").unwrap();
        assert_eq!(
            target_state(&file),
            Ok(SaveTarget::Existing(stamp_of(&file).unwrap()))
        );
        assert_eq!(target_state(&folder), Err(ErrorKind::IsADirectory));
        drop(std::fs::remove_dir_all(&folder));
    }
}
