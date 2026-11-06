//! 検索で比べる前に、条件で区別しない違いを畳む（ADR 0030 の決定 3）。

use alloc::string::String;

use crate::find_options::FindOptions;

/// 半角カタカナ（U+FF61〜U+FF9F）に当たる全角の字。並びは符号の順。
const HALF_WIDTH_KANA: [char; 63] = [
    '。', '「', '」', '、', '・', 'ヲ', 'ァ', 'ィ', 'ゥ', 'ェ', 'ォ', 'ャ', 'ュ', 'ョ', 'ッ', 'ー',
    'ア', 'イ', 'ウ', 'エ', 'オ', 'カ', 'キ', 'ク', 'ケ', 'コ', 'サ', 'シ', 'ス', 'セ', 'ソ', 'タ',
    'チ', 'ツ', 'テ', 'ト', 'ナ', 'ニ', 'ヌ', 'ネ', 'ノ', 'ハ', 'ヒ', 'フ', 'ヘ', 'ホ', 'マ', 'ミ',
    'ム', 'メ', 'モ', 'ヤ', 'ユ', 'ヨ', 'ラ', 'リ', 'ル', 'レ', 'ロ', 'ワ', 'ン', '゛', '゜',
];

/// 半角の濁点（ﾞ）。
const HALF_VOICED: char = '\u{FF9E}';
/// 半角の半濁点（ﾟ）。
const HALF_SEMI_VOICED: char = '\u{FF9F}';

/// `text` の、条件 `options` で区別しない違いを畳んだ写し。全角半角を区別しなければ全角の英数字・記号・空白を半角に、
/// 半角カタカナを全角に（後ろの濁点と半濁点は前の字と合わせる）。大文字小文字を区別しなければ小文字に。ひらがなと
/// カタカナを区別しなければカタカナをひらがなに。
#[must_use]
pub fn fold_for_find(text: &str, options: FindOptions) -> String {
    let mut folded = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(character) = chars.next() {
        let mut character = character;
        if !options.match_width() {
            character = narrowed(character);
            if let Some(full) = widened_kana(character) {
                character = joined_with_mark(full, &mut chars);
            }
        }
        if !options.match_kana() {
            character = hiragana(character);
        }
        if options.match_case() {
            folded.push(character);
        } else {
            folded.extend(character.to_lowercase());
        }
    }
    folded
}

/// 半角カタカナを全角にした `full` に、続く半角の濁点・半濁点 `rest` の頭を合わせた字（合わせたら印を読み進める）。
fn joined_with_mark(full: char, rest: &mut core::iter::Peekable<core::str::Chars<'_>>) -> char {
    let Some(mark @ (HALF_VOICED | HALF_SEMI_VOICED)) = rest.peek().copied() else {
        return full;
    };
    let Some(joined) = voiced(full, mark == HALF_SEMI_VOICED) else {
        return full;
    };
    let _mark = rest.next();
    joined
}

/// 全角の英数字・記号（U+FF01〜U+FF5E）と全角の空白を半角に。ほかはそのまま。
fn narrowed(character: char) -> char {
    match character {
        '\u{3000}' => ' ',
        '\u{FF01}'..='\u{FF5E}' => {
            char::from_u32(u32::from(character).saturating_sub(0xFEE0)).unwrap_or(character)
        }
        _ => character,
    }
}

/// 半角カタカナなら全角の字。
fn widened_kana(character: char) -> Option<char> {
    let offset = u32::from(character).checked_sub(0xFF61)?;
    HALF_WIDTH_KANA.get(usize::try_from(offset).ok()?).copied()
}

/// 濁点を合わせると次の符号の字になる全角カタカナ。
const VOICEABLE: &str = "カキクケコサシスセソタチツテトハヒフヘホ";
/// 半濁点を合わせると 2 つ先の符号の字になる全角カタカナ。
const SEMI_VOICEABLE: &str = "ハヒフヘホ";

/// 全角カタカナ `base` に濁点（`semi` なら半濁点）を合わせた字。合わせられなければ `None`。
fn voiced(base: char, semi: bool) -> Option<char> {
    let step = match (semi, base) {
        (false, 'ウ') => return Some('ヴ'),
        (false, _) if VOICEABLE.contains(base) => 1_u32,
        (true, _) if SEMI_VOICEABLE.contains(base) => 2_u32,
        (false | true, _) => return None,
    };
    char::from_u32(u32::from(base).checked_add(step)?)
}

/// カタカナ（ァ〜ヶ）ならひらがな。ほかはそのまま。
fn hiragana(character: char) -> char {
    match character {
        '\u{30A1}'..='\u{30F6}' => {
            char::from_u32(u32::from(character).saturating_sub(0x60)).unwrap_or(character)
        }
        _ => character,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn standard(text: &str) -> String {
        fold_for_find(text, FindOptions::standard())
    }

    #[test]
    fn width_and_case_fold_by_default_and_kana_is_kept() {
        assert_eq!(standard("ＡＢｃ１２３　Ｘ"), "abc123 x");
        assert_eq!(standard("ｶﾞｷﾞｸﾞ ﾊﾟﾋﾟ ｳﾞ ｱｲｳ ｰ"), "ガギグ パピ ヴ アイウ ー");
        assert_eq!(standard("ｯﾞ"), "ッ゛");
        assert_eq!(standard("ﾄﾞﾎﾞﾎﾟ"), "ドボポ");
        assert_eq!(standard("ﾂﾞﾃﾞﾁﾞﾀﾞ"), "ヅデヂダ");
        assert_eq!(standard("カタカナ"), "カタカナ");
        assert_eq!(standard("ÄÖÜ"), "äöü");
    }

    #[test]
    fn each_option_turns_its_fold_off_or_on() {
        let options = FindOptions::standard();
        assert_eq!(fold_for_find("ＡＢ", options.toggled_case()), "AB");
        assert_eq!(fold_for_find("ＡＢ", options.toggled_width()), "ａｂ");
        assert_eq!(
            fold_for_find("カタカナとかな", options.toggled_kana()),
            "かたかなとかな"
        );
        assert_eq!(fold_for_find("ｶﾞ", options.toggled_kana()), "が");
    }
}
