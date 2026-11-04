//! 文字の単位（ADR 0019）。

/// 切り出しが文字の頭を見分けるための、文字の単位。閉じた集合（RS-002）。文字コードから決まり、利用者は直接選ばない。
///
/// 区切り・引用符・改行・エスケープは ASCII の 1 バイトで、文字の頭でだけ見分ける（ADR 0019）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextUnits {
    /// どのバイトも文字か、ASCII と重ならない文字の一部（UTF-8・EUC-JP・Windows-1252）。
    SingleByte,
    /// `Shift_JIS`（CP932）。1 バイト目（0x81〜0x9F・0xE0〜0xFC）の次が 2 バイト目として正しければ 2 バイトで 1 文字。
    ShiftJis,
    /// UTF-16 のリトルエンディアン。2 バイトの単位で進み、単位の値を ASCII と比べる（サロゲートの半分は ASCII と重ならない）。
    Utf16Le,
    /// UTF-16 のビッグエンディアン。
    Utf16Be,
}

impl TextUnits {
    /// `position` から始まる文字のバイト数（入力の終わりなら 1）。`Shift_JIS` で 1 バイト目の次が 2 バイト目として
    /// 正しくなければ 1（壊れたファイルで改行や区切りを飲み込まない）。
    #[must_use]
    pub fn char_width(self, input: &[u8], position: usize) -> usize {
        match self {
            Self::SingleByte => 1,
            Self::Utf16Le | Self::Utf16Be => self.unit_len(),
            Self::ShiftJis => {
                let lead = input.get(position).copied().is_some_and(is_shift_jis_lead);
                let trail = input
                    .get(position.saturating_add(1))
                    .copied()
                    .is_some_and(is_shift_jis_trail);
                if lead && trail { 2 } else { 1 }
            }
        }
    }

    /// `byte` が、次のバイトと組んで 1 文字になりうる `Shift_JIS` の 1 バイト目か（UTF-16 の単位は別に組む）。
    #[must_use]
    pub fn is_lead(self, byte: u8) -> bool {
        match self {
            Self::SingleByte | Self::Utf16Le | Self::Utf16Be => false,
            Self::ShiftJis => is_shift_jis_lead(byte),
        }
    }

    /// `byte` が、1 バイト目の次に来て 1 文字をなす `Shift_JIS` の 2 バイト目として正しいか。
    #[must_use]
    pub fn is_trail(self, byte: u8) -> bool {
        match self {
            Self::SingleByte | Self::Utf16Le | Self::Utf16Be => false,
            Self::ShiftJis => is_shift_jis_trail(byte),
        }
    }

    /// ASCII の 1 文字（区切り・引用符・改行・エスケープ）のバイト数。1 バイトの文字コードは 1、UTF-16 は 2。
    #[must_use]
    pub fn unit_len(self) -> usize {
        match self {
            Self::SingleByte | Self::ShiftJis => 1,
            Self::Utf16Le | Self::Utf16Be => 2,
        }
    }

    /// `position` の単位の値（1 バイトの文字コードはそのバイト、UTF-16 は 2 バイトの値）。入力の終わり（UTF-16 で
    /// 奇数の長さの最後の 1 バイトを含む）なら `None`。
    #[must_use]
    pub fn unit_at(self, input: &[u8], position: usize) -> Option<u16> {
        let first = *input.get(position)?;
        match self {
            Self::SingleByte | Self::ShiftJis => Some(u16::from(first)),
            Self::Utf16Le => {
                let second = *input.get(position.checked_add(1)?)?;
                Some(u16::from_le_bytes([first, second]))
            }
            Self::Utf16Be => {
                let second = *input.get(position.checked_add(1)?)?;
                Some(u16::from_be_bytes([first, second]))
            }
        }
    }

    /// `position` の単位が ASCII の `byte` か。
    #[must_use]
    pub fn is_ascii_at(self, input: &[u8], position: usize, byte: u8) -> bool {
        self.unit_at(input, position) == Some(u16::from(byte))
    }

    /// 1 文字 `character` が ASCII の `byte` か。
    #[must_use]
    pub fn is_ascii_char(self, character: &[u8], byte: u8) -> bool {
        character.len() == self.unit_len() && self.is_ascii_at(character, 0, byte)
    }

    /// 1 文字 `character` が ASCII なら、そのバイト。
    #[must_use]
    pub fn ascii_of(self, character: &[u8]) -> Option<u8> {
        if character.len() != self.unit_len() {
            return None;
        }
        let unit = self.unit_at(character, 0)?;
        u8::try_from(unit).ok().filter(u8::is_ascii)
    }

    /// 値 `value` の文字を、ASCII はそのバイト、それ以外は ASCII でない印（0xFF）1 つにした列。判別（数に見えるか・
    /// 空か・切り方の指紋）を文字の単位に依らず同じ規則で行うため（ADR 0019）。1 バイトの文字コードでは `value` のまま。
    #[must_use]
    pub fn narrow(self, value: &[u8]) -> alloc::vec::Vec<u8> {
        match self {
            Self::SingleByte | Self::ShiftJis => value.to_vec(),
            Self::Utf16Le | Self::Utf16Be => characters(value, self)
                .map(|character| self.ascii_of(character).unwrap_or(NOT_ASCII))
                .collect(),
        }
    }

