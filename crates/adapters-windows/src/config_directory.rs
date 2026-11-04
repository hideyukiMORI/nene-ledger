//! 設定の置き場。`%APPDATA%\nene-ledger\`（利用者ごとの移動するプロファイル）。

use std::env;
use std::path::PathBuf;

use crate::adapter_failure::AdapterFailure;

/// 設定の根の下の名前（X11 版と同じ）。
const DIRECTORY: &str = "nene-ledger";

/// 設定の置き場の `name` のファイルの道。フォルダはまだ作らない。
pub(crate) fn file_path(name: &str) -> Result<PathBuf, AdapterFailure> {
    let base = env::var_os("APPDATA")
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .ok_or(AdapterFailure::NoConfigDirectory)?;
    Ok(base.join(DIRECTORY).join(name))
}
