//! 表示の幅（半角を 1・全角を 2 と数える）。

/// 全角（幅 2）とみなす範囲（始まりの順・重ならない）。🔴 近似の表。字の測り方を UI から受け取るまでの代わり。
///
/// Unicode の East Asian Width の W と F の主な範囲に、表に使う 2 つの字体（ui-x11 の Noto Sans CJK JP・ui-win32 の
/// Yu Gothic UI）の **どちらかで送り幅が数字 `0` の 1.8 倍以上** の字を足した（GitHub Issue #37・2026-11-05 に字体の
/// ファイルの `cmap` と `hmtx` から測った）。丸数字 `①`・ローマ数字 `Ⅳ`・`※`・`○`・`×`・`→`・`★`・罫線など、日本の
/// 表でよく使う「曖昧な幅」の字は日本語の字体で全角に描かれ、1 と数えると列が狭くて切れる。広すぎる誤りは余白が出る
/// だけなので、どちらかの字体で全角なら 2 とする。結合文字の幅 0 は扱わない。
const WIDE: [(u32, u32); 158] = [
    (0xA7, 0xA7),
    (0xB1, 0xB1),
    (0xB6, 0xB6),
    (0xD7, 0xD7),
    (0xF7, 0xF7),
    (0x1FC, 0x1FC),
    (0x2C9, 0x2C9),
    (0x409, 0x40A),
    (0x42E, 0x42E),
    (0x1100, 0x115F),
    (0x2003, 0x2003),
    (0x2010, 0x2010),
    (0x2014, 0x2016),
    (0x2020, 0x2021),
    (0x2025, 0x2027),
    (0x2030, 0x2030),
    (0x2032, 0x2037),
    (0x203B, 0x203C),
    (0x2042, 0x2042),
    (0x2047, 0x2049),
    (0x2051, 0x2051),
    (0x205A, 0x205A),
    (0x205D, 0x205D),
    (0x20A7, 0x20A7),
    (0x2100, 0x2100),
    (0x2103, 0x2103),
    (0x2105, 0x2105),
    (0x2109, 0x210A),
    (0x210F, 0x210F),
    (0x2113, 0x2113),
    (0x2116, 0x2116),
    (0x2121, 0x2121),
    (0x2126, 0x2127),
    (0x212B, 0x212B),
    (0x2135, 0x2135),
    (0x213B, 0x213B),
    (0x2150, 0x2152),
    (0x2156, 0x215A),
    (0x2160, 0x216B),
    (0x2170, 0x217B),
    (0x217F, 0x217F),
    (0x2189, 0x2189),
    (0x2190, 0x2199),
    (0x21B8, 0x21B9),
    (0x21C4, 0x21C6),
    (0x21CB, 0x21CC),
    (0x21D0, 0x21D0),
    (0x21D2, 0x21D2),
    (0x21D4, 0x21D4),
    (0x21E6, 0x21E9),
    (0x21F5, 0x21F5),
    (0x2200, 0x2200),
    (0x2202, 0x2203),
    (0x2205, 0x220B),
    (0x220F, 0x220F),
    (0x2211, 0x2213),
    (0x2215, 0x2215),
    (0x221A, 0x221A),
    (0x221D, 0x2220),
    (0x2223, 0x2223),
    (0x2225, 0x222E),
    (0x2234, 0x2237),
    (0x223D, 0x223D),
    (0x2243, 0x2243),
    (0x2245, 0x2245),
    (0x2248, 0x2248),
    (0x224C, 0x224C),
    (0x2252, 0x2252),
    (0x2260, 0x2262),
    (0x2264, 0x2267),
    (0x226A, 0x226B),
    (0x226E, 0x226F),
    (0x2272, 0x2273),
    (0x2276, 0x2277),
    (0x2282, 0x2287),
    (0x228A, 0x228B),
    (0x2295, 0x2299),
    (0x22A0, 0x22A0),
    (0x22A5, 0x22A5),
    (0x22BF, 0x22BF),
    (0x22C4, 0x22C4),
    (0x22DA, 0x22DB),
    (0x22EF, 0x22EF),
    (0x2305, 0x2307),
    (0x2312, 0x2312),
    (0x2318, 0x2318),
    (0x2329, 0x232A),
    (0x239B, 0x23AD),
    (0x23B0, 0x23B1),
    (0x23BE, 0x23CC),
    (0x23CE, 0x23CE),
    (0x23DA, 0x23DB),
    (0x2423, 0x2423),
    (0x2460, 0x25AB),
    (0x25B1, 0x25B3),
    (0x25B6, 0x25B7),
    (0x25BC, 0x25BD),
    (0x25C0, 0x25C1),
    (0x25C6, 0x25C7),
    (0x25C9, 0x25CC),
    (0x25CE, 0x25D3),
    (0x25E2, 0x25E6),
    (0x25EF, 0x25EF),
    (0x25FB, 0x25FC),
    (0x2600, 0x2603),
    (0x2605, 0x2606),
    (0x2609, 0x2609),
    (0x260E, 0x260F),
    (0x2616, 0x2617),
    (0x261C, 0x261F),
    (0x262F, 0x262F),
    (0x2640, 0x2642),
    (0x2660, 0x266F),
    (0x2672, 0x267D),
    (0x26A0, 0x26A0),
    (0x26AA, 0x26AB),
    (0x26BD, 0x26BE),
    (0x26CB, 0x26CB),
    (0x2702, 0x2702),
    (0x2713, 0x2713),
    (0x271A, 0x271A),
    (0x273D, 0x273D),
    (0x273F, 0x2740),
    (0x2756, 0x2756),
    (0x2776, 0x2793),
    (0x27A1, 0x27A1),
    (0x2934, 0x2935),
    (0x29BF, 0x29BF),
    (0x29FA, 0x29FB),
    (0x2B05, 0x2B07),
    (0x2B1A, 0x2B1A),
    (0x2B25, 0x2B26),
    (0x2B29, 0x2B29),
    (0x2B60, 0x2B65),
    (0x2B82, 0x2B83),
    (0x2B95, 0x2B95),
    (0x2E3A, 0x2E3B),
    (0x2E80, 0x303F),
    (0x3041, 0x4DBF),
    (0x4E00, 0xA4CF),
    (0xAC00, 0xD7A3),
    (0xF900, 0xFAFF),
    (0xFE10, 0xFE19),
    (0xFE30, 0xFE52),
    (0xFE54, 0xFE66),
    (0xFE68, 0xFE6B),
    (0xFF00, 0xFF60),
    (0xFFE0, 0xFFE6),
    (0x1_B000, 0x1_B001),
    (0x1_F100, 0x1_F10C),
    (0x1_F110, 0x1_F12E),
    (0x1_F130, 0x1_F169),
    (0x1_F170, 0x1_F1AC),
    (0x1_F200, 0x1_F202),
    (0x1_F210, 0x1_F23B),
    (0x1_F240, 0x1_F248),
    (0x1_F250, 0x1_F251),
    (0x1_F300, 0x3_FFFD),
];

