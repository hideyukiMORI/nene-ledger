//! ファイルを開けなかった理由。

use nene_ledger_scan::scan_failure::ScanFailure;

/// ファイルを開けなかった理由。黙って既定へ落とさず、理由を示す（FR-023）。
#[derive(Debug)]
pub(crate) enum OpenFailure {
    /// 読めなかった。
    Scan(ScanFailure),
}

impl From<ScanFailure> for OpenFailure {
    fn from(failure: ScanFailure) -> Self {
        Self::Scan(failure)
    }
}
