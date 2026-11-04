//! 文字のファイルでないもの（exe・画像・ZIP など）の判別（Issue #57）。

use crate::text_units::TextUnits;

/// 標本 `content`（BOM の後ろ）を文字の単位 `units` で見て、文字のファイルでないようなら `true`。
///
/// NUL があれば文字のファイルでないとみなす。1 バイトの単位（UTF-8・Shift_JIS など）は 0 のバイト、UTF-16 は 0 の
/// 2 バイトの単位を見る（UTF-16 の英数字は 0 のバイトを含むので、バイトでは見ない）。CSV に NUL はまず入らず、
/// exe・画像・ZIP（xlsx を含む）にはほぼ必ず入る。開くかどうかは利用者が決める（原本は読むだけ・ADR 0005）。
#[must_use]
pub fn looks_binary(content: &[u8], units: TextUnits) -> bool {
    match units {
        TextUnits::SingleByte | TextUnits::ShiftJis => content.contains(&0),
        TextUnits::Utf16Le | TextUnits::Utf16Be => content.as_chunks::<2>().0.contains(&[0, 0]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    fn utf16le(text: &str) -> Vec<u8> {
        text.encode_utf16().flat_map(u16::to_le_bytes).collect()
    }

    #[test]
    fn text_files_are_text_and_files_with_nul_are_not() {
        assert!(!looks_binary(b"id,name\n1,a\n", TextUnits::SingleByte));
        // Shift_JIS の「表」（95 5C）。
        assert!(!looks_binary(b"\x95\x5c,1\n", TextUnits::ShiftJis));
        // UTF-16 の英数字は 0 のバイトを含むが文字。
        assert!(!looks_binary(
            &utf16le("id,name\r\n1,a\r\n"),
            TextUnits::Utf16Le
        ));
        assert!(looks_binary(b"MZ\x90\x00\x03\x00", TextUnits::SingleByte));
        assert!(looks_binary(
            b"PK\x03\x04\x14\x00\x06\x00",
            TextUnits::SingleByte
        ));
        assert!(looks_binary(&[0x4D, 0x5A, 0x00, 0x00], TextUnits::Utf16Le));
        assert!(!looks_binary(b"", TextUnits::SingleByte));
    }
}