    /// ASCII の `byte` を、この単位で `out` に書く（エスケープを外した `\t` などを値に足すとき）。
    pub fn push_ascii(self, out: &mut alloc::vec::Vec<u8>, byte: u8) {
        match self {
            Self::SingleByte | Self::ShiftJis => out.push(byte),
            Self::Utf16Le => out.extend_from_slice(&[byte, 0]),
            Self::Utf16Be => out.extend_from_slice(&[0, byte]),
        }
    }
}

/// `input` を文字ごとに切った列（どの文字も 1 か 2 バイト）。
pub fn characters(input: &[u8], units: TextUnits) -> impl Iterator<Item = &[u8]> {
    let mut position = 0_usize;
    core::iter::from_fn(move || {
        if position >= input.len() {
            return None;
        }
        let width = units.char_width(input, position);
        let end = position.saturating_add(width).min(input.len());
        let character = input.get(position..end);
        position = end;
        character
    })
}

/// `narrow` が ASCII でない文字の代わりに置く印。
const NOT_ASCII: u8 = 0xFF;

fn is_shift_jis_lead(byte: u8) -> bool {
    matches!(byte, 0x81..=0x9F | 0xE0..=0xFC)
}

fn is_shift_jis_trail(byte: u8) -> bool {
    matches!(byte, 0x40..=0x7E | 0x80..=0xFC)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;

    #[test]
    fn single_bytes_are_one_wide() {
        assert_eq!(TextUnits::SingleByte.char_width(b"\x95\x5C", 0), 1);
        assert!(!TextUnits::SingleByte.is_lead(0x95));
        assert!(!TextUnits::SingleByte.is_trail(0x5C));
    }

    #[test]
    fn shift_jis_pairs_a_lead_with_a_valid_trail_only() {
        let units = TextUnits::ShiftJis;
        // 「表」は 95 5C。2 バイト目がバックスラッシュでも 1 文字。
        assert_eq!(units.char_width(b"\x95\x5C", 0), 2);
        // 1 バイト目の次が改行・入力の終わり・2 バイト目として正しくないバイトなら 1。
        assert_eq!(units.char_width(b"\x95\n", 0), 1);
        assert_eq!(units.char_width(b"\x95", 0), 1);
        assert_eq!(units.char_width(b"\x95\x3F", 0), 1);
        assert_eq!(units.char_width(b"a\x5C", 0), 1);
        assert!(units.is_lead(0xE0) && !units.is_lead(0xA0) && !units.is_lead(0x7F));
        assert!(units.is_trail(0x40) && units.is_trail(0xFC) && !units.is_trail(0x7F));
    }

    #[test]
    fn utf16_units_are_read_and_written_in_both_byte_orders() {
        let le = TextUnits::Utf16Le;
        let be = TextUnits::Utf16Be;
        assert_eq!(le.unit_at(b"\x2C\x00", 0), Some(0x2C));
        assert_eq!(be.unit_at(b"\x00\x2C", 0), Some(0x2C));
        assert_eq!(le.unit_at(b"\x2C", 0), None);
        assert!(le.is_ascii_at(b"a\x00,\x00", 2, b','));
        assert!(!le.is_ascii_at(b"\x2C\x30", 0, b','));
        assert_eq!(le.char_width(b"\x00\xD8\x00\xDC", 0), 2);
        assert_eq!(le.unit_len(), 2);
        assert!(be.is_ascii_char(b"\x00\"", b'"'));
        assert!(!be.is_ascii_char(b"\"", b'"'));
    }

    #[test]
    fn ascii_is_written_and_narrowed_in_units() {
        let le = TextUnits::Utf16Le;
        let be = TextUnits::Utf16Be;
        let mut out = Vec::new();
        le.push_ascii(&mut out, b'\t');
        be.push_ascii(&mut out, b'\n');
        TextUnits::SingleByte.push_ascii(&mut out, b'x');
        assert_eq!(out, b"\t\x00\x00\nx");
        assert!(!le.is_lead(0x95) && !le.is_trail(0x5C));
        assert_eq!(le.ascii_of(b"a\x00"), Some(b'a'));
        assert_eq!(le.ascii_of(b"\x42\x30"), None);
        assert_eq!(
            le.narrow(
                "1あ,"
                    .encode_utf16()
                    .flat_map(u16::to_le_bytes)
                    .collect::<Vec<u8>>()
                    .as_slice()
            ),
            b"1\xFF,"
        );
        assert_eq!(TextUnits::ShiftJis.narrow(b"\x95\x5C"), b"\x95\x5C");
    }

    #[test]
    fn characters_split_the_input() {
        let parts: Vec<&[u8]> = characters(b"a\x95\x5C|\x95", TextUnits::ShiftJis).collect();
        assert_eq!(
            parts,
            vec![&b"a"[..], &b"\x95\x5C"[..], &b"|"[..], &b"\x95"[..]]
        );
        assert_eq!(characters(b"", TextUnits::SingleByte).count(), 0);
    }
}
