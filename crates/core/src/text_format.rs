//! 文字コードと方言の組。

use crate::dialect::Dialect;
use crate::text_encoding::TextEncoding;

/// 原本の読み方。文字コードと方言の組で、方言は文字コードの文字の単位を持つ（RS-001・ADR 0019）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextFormat {
    encoding: TextEncoding,
    dialect: Dialect,
}

impl TextFormat {
    /// 組を作る。方言には、文字コードから決まる文字の単位を入れる（ADR 0019）。どの文字コードと方言の組も切れる。
    #[must_use]
    pub fn new(encoding: TextEncoding, dialect: Dialect) -> Self {
        Self {
            encoding,
            dialect: dialect.with_units(encoding.units()),
        }
    }

    /// 文字コード。
    #[must_use]
    pub fn encoding(&self) -> TextEncoding {
        self.encoding
    }

    /// 方言。
    #[must_use]
    pub fn dialect(&self) -> Dialect {
        self.dialect
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delimiter::Delimiter;
    use crate::quoting::Quoting;
    use crate::text_units::TextUnits;

    fn dialect(delimiter: u8, quoting: Quoting) -> Dialect {
        Dialect::new(Delimiter::new(delimiter).unwrap(), quoting, b'"').unwrap()
    }

    #[test]
    fn utf16_dialects_get_utf16_units() {
        let format = TextFormat::new(TextEncoding::Utf16LeWithBom, Dialect::rfc4180());
        assert_eq!(format.dialect().units(), TextUnits::Utf16Le);
        let big = TextFormat::new(TextEncoding::Utf16Be, Dialect::rfc4180());
        assert_eq!(big.dialect().units(), TextUnits::Utf16Be);
    }

    #[test]
    fn shift_jis_dialects_get_shift_jis_units_even_with_trail_range_bytes() {
        // 2 バイト目の範囲（0x40〜0x7E）の区切り・エスケープ・引用符も、文字の頭でだけ見分けるので受ける（ADR 0019）。
        let pipe = dialect(b'|', Quoting::Doubled);
        let backslash = dialect(b'\t', Quoting::Backslash);
        let quote_at = Dialect::new(Delimiter::COMMA, Quoting::Doubled, b'@').unwrap();
        for wanted in [pipe, backslash, quote_at] {
            let format = TextFormat::new(TextEncoding::ShiftJis, wanted);
            assert_eq!(format.dialect(), wanted.with_units(TextUnits::ShiftJis));
            assert_eq!(format.dialect().units(), TextUnits::ShiftJis);
        }
    }

    #[test]
    fn the_usual_pairs_are_accepted() {
        let plain_semicolon = dialect(b';', Quoting::Plain);
        for (encoding, dialect) in [
            (TextEncoding::ShiftJis, Dialect::rfc4180()),
            (TextEncoding::ShiftJis, dialect(b'\t', Quoting::Doubled)),
            (TextEncoding::ShiftJis, plain_semicolon),
            (TextEncoding::EucJp, dialect(b'|', Quoting::Backslash)),
            (
                TextEncoding::Utf8WithBom,
                dialect(b'|', Quoting::QuotedBackslash),
            ),
        ] {
            let format = TextFormat::new(encoding, dialect);
            let units = encoding.units();
            assert_eq!(
                (format.encoding(), format.dialect()),
                (encoding, dialect.with_units(units))
            );
        }
    }
}
