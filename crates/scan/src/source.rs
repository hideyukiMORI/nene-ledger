//! 読むだけで開いた原本。

use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use std::path::Path;
use std::sync::Arc;

use nene_ledger_core::text_encoding::TextEncoding;

use crate::block_size::BlockSize;
use crate::scan_failure::ScanFailure;
use crate::source_origin::SourceOrigin;

/// 読むだけで開いた原本。位置を指定して塊を読む（ADR 0017 の決定 1・4）。
///
/// 🔴 書き込みの API を使わない（ADR 0005・CNF-015）。Windows では読み・書き・削除を他に許して開き、
/// Excel などの保存を妨げない。そのぶん、開いている間に中身が変わりうる（検出は FR-019）。
#[derive(Debug)]
pub struct Source {
    origin: SourceOrigin,
    len: u64,
}

impl Source {
    /// `path` を読むだけで開く。
    ///
    /// 🔴 普通のファイルだけを開く。デバイス（`/dev/zero`）や名前つきの管は大きさが分からず、読み出しが空に
    /// ならないものもあるので、索引が終わらない（GitHub Issue #25）。
    ///
    /// # Errors
    ///
    /// 開けない・大きさを読めない・普通のファイルでない（`ErrorKind::InvalidInput`）とき。
    pub fn open(path: &Path) -> Result<Self, ScanFailure> {
        let file = reading_options()
            .open(path)
            .map_err(|error| ScanFailure::Open(error.kind()))?;
        let metadata = file
            .metadata()
            .map_err(|error| ScanFailure::Open(error.kind()))?;
        if !metadata.is_file() {
            return Err(ScanFailure::Open(ErrorKind::InvalidInput));
        }
        Ok(Self {
            origin: SourceOrigin::File(file),
            len: metadata.len(),
        })
    }

    /// ファイルの無い原本。中身はメモリの `bytes`（新しい表は BOM だけ・ADR 0028 の決定 2・3）。
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let len = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        Self {
            origin: SourceOrigin::Bytes(Arc::from(bytes)),
            len,
        }
    }

    /// 同じ原本を別に読む口（裏の仕事に渡す）。位置を指定して読むので、読む位置を共有しない。
    ///
    /// # Errors
    ///
    /// OS が複製を拒んだとき。
    pub fn try_clone(&self) -> Result<Self, ScanFailure> {
        let origin = match &self.origin {
            SourceOrigin::File(file) => SourceOrigin::File(
                file.try_clone()
                    .map_err(|error| ScanFailure::Open(error.kind()))?,
            ),
            SourceOrigin::Bytes(bytes) => SourceOrigin::Bytes(Arc::clone(bytes)),
        };
        Ok(Self {
            origin,
            len: self.len,
        })
    }

    /// 開いたときの大きさ。
    #[must_use]
    pub fn len(&self) -> u64 {
        self.len
    }

    /// 開いたときに空だったか。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 最初のレコードが始まる位置。原本が `encoding` の BOM で始まっていれば BOM の後ろ、そうでなければ 0。
    ///
    /// BOM は切り出しに含めない（先頭のセルの引用を壊さない）。原本からは消さない（KEEP-04）。
    ///
    /// # Errors
    ///
    /// OS が読み出しを拒んだとき。
    pub fn content_start(&self, encoding: TextEncoding) -> Result<u64, ScanFailure> {
        let bom = encoding.bom();
        let Some(size) = BlockSize::new(bom.len()) else {
            return Ok(0);
        };
        let head = self.read_block(0, size)?;
        if head == bom {
            Ok(u64::try_from(bom.len())?)
        } else {
            Ok(0)
        }
    }

    /// `offset` から最大 `size` バイトを読む。短いのはファイルの終わりに着いたときだけ。
    ///
    /// # Errors
    ///
    /// OS が読み出しを拒んだとき。
    pub fn read_block(&self, offset: u64, size: BlockSize) -> Result<Vec<u8>, ScanFailure> {
        let mut block = vec![0_u8; size.get()];
        let mut filled = 0_usize;
        while let Some(rest) = block.get_mut(filled..).filter(|rest| !rest.is_empty()) {
            let at = offset.saturating_add(u64::try_from(filled)?);
            match self.read_at(rest, at) {
                Ok(0) => break,
                Ok(read) => filled = filled.saturating_add(read),
                Err(error) if error.kind() == ErrorKind::Interrupted => {}
                Err(error) => {
                    return Err(ScanFailure::Read {
                        offset: at,
                        kind: error.kind(),
                    });
                }
            }
        }
        block.truncate(filled);
        Ok(block)
    }

    fn read_at(&self, buffer: &mut [u8], offset: u64) -> std::io::Result<usize> {
        match &self.origin {
            SourceOrigin::File(file) => read_file_at(file, buffer, offset),
            SourceOrigin::Bytes(bytes) => {
                let start = usize::try_from(offset).unwrap_or(usize::MAX);
                let rest = bytes.get(start..).unwrap_or_default();
                let count = rest.len().min(buffer.len());
                if let (Some(to), Some(from)) = (buffer.get_mut(..count), rest.get(..count)) {
                    to.copy_from_slice(from);
                }
                Ok(count)
            }
        }
    }
}

#[cfg(unix)]
fn read_file_at(file: &File, buffer: &mut [u8], offset: u64) -> std::io::Result<usize> {
    std::os::unix::fs::FileExt::read_at(file, buffer, offset)
}

#[cfg(windows)]
fn read_file_at(file: &File, buffer: &mut [u8], offset: u64) -> std::io::Result<usize> {
    std::os::windows::fs::FileExt::seek_read(file, buffer, offset)
}

/// 読むだけの開き方。
#[cfg(unix)]
fn reading_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.read(true);
    options
}

/// 読むだけの開き方。他のアプリの読み・書き・削除（置き換えの保存を含む）を妨げない（ADR 0017 の決定 4）。
#[cfg(windows)]
fn reading_options() -> OpenOptions {
    use std::os::windows::fs::OpenOptionsExt;
    /// `FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE`。
    const SHARE_ALL: u32 = 0x0000_0001 | 0x0000_0002 | 0x0000_0004;
    let mut options = OpenOptions::new();
    options.read(true).share_mode(SHARE_ALL);
    options
}

#[cfg(test)]
mod tests {
    use super::*;

    /// メモリの原本は、ファイルの原本と同じく位置を指定して読め、終わりで短くなり、BOM を見分ける。
    #[test]
    fn a_source_in_memory_reads_like_a_file() {
        let source = Source::from_bytes(b"\xEF\xBB\xBFa,b\r\n");
        assert_eq!(source.len(), 8);
        assert!(!source.is_empty());
        let block = BlockSize::new(4).unwrap();
        assert_eq!(source.read_block(0, block).unwrap(), b"\xEF\xBB\xBFa");
        assert_eq!(source.read_block(6, block).unwrap(), b"\r\n");
        assert_eq!(source.read_block(99, block).unwrap(), b"");
        assert_eq!(source.content_start(TextEncoding::Utf8WithBom).unwrap(), 3);
        let copy = source.try_clone().unwrap();
        assert_eq!(copy.read_block(3, block).unwrap(), b"a,b\r");
        assert!(Source::from_bytes(b"").is_empty());
    }
}
