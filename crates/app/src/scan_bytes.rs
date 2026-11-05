//! 保存が原本を読むために、scan の原本を位置を指定して読む（application の `SourceBytes` の実装）。

use std::io::ErrorKind;

use nene_ledger_application::source_bytes::SourceBytes;
use nene_ledger_scan::block_size::BlockSize;
use nene_ledger_scan::scan_failure::ScanFailure;
use nene_ledger_scan::source::Source;

/// 原本を、位置を指定して読む。
pub(crate) struct ScanBytes<'a> {
    source: &'a Source,
}

impl<'a> ScanBytes<'a> {
    /// 原本 `source` を読む。
    pub(crate) fn new(source: &'a Source) -> Self {
        Self { source }
    }
}

impl SourceBytes for ScanBytes<'_> {
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
}
