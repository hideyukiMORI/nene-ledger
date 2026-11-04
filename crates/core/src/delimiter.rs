//! 区切り文字。

use crate::dialect_failure::DialectFailure;

/// 区切り文字（1 バイト・ASCII・改行でない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Delimiter(u8);

impl Delimiter {
    /// カンマ。
    pub const COMMA: Self = Self(b',');
    /// タブ。
    pub const TAB: Self = Self(b'\t');

    /// 区切りを作る。
    ///
    /// # Errors
    ///
    /// 改行なら [`DialectFailure::LineBreakDelimiter`]、ASCII でなければ [`DialectFailure::NonAsciiDelimiter`]。
    pub fn new(byte: u8) -> Result<Self, DialectFailure> {
        if byte == b'\r' || byte == b'\n' {
            return Err(DialectFailure::LineBreakDelimiter);
        }
        if !byte.is_ascii() {
            return Err(DialectFailure::NonAsciiDelimiter);
        }
        Ok(Self(byte))
    }

    /// 区切りのバイト。
    #[must_use]
    pub fn byte(self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_ascii_and_rejects_breaks_and_non_ascii() {
        assert_eq!(Delimiter::new(b';').map(Delimiter::byte), Ok(b';'));
        assert_eq!(
            Delimiter::new(b'\n'),
            Err(DialectFailure::LineBreakDelimiter)
        );
        assert_eq!(
            Delimiter::new(b'\r'),
            Err(DialectFailure::LineBreakDelimiter)
        );
        assert_eq!(Delimiter::new(0xE3), Err(DialectFailure::NonAsciiDelimiter));
    }
}
