//! 原本の印。開いた後に外から変えられたかを、保存の前に確かめる（FILE-01・ADR 0025 の決定 2）。

/// 原本の印。大きさ・更新日時・ファイルの同一性（取れるときだけ）。値が 1 つでも違えば、外から変えられたとみなす。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileStamp {
    length: u64,
    modified: Option<(u64, u32)>,
    identity: Option<(u64, u64)>,
}

impl FileStamp {
    /// 印を作る。`modified` は 1970 年からの秒とナノ秒（取れなければ `None`）、`identity` は Unix の dev と inode
    /// （取れなければ `None`）。
    #[must_use]
    pub fn new(length: u64, modified: Option<(u64, u32)>, identity: Option<(u64, u64)>) -> Self {
        Self {
            length,
            modified,
            identity,
        }
    }

    /// 大きさ（バイト）。
    #[must_use]
    pub fn length(self) -> u64 {
        self.length
    }
}
