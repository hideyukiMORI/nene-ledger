//! 設定の置き場。`$XDG_CONFIG_HOME/nene-ledger/`、無ければ `$HOME/.config/nene-ledger/`。

use std::env;
use std::path::PathBuf;

use crate::adapter_failure::AdapterFailure;

/// 設定の根の下の名前。
const DIRECTORY: &str = "nene-ledger";

/// 設定の置き場の `name` のファイルの道。フォルダはまだ作らない。
pub(crate) fn file_path(name: &str) -> Result<PathBuf, AdapterFailure> {
    Ok(base()
        .ok_or(AdapterFailure::NoConfigDirectory)?
        .join(DIRECTORY)
        .join(name))
}

/// 利用者の設定の根（`$XDG_CONFIG_HOME`、無ければ `$HOME/.config`）。他のプログラムの設定（GTK）もここにある。
pub(crate) fn base() -> Option<PathBuf> {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
}
