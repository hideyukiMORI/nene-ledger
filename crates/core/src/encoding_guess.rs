//! 文字コードの判別の結果。

use alloc::vec::Vec;

use crate::detecting::SAMPLE_LIMIT;
use crate::encoding_detecting::{
    bom_encoding, euc_jp_has_hiragana, euc_jp_valid, shift_jis_lacks_common_characters,
    shift_jis_valid, utf8_valid, utf16_without_bom,
};
use crate::text_encoding::TextEncoding;

/// BOM の無い候補を示す順（先頭の候補の後ろに、これから先頭と同じものを除いて並べる）。
const FALLBACK_ORDER: [TextEncoding; 6] = [
    TextEncoding::Utf8,
    TextEncoding::ShiftJis,
    TextEncoding::EucJp,
    TextEncoding::Windows1252,
    TextEncoding::Utf16Le,
    TextEncoding::Utf16Be,
];

/// 文字コードの判別の結果（OPEN-09・ADR 0015 の決定 2）。値は変えない。
///
/// 順は BOM → BOM の無い UTF-16 らしさ → UTF-8 → CP932 / EUC-JP → Windows-1252。
/// 🔴 構造が正しいかだけを見る推し量りで、確信が無ければ黙って決めず候補を示して選ばせる（FR-023）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodingGuess {
    candidates: Vec<TextEncoding>,
    confident: bool,
}

impl EncodingGuess {
    /// 入力の先頭（最大 `SAMPLE_LIMIT` バイト。方言の判別と同じ量）から文字コードを推し量る。
    ///
    /// 入力が `SAMPLE_LIMIT` を超えれば標本の後ろに続きがあるとみなし、末尾で切れた字を誤りとしない。原本から読むときは
    /// [`READ_LIMIT`](crate::detecting::READ_LIMIT) だけ渡す（GitHub Issue #10）。
    #[must_use]
    pub fn of(input: &[u8]) -> Self {
        if let Some(encoding) = bom_encoding(input) {
            return Self::led_by(encoding, true);
        }
        let complete = input.len() <= SAMPLE_LIMIT;
        let sample = input.get(..SAMPLE_LIMIT).unwrap_or(input);
        if let Some(utf16) = utf16_without_bom(sample) {
            return Self::led_by(utf16, false);
        }
        if utf8_valid(sample, complete) {
            // ASCII だけでも確信ありとする。ASCII の範囲は UTF-8・CP932・EUC-JP・Windows-1252 で読み方が同じ。
            return Self::led_by(TextEncoding::Utf8, true);
        }
        match (
            shift_jis_valid(sample, complete),
            euc_jp_valid(sample, complete),
        ) {
            (true, false) if shift_jis_lacks_common_characters(sample) => {
                Self::led_by(TextEncoding::Windows1252, false)
            }
            (true, false) => Self::led_by(TextEncoding::ShiftJis, true),
            (false, true) => Self::led_by(TextEncoding::EucJp, true),
            (true, true) if euc_jp_has_hiragana(sample) => Self::led_by(TextEncoding::EucJp, false),
            (true, true) => Self::led_by(TextEncoding::ShiftJis, false),
            (false, false) => Self::led_by(TextEncoding::Windows1252, false),
        }
    }

    /// `first` を先頭に、残りを `FALLBACK_ORDER` の順に並べる。
    fn led_by(first: TextEncoding, confident: bool) -> Self {
        let mut candidates = Vec::with_capacity(FALLBACK_ORDER.len().saturating_add(1));
        candidates.push(first);
        candidates.extend(FALLBACK_ORDER.into_iter().filter(|&other| other != first));
        Self {
            candidates,
            confident,
        }
    }

    /// いちばんもっともらしい文字コード。
    #[must_use]
    pub fn best(&self) -> TextEncoding {
        self.candidates
            .first()
            .copied()
            .unwrap_or(TextEncoding::Utf8)
    }

    /// 候補（もっともらしい順）。利用者が選び直すために示す（OPEN-10）。
    #[must_use]
    pub fn candidates(&self) -> &[TextEncoding] {
        &self.candidates
    }

