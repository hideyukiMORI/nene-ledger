//! OS のポートが失敗した理由。

use std::io;
use std::path::PathBuf;

/// OS のポートが失敗した理由。閉じた集合（ARC-010）。黙って既定値へ落とさず、理由を示す（FR-023）。
#[derive(Debug)]
pub enum AdapterFailure {
    /// 設定の置き場を決められない（`XDG_CONFIG_HOME` も `HOME` も無い）。
    NoConfigDirectory,
    /// 設定のファイルが読めない形（知らない版・知らない鍵・読めない値）。
    BadSettingsFile(PathBuf),
    /// 読み書きに失敗した。
    Io(io::Error),
}

impl From<io::Error> for AdapterFailure {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
