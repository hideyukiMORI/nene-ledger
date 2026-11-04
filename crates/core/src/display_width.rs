//! 表示の幅（半角を 1・全角を 2 と数える）。

/// 全角（幅 2）とみなす範囲。🔴 Unicode の East Asian Width の W と F の主な範囲だけを持つ近似。
/// 完全な表（結合文字の幅 0・曖昧な幅の扱い）は、字の測り方を UI から受け取る Issue で見直す。
const WIDE: [(u32, u32); 12] = [
    (0x1100, 0x115F),     // ハングルの初声
    (0x2E80, 0x303E),     // CJK の部首・記号と句読点
    (0x3041, 0x33FF),     // ひらがな・カタカナ・CJK の互換
    (0x3400, 0x4DBF),     // CJK 統合漢字 拡張 A
    (0x4E00, 0x9FFF),     // CJK 統合漢字
    (0xA000, 0xA4CF),     // イ文字
    (0xAC00, 0xD7A3),     // ハングルの音節
    (0xF900, 0xFAFF),     // CJK 互換漢字
    (0xFE30, 0xFE4F),     // CJK 互換形
    (0xFF00, 0xFF60),     // 全角形
    (0xFFE0, 0xFFE6),     // 全角の記号
    (0x1_F300, 0x3_FFFD), // 絵文字の主な範囲・CJK 拡張 B 以降
];

/// 1 字の表示の幅（半角 1・全角 2）。
#[must_use]
pub fn char_columns(character: char) -> u32 {
    let code = u32::from(character);
    let wide = WIDE
        .iter()
        .any(|&(first, last)| (first..=last).contains(&code));
    if wide { 2 } else { 1 }
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
        assert_eq!(char_columns('¶'), 1);
        assert_eq!(char_columns('😀'), 2);
    }
}
