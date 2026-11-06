//! 原本のバイト列の在りか。

use std::fs::File;
use std::sync::Arc;

/// 原本のバイト列の在りか（`Source` の中身）。
#[derive(Debug)]
pub(crate) enum SourceOrigin {
    /// 読むだけで開いたファイル。
    File(File),
    /// メモリの中のバイト列（ファイルの無い新しい表の原本・ADR 0028 の決定 2）。
    Bytes(Arc<[u8]>),
}
