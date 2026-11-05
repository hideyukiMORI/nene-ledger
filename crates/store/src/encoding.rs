//! 書き換えた値を、原本の文字コードで符号化する（ADR 0025 の決定 1）。

use encoding_rs::{EUC_JP, SHIFT_JIS, WINDOWS_1252};
use nene_ledger_core::text_encoding::TextEncoding;

/// `text` を `encoding` で符号化したバイト列（BOM を除く）。表せない字があれば `None`（SAVE-04・JP-03）。
///
/// UTF-16 は `encoding_rs` が符号化しないので、ここで書く。
#[must_use]
pub fn encoded(text: &str, encoding: TextEncoding) -> Option<Vec<u8>> {
    let table = match encoding {
        TextEncoding::Utf8 | TextEncoding::Utf8WithBom => return Some(text.as_bytes().to_vec()),
        TextEncoding::Utf16LeWithBom | TextEncoding::Utf16Le => {
            return Some(text.encode_utf16().flat_map(u16::to_le_bytes).collect());
        }
        TextEncoding::Utf16BeWithBom | TextEncoding::Utf16Be => {
            return Some(text.encode_utf16().flat_map(u16::to_be_bytes).collect());
        }
        TextEncoding::ShiftJis => SHIFT_JIS,
        TextEncoding::EucJp => EUC_JP,
        TextEncoding::Windows1252 => WINDOWS_1252,
    };
    let (bytes, _, unmappable) = table.encode(text);
    (!unmappable).then(|| bytes.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_encoding_writes_what_it_can_and_refuses_the_rest() {
        assert_eq!(encoded("aé", TextEncoding::Utf8).unwrap(), "aé".as_bytes());
        assert_eq!(encoded("a", TextEncoding::Utf16LeWithBom).unwrap(), b"a\0");
        assert_eq!(encoded("a", TextEncoding::Utf16Be).unwrap(), b"\0a");
        assert_eq!(
            encoded("日本", TextEncoding::ShiftJis).unwrap(),
            b"\x93\xfa\x96\x7b"
        );
        assert_eq!(encoded("ｱ", TextEncoding::ShiftJis).unwrap(), b"\xb1");
        assert_eq!(encoded("日", TextEncoding::EucJp).unwrap(), b"\xc6\xfc");
        assert_eq!(
            encoded("é€", TextEncoding::Windows1252).unwrap(),
            b"\xe9\x80"
        );
        assert_eq!(encoded("😀", TextEncoding::ShiftJis), None);
        assert_eq!(encoded("日", TextEncoding::Windows1252), None);
    }
}
