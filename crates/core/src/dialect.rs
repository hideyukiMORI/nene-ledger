//! 方言（区切りと引用の流儀）。

use crate::delimiter::Delimiter;
use crate::dialect_failure::DialectFailure;
use crate::quoting::Quoting;
use crate::text_units::TextUnits;
use crate::unclosed_quote::UnclosedQuote;

/// 切り出しの方言。区切り・引用の流儀・引用符・閉じない引用符の読み方・文字の単位。文字コードそのものと見出しの有無は別に持つ。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dialect {
    delimiter: Delimiter,
    quoting: Quoting,
    quote: u8,
    unclosed: UnclosedQuote,
    units: TextUnits,
}

impl Dialect {
    /// RFC 4180 の CSV（カンマ・`"`・`""`）。
    #[must_use]
    pub fn rfc4180() -> Self {
        Self {
            delimiter: Delimiter::COMMA,
            quoting: Quoting::Doubled,
            quote: b'"',
            unclosed: UnclosedQuote::ToEnd,
            units: TextUnits::SingleByte,
        }
    }

    /// 方言を作る。`quote` は引用を使わない流儀（`Backslash` / `Plain`）では使われないが、検証はする。
    ///
    /// # Errors
    ///
    /// 引用符が改行・非 ASCII・区切りと同じ、または引用符の中をバックスラッシュで逃がす流儀で
    /// 引用符がバックスラッシュのとき。
    pub fn new(delimiter: Delimiter, quoting: Quoting, quote: u8) -> Result<Self, DialectFailure> {
        if quote == b'\r' || quote == b'\n' {
            return Err(DialectFailure::LineBreakQuote);
        }
        if !quote.is_ascii() {
            return Err(DialectFailure::NonAsciiQuote);
        }
        if quote == delimiter.byte() {
            return Err(DialectFailure::QuoteEqualsDelimiter);
        }
        if quoting == Quoting::QuotedBackslash && quote == b'\\' {
            return Err(DialectFailure::BackslashQuote);
        }
        Ok(Self {
            delimiter,
            quoting,
            quote,
            unclosed: UnclosedQuote::ToEnd,
            units: TextUnits::SingleByte,
        })
    }

    /// 閉じない引用符の読み方を替えた方言（ADR 0018）。作った直後は `UnclosedQuote::ToEnd`。
    #[must_use]
    pub fn with_unclosed_quote(self, unclosed: UnclosedQuote) -> Self {
        Self { unclosed, ..self }
    }

    /// 文字の単位を替えた方言（ADR 0019）。作った直後は `TextUnits::SingleByte`。`TextFormat::new` が文字コードから決める。
    #[must_use]
    pub fn with_units(self, units: TextUnits) -> Self {
        Self { units, ..self }
    }

    /// 文字の単位。
    #[must_use]
    pub fn units(&self) -> TextUnits {
        self.units
    }

    /// 区切り。
    #[must_use]
    pub fn delimiter(&self) -> Delimiter {
        self.delimiter
    }

    /// 引用の流儀。
    #[must_use]
    pub fn quoting(&self) -> Quoting {
        self.quoting
    }

    /// 引用符のバイト。
    #[must_use]
    pub fn quote(&self) -> u8 {
        self.quote
    }

    /// 閉じない引用符の読み方。
    #[must_use]
    pub fn unclosed_quote(&self) -> UnclosedQuote {
        self.unclosed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_bad_quotes() {
        let comma = Delimiter::COMMA;
        assert_eq!(
            Dialect::new(comma, Quoting::Doubled, b'\n'),
            Err(DialectFailure::LineBreakQuote)
        );
        assert_eq!(
            Dialect::new(comma, Quoting::Doubled, 0x80),
            Err(DialectFailure::NonAsciiQuote)
        );
        assert_eq!(
            Dialect::new(comma, Quoting::Doubled, b','),
            Err(DialectFailure::QuoteEqualsDelimiter)
        );
        assert_eq!(
            Dialect::new(comma, Quoting::QuotedBackslash, b'\\'),
            Err(DialectFailure::BackslashQuote)
        );
    }

    #[test]
    fn accepts_single_quotes_and_tabs() {
        let dialect = Dialect::new(Delimiter::TAB, Quoting::Doubled, b'\'');
        assert_eq!(
            dialect.map(|d| (d.delimiter().byte(), d.quote())),
            Ok((b'\t', b'\''))
        );
        assert_eq!(Dialect::rfc4180().quoting(), Quoting::Doubled);
    }
}
