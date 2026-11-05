//! 保存が原本を読むために、scan の原本を位置を指定して読む（application の `SourceBytes` の実装）。

use std::io::ErrorKind;

use std::path::PathBuf;

use nene_ledger_application::file_stamp::FileStamp;
use nene_ledger_application::source_bytes::SourceBytes;
use nene_ledger_scan::block_size::BlockSize;
use nene_ledger_scan::scan_failure::ScanFailure;
use nene_ledger_scan::source::Source;
use nene_ledger_store::stamping::stamp_of;

/// 原本を、位置を指定して読む。名前を付けて保存では、原本の道筋と開く前の印を持ち、置き換えの直前に比べる。
pub(crate) struct ScanBytes {
    source: Source,
    original: Option<(PathBuf, FileStamp)>,
}

impl ScanBytes {
    /// 原本 `source` を読む。
    pub(crate) fn new(source: Source) -> Self {
        Self {
            source,
            original: None,
        }
    }

    /// 原本 `source` を読み、置き換えの直前に、道筋 `path` の今の印が `opened` のままかを確かめる
    /// （名前を付けて保存・GitHub Issue #126）。
    pub(crate) fn guarded(source: Source, path: PathBuf, opened: FileStamp) -> Self {
        Self {
            source,
            original: Some((path, opened)),
        }
    }
}

impl SourceBytes for ScanBytes {
    type Failure = ErrorKind;

    fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> Result<usize, ErrorKind> {
        let Some(size) = BlockSize::new(buffer.len()) else {
            return Ok(0);
        };
        let block = self
            .source
            .read_block(offset, size)
            .map_err(|failure| match failure {
                ScanFailure::Open(kind)
                | ScanFailure::Read { kind, .. }
                | ScanFailure::Spawn(kind) => kind,
                ScanFailure::OutOfRange(_) | ScanFailure::WorkerPanicked => ErrorKind::InvalidInput,
            })?;
        let count = block.len().min(buffer.len());
        if let (Some(to), Some(from)) = (buffer.get_mut(..count), block.get(..count)) {
            to.copy_from_slice(from);
        }
        Ok(count)
    }

    fn unchanged(&mut self) -> bool {
        self.original
            .as_ref()
            .is_none_or(|(path, opened)| stamp_of(path).ok() == Some(*opened))
    }
}
