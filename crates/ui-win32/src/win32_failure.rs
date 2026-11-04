//! Windows の画面側で起こりうる失敗。

use core::fmt;

/// Windows の画面側で起こりうる失敗。閉じた集合（ARC-010）。
#[derive(Debug)]
pub enum Win32Failure {
    /// Win32・Direct2D・DirectWrite の呼び出しが失敗した。
    Api(windows::core::Error),
    /// 大きさや数が Win32 の範囲を超えた。
    OutOfRange,
}

impl fmt::Display for Win32Failure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Api(error) => write!(formatter, "Windows API failed: {error}"),
            Self::OutOfRange => formatter.write_str("a size exceeds the Windows limits"),
        }
    }
}

impl From<windows::core::Error> for Win32Failure {
    fn from(error: windows::core::Error) -> Self {
        Self::Api(error)
    }
}

impl From<core::num::TryFromIntError> for Win32Failure {
    fn from(_error: core::num::TryFromIntError) -> Self {
        Self::OutOfRange
    }
}
