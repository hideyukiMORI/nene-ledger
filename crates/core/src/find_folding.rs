//! 検索で比べる前に、条件で区別しない違いを畳む（ADR 0030 の決定 3）。

use alloc::string::String;
use alloc::vec::Vec;
use core::ops::Range;

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
    // 検索の熱い道なので、元の範囲を数えずに畳む（範囲を数える `fold_steps` と同じ字を出すことはテストで固定）。
    let mut folded = String::with_capacity(text.len());
    // 位置は使わないので数えない（`char_indices` より速い）。
    let mut chars = text
        .chars()
        .map(|character| (0_usize, character))
        .peekable();
    while let Some((start, first)) = chars.next() {
        let character = fold_one(first, start, &mut chars, options).0;
        push_folded(&mut folded, character, options);
    }
    folded
}

/// `fold_for_find` と同じ写しと、写しの字ごとの元の範囲（写しの何バイト目から、元の `text` のどのバイトの範囲か）。
/// 写しで当たった所を元の文字列に戻すのに使う（置換・ADR 0030 の決定 6）。範囲は写しの順に並び、重ならない。
#[must_use]
pub fn fold_with_origins(text: &str, options: FindOptions) -> (String, Vec<(usize, Range<usize>)>) {
    let mut folded = String::with_capacity(text.len());
    let mut origins = Vec::with_capacity(text.len());
    for (character, range) in fold_steps(text, options) {
        origins.push((folded.len(), range));
        push_folded(&mut folded, character, options);
    }
    (folded, origins)
}

/// 畳んだ字 `character` を写し `folded` に足す（大文字小文字を区別しなければ小文字にして）。
fn push_folded(folded: &mut String, character: char, options: FindOptions) {
    if options.match_case() {
        folded.push(character);
    } else {
        folded.extend(character.to_lowercase());
    }
}

/// `text` を前から 1 字ずつ畳む（小文字にする前の字と、元の `text` のバイトの範囲）。半角カタカナと後ろの濁点・半濁点は
/// 合わせて 1 字になる。
fn fold_steps(text: &str, options: FindOptions) -> impl Iterator<Item = (char, Range<usize>)> + '_ {
    let mut chars = text.char_indices().peekable();
    core::iter::from_fn(move || {
        let (start, first) = chars.next()?;
        let (character, end) = fold_one(first, start, &mut chars, options);
        Some((character, start..end))
    })
}

/// 元の `start` バイト目の字 `first` を畳んだ字（小文字にする前）と、元の範囲の終わり。続く濁点・半濁点は `rest` から読む。
fn fold_one<I: Iterator<Item = (usize, char)>>(
    first: char,
    start: usize,
    rest: &mut core::iter::Peekable<I>,
    options: FindOptions,
) -> (char, usize) {
    let mut character = first;
    let mut end = start.saturating_add(first.len_utf8());
    if !options.match_width() {
        character = narrowed(character);
        if let Some(full) = widened_kana(character) {
            (character, end) = joined_with_mark(full, end, rest);
        }
    }
    if !options.match_kana() {
        character = hiragana(character);
    }
    (character, end)
}

/// 半角カタカナを全角にした `full`（元の範囲の終わり `end`）に、続く半角の濁点・半濁点 `rest` の頭を合わせた字と、
/// 合わせた後の終わり（合わせたら印を読み進める）。
fn joined_with_mark<I: Iterator<Item = (usize, char)>>(
    full: char,
    end: usize,
    rest: &mut core::iter::Peekable<I>,
) -> (char, usize) {
    let Some(&(at, mark @ (HALF_VOICED | HALF_SEMI_VOICED))) = rest.peek() else {
        return (full, end);
    };
    let Some(joined) = voiced(full, mark == HALF_SEMI_VOICED) else {
        return (full, end);
    };
    let _mark = rest.next();
    (joined, at.saturating_add(mark.len_utf8()))
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
    use alloc::vec;

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

    /// 写しの字ごとの元の範囲。半角カタカナと濁点は 2 字で 1 つの範囲、全角の英字は 3 バイトで 1 バイトの字。
    #[test]
    fn origins_point_back_to_the_source_bytes() {
        let (folded, origins) = fold_with_origins("Ａｶﾞx", FindOptions::standard());
        assert_eq!(folded, "aガx");
        assert_eq!(origins, vec![(0, 0..3), (1, 3..9), (4, 9..10)]);
        for text in ["ＡＢｃ１２３　Ｘ", "ｶﾞｷﾞｸﾞ ﾊﾟﾋﾟ", "ÄÖÜ", "plain", ""]
        {
            let (copy, ranges) = fold_with_origins(text, FindOptions::standard());
            assert_eq!(copy, standard(text));
            let covered: usize = ranges.iter().map(|(_, range)| range.len()).sum();
            assert_eq!(covered, text.len());
        }
    }
}