/// 前の字に重なって描かれ、幅を持たない字（始まりの順・重ならない）。結合文字・濁点と半濁点の結合形・異体字の選択子・
/// 肌の色の修飾・ゼロ幅の字と結合子・BOM。🔴 近似の表（書記素の規則の全部ではない・GitHub Issue #42）。
const EXTENDING: [(u32, u32); 13] = [
    (0x300, 0x36F),
    (0x1AB0, 0x1AFF),
    (0x1DC0, 0x1DFF),
    (0x200B, 0x200F),
    (0x2060, 0x2064),
    (0x20D0, 0x20FF),
    (0x3099, 0x309A),
    (0xFE00, 0xFE0F),
    (0xFE20, 0xFE2F),
    (0xFEFF, 0xFEFF),
    (0x1_F3FB, 0x1_F3FF),
    (0xE_0020, 0xE_007F),
    (0xE_0100, 0xE_01EF),
];

/// 前の字に重なって描かれ、幅を持たない字か（折り返しで前の字から離さない・GitHub Issue #42）。
#[must_use]
pub fn extends_previous(character: char) -> bool {
    in_ranges(&EXTENDING, u32::from(character))
}

/// `code` が範囲の表 `ranges`（始まりの順・重ならない）のどれかに入るか。
fn in_ranges(ranges: &[(u32, u32)], code: u32) -> bool {
    let after = ranges.partition_point(|&(_, last)| last < code);
    ranges.get(after).is_some_and(|&(first, _)| first <= code)
}