    /// 確信があるか。無ければ黙って決めず、候補を示して選ばせる（FR-023）。
    #[must_use]
    pub fn confident(&self) -> bool {
        self.confident
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detecting::READ_LIMIT;
    use alloc::vec;

    fn guess(input: &[u8]) -> (TextEncoding, bool) {
        let guess = EncodingGuess::of(input);
        (guess.best(), guess.confident())
    }

    /// 「日本語」の CP932（JIS 0x467C 0x4B5C 0x386C）。`本` の 2 バイト目は `{`（0x7B）。
    const NIHONGO_SHIFT_JIS: &[u8] = b"\x93\xFA\x96\x7B\x8C\xEA";
    /// 「日本語」の EUC-JP。
    const NIHONGO_EUC_JP: &[u8] = b"\xC6\xFC\xCB\xDC\xB8\xEC";

    fn line(cells: &[&[u8]]) -> Vec<u8> {
        let mut out = cells.join(&b","[..]);
        out.push(b'\n');
        out
    }

    #[test]
    fn a_bom_decides() {
        assert_eq!(
            guess(b"\xEF\xBB\xBFa,b\n"),
            (TextEncoding::Utf8WithBom, true)
        );
        assert_eq!(
            guess(b"\xFF\xFEa\x00"),
            (TextEncoding::Utf16LeWithBom, true)
        );
        assert_eq!(
            guess(b"\xFE\xFF\x00a"),
            (TextEncoding::Utf16BeWithBom, true)
        );
    }

    #[test]
    fn ascii_and_japanese_utf8_are_utf8() {
        assert_eq!(guess(b""), (TextEncoding::Utf8, true));
        assert_eq!(guess(b"id,name\n1,x\n"), (TextEncoding::Utf8, true));
        assert_eq!(
            guess("日本語,東京\n".as_bytes()),
            (TextEncoding::Utf8, true)
        );
    }

    #[test]
    fn shift_jis_and_euc_jp_with_kanji_are_told_apart() {
        let sjis = line(&[b"1", NIHONGO_SHIFT_JIS]);
        assert_eq!(guess(&sjis), (TextEncoding::ShiftJis, true));
        let euc = line(&[b"1", NIHONGO_EUC_JP]);
        assert_eq!(guess(&euc), (TextEncoding::EucJp, true));
    }

    #[test]
    fn readable_both_ways_is_not_confident() {
        // 半角カナ「ｱｲｳｴ」の CP932 は、EUC-JP の 2 文字としても読める。ひらがなが無いので CP932 を先に。
        assert_eq!(
            guess(b"\xB1\xB2\xB3\xB4\n"),
            (TextEncoding::ShiftJis, false)
        );
        // 「あいう」の EUC-JP は、CP932 の半角カナ 6 文字としても読める。ひらがながあるので EUC-JP を先に。
        assert_eq!(
            guess(b"\xA4\xA2\xA4\xA4\xA4\xA6\n"),
            (TextEncoding::EucJp, false)
        );
    }

    #[test]
    fn neither_falls_back_to_windows_1252_without_confidence() {
        let cafe = b"caf\xE9\n";
        assert_eq!(guess(cafe), (TextEncoding::Windows1252, false));
        let candidates = EncodingGuess::of(cafe).candidates().to_vec();
        assert_eq!(
            candidates,
            vec![
                TextEncoding::Windows1252,
                TextEncoding::Utf8,
                TextEncoding::ShiftJis,
                TextEncoding::EucJp,
                TextEncoding::Utf16Le,
                TextEncoding::Utf16Be,
            ]
        );
    }

    #[test]
    fn utf16_without_a_bom_is_suggested_without_confidence() {
        assert_eq!(
            guess(b"a\x00,\x00b\x00\n\x00"),
            (TextEncoding::Utf16Le, false)
        );
        assert_eq!(
            guess(b"\x00a\x00,\x00b\x00\n"),
            (TextEncoding::Utf16Be, false)
        );
    }

    /// NUL が 1 つだけの短い標本は UTF-16 とみなさない（GitHub Issue #25）。2 字の UTF-16 は UTF-16 のまま。
    #[test]
    fn a_single_nul_is_not_enough_to_lean_towards_utf16() {
        assert_eq!(guess(b"a\tb\n\x00\x01").0, TextEncoding::Utf8);
        assert_eq!(guess(b"a\x00\n\x00").0, TextEncoding::Utf16Le);
    }

    #[test]
    fn a_character_cut_at_the_end_of_the_sample_is_not_an_error() {
        let mut input = b"ab".to_vec();
        while input.len() <= SAMPLE_LIMIT {
            input.extend_from_slice("あ".as_bytes());
        }
        assert_ne!((SAMPLE_LIMIT - 2) % 3, 0);
        assert_eq!(guess(&input), (TextEncoding::Utf8, true));
        // 標本がファイルの全部なら、欠けた文字は誤り。
        assert_ne!(guess(&"あ".as_bytes()[..2]).0, TextEncoding::Utf8);
    }

    /// 原本の先頭から [`READ_LIMIT`] だけ読んだ標本（app の読み方）で、`SAMPLE_LIMIT` の境目に字がまたがっても
    /// 文字コードを取り違えない（GitHub Issue #10）。境目をずらして、字の切れ方をすべて通す。
    #[test]
    fn a_sample_read_up_to_the_read_limit_keeps_a_character_split_at_the_sample_limit() {
        for (character, expected) in [
            ("山".as_bytes(), TextEncoding::Utf8),
            (&NIHONGO_SHIFT_JIS[..2], TextEncoding::ShiftJis),
            (&NIHONGO_EUC_JP[..2], TextEncoding::EucJp),
        ] {
            for shift in 0..character.len() {
                let input = repeated_past_the_read_limit(shift, character);
                let read = &input[..READ_LIMIT];
                assert_eq!(guess(read).0, expected, "shift {shift}");
            }
        }
    }

    /// `shift` バイトの `a` の後ろに、`READ_LIMIT` を超えるまで `character,` を並べる。
    fn repeated_past_the_read_limit(shift: usize, character: &[u8]) -> Vec<u8> {
        let mut input = vec![b'a'; shift];
        while input.len() <= READ_LIMIT {
            input.extend_from_slice(character);
            input.push(b',');
        }
        input
    }

    /// Windows-1252 のフランス語が、たまたま CP932 として正しい並びでも、CP932 と確信しない（GitHub Issue #18）。
    #[test]
    fn latin_text_that_is_also_valid_shift_jis_is_not_taken_as_japanese() {
        // `nom;ville` / `Hélène;Orléans` / `Jérôme;Évry` の Windows-1252。`é` + ASCII が CP932 の第二水準の字になる。
        let latin = b"nom;ville\nH\xE9l\xE8ne;Orl\xE9ans\nJ\xE9r\xF4me;\xC9vry\n";
        assert!(shift_jis_valid(latin, true));
        assert_eq!(guess(latin), (TextEncoding::Windows1252, false));
        assert!(
            EncodingGuess::of(latin)
                .candidates()
                .contains(&TextEncoding::ShiftJis)
        );
    }

    /// 日本語の CP932 は、漢字だけでも半角カナだけ（全銀の形式）でも CP932 のまま。
    #[test]
    fn japanese_shift_jis_with_kanji_or_halfwidth_kana_only_stays_shift_jis() {
        // 「山田」「名前」（第一水準）・「ｱｲｳ」（半角カナ・1 バイト）。
        let kanji = line(&[b"\x8E\x52\x93\x63", b"\x96\xBC\x91\x4F"]);
        assert_eq!(guess(&kanji), (TextEncoding::ShiftJis, true));
        let kana = line(&[b"\xB1\xB2\xB3", b"123"]);
        assert_eq!(guess(&kana).0, TextEncoding::ShiftJis);
        // 第一水準と第二水準が混ざっても、第一水準が 1 割あれば日本語とみなす。
        let mut mixed = Vec::new();
        for _ in 0..9_u32 {
            mixed.extend_from_slice(b"\xE9\x6C");
        }
        mixed.extend_from_slice(b"\x8E\x52");
        assert_eq!(guess(&mixed), (TextEncoding::ShiftJis, true));
    }

    #[test]
    fn invalid_bytes_break_each_reading() {
        assert!(!shift_jis_valid(b"\xA0", true));
        assert!(!shift_jis_valid(b"\x81\x20", true));
        assert!(shift_jis_valid(b"\x81", false));
        assert!(!euc_jp_valid(b"\x8E\xE0", true));
        assert!(euc_jp_valid(b"\x8F\xA1\xA1", true));
        assert!(!euc_jp_valid(b"\x8F\xA1\x20", true));
        assert!(!euc_jp_valid(b"\x90", true));
        assert!(!euc_jp_has_hiragana(b"\x8E\xB1\x8F\xA1\xA1\xA5\xA2"));
    }
}
