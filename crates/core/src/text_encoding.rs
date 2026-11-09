//! 文字コード。

use crate::text_units::TextUnits;

/// 原本の文字コード。閉じた集合（FR-002・ADR 0015）。表を引く復号は scan の仕事で、ここは名前と BOM だけを持つ。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextEncoding {
    /// BOM の無い UTF-8。
    Utf8,
    /// BOM（`EF BB BF`）の付いた UTF-8。
    Utf8WithBom,
    /// BOM（`FF FE`）の付いた UTF-16 のリトルエンディアン。
    Utf16LeWithBom,
    /// BOM（`FE FF`）の付いた UTF-16 のビッグエンディアン。
    Utf16BeWithBom,
    /// BOM の無い UTF-16 のリトルエンディアン。
    Utf16Le,
    /// BOM の無い UTF-16 のビッグエンディアン。
    Utf16Be,
    /// Windows の CP932（WHATWG の `Shift_JIS`）。JIS の `Shift_JIS` との区別は JP-01 で別に持つ。
    ShiftJis,
    /// EUC-JP。
    EucJp,
    /// Windows-1252（Latin-1 を含む）。
    Windows1252,
}

/// いちばん長い BOM のバイト数（UTF-8 の `EF BB BF`）。
pub const LONGEST_BOM: usize = 3;

impl TextEncoding {
    /// 切り出しの文字の単位（ADR 0019）。
    #[must_use]
    pub fn units(self) -> TextUnits {
        match self {
            Self::ShiftJis => TextUnits::ShiftJis,
            Self::Utf16LeWithBom | Self::Utf16Le => TextUnits::Utf16Le,
            Self::Utf16BeWithBom | Self::Utf16Be => TextUnits::Utf16Be,
            Self::Utf8 | Self::Utf8WithBom | Self::EucJp | Self::Windows1252 => {
                TextUnits::SingleByte
            }
        }
    }

    /// ASCII の字を ASCII の 1 バイトで書き、ASCII だけのバイト列が ASCII の字だけを表す文字コードか（UTF-16 でない）。
    /// 検索で、ASCII だけのレコードを復号せずに照らせる（ADR 0030 の決定 4）。
    #[must_use]
    pub const fn keeps_ascii(self) -> bool {
        match self {
            Self::Utf16LeWithBom | Self::Utf16BeWithBom | Self::Utf16Le | Self::Utf16Be => false,
            Self::Utf8 | Self::Utf8WithBom | Self::ShiftJis | Self::EucJp | Self::Windows1252 => {
                true
            }
        }
    }

    /// 値 `text` をこの文字コードで書いたときのバイト数（BOM を除く）。表を引かず、文字の範囲だけで数える。
    /// このファイルから復号した値なら原本のバイト数と一致する。ただし、読めないバイトの代わりの `U+FFFD` と、EUC-JP の
    /// 3 バイトの補助漢字（JIS X 0212）は 2 バイトとして数える（Issue #59）。
    #[must_use]
    pub fn encoded_length(self, text: &str) -> u64 {
        let length = |count: usize| u64::try_from(count).unwrap_or(u64::MAX);
        match self {
            Self::Utf8 | Self::Utf8WithBom => length(text.len()),
            Self::Utf16LeWithBom | Self::Utf16BeWithBom | Self::Utf16Le | Self::Utf16Be => {
                length(text.encode_utf16().count()).saturating_mul(2)
            }
            Self::Windows1252 => length(text.chars().count()),
            Self::ShiftJis | Self::EucJp => text.chars().fold(0_u64, |total, character| {
                total.saturating_add(self.japanese_width(character))
            }),
        }
    }

    /// 値 `text` の中で、この文字コードでは**必ず**表せない最初の字。表を引かずに言い切れるものだけ：`Shift_JIS`（CP932）と
    /// EUC-JP の、基本多言語面の外の字（絵文字・`𠮷` など。どちらの表も U+FFFF までしか持たない）。表せるか表を引かないと
    /// 分からない字（`♥` など）は `None` のまま（保存が書く前に確かめる・SAVE-04・GitHub Issue #395）。
    #[must_use]
    pub fn surely_unrepresentable(self, text: &str) -> Option<char> {
        match self {
            Self::ShiftJis | Self::EucJp => text.chars().find(|&character| character > '\u{FFFF}'),
            Self::Utf8
            | Self::Utf8WithBom
            | Self::Utf16LeWithBom
            | Self::Utf16BeWithBom
            | Self::Utf16Le
            | Self::Utf16Be
            | Self::Windows1252 => None,
        }
    }