/// 1 字の表示の幅（半角 1・全角 2・前の字に重なる字は 0）。
#[must_use]
pub fn char_columns(character: char) -> u32 {
    if extends_previous(character) {
        return 0;
    }
    if in_ranges(&WIDE, u32::from(character)) {
        2
    } else {
        1
    }
}

/// 文字列の表示の幅（半角 1・全角 2 の合計）。
#[must_use]
pub fn display_columns(text: &str) -> u32 {
    text.chars()
        .map(char_columns)
        .fold(0_u32, u32::saturating_add)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_is_one_and_japanese_is_two() {
        assert_eq!(display_columns(""), 0);
        assert_eq!(display_columns("abc,1"), 5);
        assert_eq!(display_columns("東京"), 4);
        assert_eq!(display_columns("あア"), 4);
        assert_eq!(display_columns("ＡＢ"), 4);
        assert_eq!(display_columns("한글"), 4);
    }

    #[test]
    fn half_width_kana_and_latin_letters_are_one() {
        assert_eq!(display_columns("ｱｲｳ"), 3);
        assert_eq!(display_columns("café"), 4);
        assert_eq!(char_columns('😀'), 2);
    }

    /// 日本の表でよく使う曖昧な幅の字は、日本語の字体で全角に描かれるので 2（GitHub Issue #37）。
    #[test]
    fn symbols_drawn_full_width_in_japanese_fonts_are_two() {
        assert_eq!(display_columns("①⑳"), 4);
        assert_eq!(display_columns("ⅠⅣⅫ"), 6);
        for symbol in [
            '※', '○', '×', '△', '□', '■', '→', '⇒', '★', '☆', '♪', '℃', '¶', '─', '┼',
        ] {
            assert_eq!(char_columns(symbol), 2, "{symbol}");
        }
        // ラテン文字・数字・半角カナ・ASCII の記号は 1 のまま。
        for narrow in ['a', 'é', '0', '-', '~', 'ｱ', '¥'] {
            assert_eq!(char_columns(narrow), 1, "{narrow}");
        }
    }

    /// 表は始まりの順で、重ならない（二分探索の前提）。
    #[test]
    fn the_wide_ranges_are_sorted_and_disjoint() {
        assert!(WIDE.iter().all(|&(first, last)| first <= last));
        assert!(WIDE.windows(2).all(|pair| pair[0].1 < pair[1].0));
        let columns_at = |code: u32| char::from_u32(code).map(char_columns);
        for &(first, last) in &WIDE {
            assert_eq!(columns_at(first), Some(2));
            assert_eq!(columns_at(last), Some(2));
            // 範囲のすぐ後ろは、次の範囲が続いていなければ 1（ここでの範囲は隣り合わない）。
            assert_eq!(columns_at(last + 1), Some(1));
        }
    }

    /// 結合文字・濁点の結合形・異体字の選択子・ゼロ幅の字は幅 0（GitHub Issue #42。前は 1 か 2 と数えた）。
    #[test]
    fn combining_and_zero_width_characters_take_no_room() {
        assert_eq!(display_columns("e\u{301}"), 1);
        assert_eq!(display_columns("か\u{3099}"), 2);
        assert_eq!(display_columns("\u{2764}\u{FE0F}"), 1);
        assert_eq!(display_columns("a\u{200B}b"), 2);
        assert_eq!(display_columns("\u{FEFF}"), 0);
        assert!(extends_previous('\u{1F3FB}'));
        assert!(!extends_previous('a'));
        assert!(extends_previous('\u{200D}'));
    }
}
