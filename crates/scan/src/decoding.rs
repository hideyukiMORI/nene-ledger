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
