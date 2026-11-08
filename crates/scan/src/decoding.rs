//! セルの値を表示のために復号する（`encoding_rs`・ADR 0015）。

use encoding_rs::{EUC_JP, Encoding, SHIFT_JIS, UTF_8, UTF_16BE, UTF_16LE, WINDOWS_1252};
use nene_ledger_core::text_encoding::TextEncoding;

use crate::decoded_text::DecodedText;

/// 文字コードの表。
fn table(encoding: TextEncoding) -> &'static Encoding {
    match encoding {
        TextEncoding::Utf8 | TextEncoding::Utf8WithBom => UTF_8,
        TextEncoding::Utf16LeWithBom | TextEncoding::Utf16Le => UTF_16LE,
        TextEncoding::Utf16BeWithBom | TextEncoding::Utf16Be => UTF_16BE,
        TextEncoding::ShiftJis => SHIFT_JIS,
        TextEncoding::EucJp => EUC_JP,
        TextEncoding::Windows1252 => WINDOWS_1252,
    }
}

/// `bytes`（セルの値・引用符とエスケープを外したもの）を表示のために復号する。
///
/// BOM は見ない（切り出しが BOM の後ろから始まる）。原本のバイト列は変えず、保存は原本を写す（ADR 0015 の決定 3）。
#[must_use]
pub fn decode(encoding: TextEncoding, bytes: &[u8]) -> DecodedText {
    let (text, malformed) = table(encoding).decode_without_bom_handling(bytes);
    DecodedText::new(text.into_owned(), malformed)
}

/// [`decode`] と同じ復号で、字を `out` の後ろに書き足す（読めたかは返さない）。窓の行を 1 本の文字列で作るとき、セルごとの
/// `String` を作らずに済む（GitHub Issue #22）。
pub fn decode_into(encoding: TextEncoding, bytes: &[u8], out: &mut String) {
    let (text, _malformed) = table(encoding).decode_without_bom_handling(bytes);
    out.push_str(&text);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 書き足す復号は、`decode` と同じ字を出す（読めないバイト・BOM に見える並び・どの文字コードでも）。
    #[test]
    fn decoding_into_a_string_gives_the_same_text() {
        let inputs: [&[u8]; 6] = [
            b"plain",
            "東京".as_bytes(),
            b"\x93\x8c\x8b\x9e",
            b"\xef\xbb\xbfbom",
            b"\xff\xfe\x00",
            b"",
        ];
        let encodings = [
            TextEncoding::Utf8,
            TextEncoding::Utf8WithBom,
            TextEncoding::Utf16Le,
            TextEncoding::Utf16BeWithBom,
            TextEncoding::ShiftJis,
            TextEncoding::EucJp,
            TextEncoding::Windows1252,
        ];
        for encoding in encodings {
            let mut out = String::from("前,");
            for bytes in inputs {
                let before = out.len();
                decode_into(encoding, bytes, &mut out);
                assert_eq!(
                    out.get(before..),
                    Some(decode(encoding, bytes).text()),
                    "{encoding:?} {bytes:?}"
                );
            }
        }
    }
}