    /// `Shift_JIS`（CP932）と EUC-JP の 1 字のバイト数。ASCII は 1、半角カナは `Shift_JIS` で 1・EUC-JP で 2
    /// （`8E` を前に置く）、ほかは 2。`Shift_JIS` の `0x80` は WHATWG の復号で `U+0080` の 1 バイト。
    fn japanese_width(self, character: char) -> u64 {
        let halfwidth_kana = ('\u{FF61}'..='\u{FF9F}').contains(&character);
        let single = character.is_ascii()
            || (self == Self::ShiftJis && (halfwidth_kana || character == '\u{80}'));
        if single { 1 } else { 2 }
    }

    /// 先頭の BOM のバイト列。BOM の無い文字コードでは空。保存では原本のまま写す（KEEP-04）。
    #[must_use]
    pub fn bom(self) -> &'static [u8] {
        match self {
            Self::Utf8WithBom => b"\xEF\xBB\xBF",
            Self::Utf16LeWithBom => b"\xFF\xFE",
            Self::Utf16BeWithBom => b"\xFE\xFF",
            Self::Utf8
            | Self::Utf16Le
            | Self::Utf16Be
            | Self::ShiftJis
            | Self::EucJp
            | Self::Windows1252 => b"",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boms_and_text_units() {
        assert_eq!(TextEncoding::Utf8WithBom.bom(), b"\xEF\xBB\xBF");
        assert_eq!(TextEncoding::Utf16LeWithBom.bom(), b"\xFF\xFE");
        assert_eq!(TextEncoding::Utf16BeWithBom.bom(), b"\xFE\xFF");
        assert!(TextEncoding::ShiftJis.bom().is_empty());
        assert_eq!(TextEncoding::ShiftJis.units(), TextUnits::ShiftJis);
        assert_eq!(TextEncoding::Utf16LeWithBom.units(), TextUnits::Utf16Le);
        assert_eq!(TextEncoding::Utf16Be.units(), TextUnits::Utf16Be);
        assert_eq!(TextEncoding::EucJp.units(), TextUnits::SingleByte);
    }

    #[test]
    fn no_bom_is_longer_than_the_longest_bom() {
        let every = [
            TextEncoding::Utf8,
            TextEncoding::Utf8WithBom,
            TextEncoding::Utf16LeWithBom,
            TextEncoding::Utf16BeWithBom,
            TextEncoding::Utf16Le,
            TextEncoding::Utf16Be,
            TextEncoding::ShiftJis,
            TextEncoding::EucJp,
            TextEncoding::Windows1252,
        ];
        assert!(
            every
                .iter()
                .all(|encoding| encoding.bom().len() <= LONGEST_BOM)
        );
        assert!(
            every
                .iter()
                .any(|encoding| encoding.bom().len() == LONGEST_BOM)
        );
    }

    #[test]
    fn encoded_lengths_follow_the_file_encoding() {
        let value = "0012 ｱ山田𠮷";
        // UTF-8: 5 ＋ 3 ＋ 3 × 2 ＋ 4。UTF-16: 8 単位 ＋ サロゲートの組 2 単位。
        assert_eq!(TextEncoding::Utf8.encoded_length(value), 18);
        assert_eq!(TextEncoding::Utf16LeWithBom.encoded_length(value), 20);
        assert_eq!(TextEncoding::Utf16Be.encoded_length(value), 20);
        // Shift_JIS: ASCII 5 ＋ 半角カナ 1 ＋ 漢字 2 × 2（𠮷 は Shift_JIS の値には現れないが 2 と数える）。
        assert_eq!(TextEncoding::ShiftJis.encoded_length("0012 ｱ山田"), 10);
        assert_eq!(TextEncoding::ShiftJis.encoded_length("\u{80}"), 1);
        // EUC-JP: 半角カナは 2。
        assert_eq!(TextEncoding::EucJp.encoded_length("0012 ｱ山田"), 11);
        assert_eq!(TextEncoding::Windows1252.encoded_length("Müller"), 6);
        assert_eq!(TextEncoding::ShiftJis.encoded_length(""), 0);
    }

    /// 絵文字や `𠮷` は `Shift_JIS` と EUC-JP では必ず表せない。表を引かないと分からない字と Unicode の文字コードは言わない
    /// （GitHub Issue #395）。
    #[test]
    fn characters_beyond_the_basic_plane_are_surely_unrepresentable_in_japanese_encodings() {
        assert_eq!(
            TextEncoding::ShiftJis.surely_unrepresentable("𠮷田"),
            Some('𠮷')
        );
        assert_eq!(
            TextEncoding::EucJp.surely_unrepresentable("ok 😀"),
            Some('😀')
        );
        assert_eq!(
            TextEncoding::ShiftJis.surely_unrepresentable("髙橋 ①"),
            None
        );
        assert_eq!(TextEncoding::Utf8.surely_unrepresentable("😀"), None);
        assert_eq!(TextEncoding::Windows1252.surely_unrepresentable("😀"), None);
    }
}
