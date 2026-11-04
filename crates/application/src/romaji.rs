//! かなの読みをローマ字にし、検索のために表記の揺れを寄せる（日本語の入力が無い間の検索・Issue #41）。

use alloc::string::String;

/// かな 2 字（拗音）のローマ字（訓令式）。
const DIGRAPHS: [(&str, &str); 33] = [
    ("きゃ", "kya"),
    ("きゅ", "kyu"),
    ("きょ", "kyo"),
    ("しゃ", "sya"),
    ("しゅ", "syu"),
    ("しょ", "syo"),
    ("ちゃ", "tya"),
    ("ちゅ", "tyu"),
    ("ちょ", "tyo"),
    ("にゃ", "nya"),
    ("にゅ", "nyu"),
    ("にょ", "nyo"),
    ("ひゃ", "hya"),
    ("ひゅ", "hyu"),
    ("ひょ", "hyo"),
    ("みゃ", "mya"),
    ("みゅ", "myu"),
    ("みょ", "myo"),
    ("りゃ", "rya"),
    ("りゅ", "ryu"),
    ("りょ", "ryo"),
    ("ぎゃ", "gya"),
    ("ぎゅ", "gyu"),
    ("ぎょ", "gyo"),
    ("じゃ", "zya"),
    ("じゅ", "zyu"),
    ("じょ", "zyo"),
    ("びゃ", "bya"),
    ("びゅ", "byu"),
    ("びょ", "byo"),
    ("ぴゃ", "pya"),
    ("ぴゅ", "pyu"),
    ("ぴょ", "pyo"),
];

/// かな 1 字のローマ字（訓令式）。
const MONOGRAPHS: [(char, &str); 71] = [
    ('あ', "a"),
    ('い', "i"),
    ('う', "u"),
    ('え', "e"),
    ('お', "o"),
    ('か', "ka"),
    ('き', "ki"),
    ('く', "ku"),
    ('け', "ke"),
    ('こ', "ko"),
    ('さ', "sa"),
    ('し', "si"),
    ('す', "su"),
    ('せ', "se"),
    ('そ', "so"),
    ('た', "ta"),
    ('ち', "ti"),
    ('つ', "tu"),
    ('て', "te"),
    ('と', "to"),
    ('な', "na"),
    ('に', "ni"),
    ('ぬ', "nu"),
    ('ね', "ne"),
    ('の', "no"),
    ('は', "ha"),
    ('ひ', "hi"),
    ('ふ', "hu"),
    ('へ', "he"),
    ('ほ', "ho"),
    ('ま', "ma"),
    ('み', "mi"),
    ('む', "mu"),
    ('め', "me"),
    ('も', "mo"),
    ('や', "ya"),
    ('ゆ', "yu"),
    ('よ', "yo"),
    ('ら', "ra"),
    ('り', "ri"),
    ('る', "ru"),
    ('れ', "re"),
    ('ろ', "ro"),
    ('わ', "wa"),
    ('を', "o"),
    ('ん', "n"),
    ('が', "ga"),
    ('ぎ', "gi"),
    ('ぐ', "gu"),
    ('げ', "ge"),
    ('ご', "go"),
    ('ざ', "za"),
    ('じ', "zi"),
    ('ず', "zu"),
    ('ぜ', "ze"),
    ('ぞ', "zo"),
    ('だ', "da"),
    ('ぢ', "zi"),
    ('づ', "zu"),
    ('で', "de"),
    ('ど', "do"),
    ('ば', "ba"),
    ('び', "bi"),
    ('ぶ', "bu"),
    ('べ', "be"),
    ('ぼ', "bo"),
    ('ぱ', "pa"),
    ('ぴ', "pi"),
    ('ぷ', "pu"),
    ('ぺ', "pe"),
    ('ぽ', "po"),
];

/// ヘボン式などの揺れを訓令式へ寄せる組（長いものから）。
const SPELLINGS: [(&str, &str); 15] = [
    ("sha", "sya"),
    ("shu", "syu"),
    ("sho", "syo"),
    ("cha", "tya"),
    ("chu", "tyu"),
    ("cho", "tyo"),
    ("ja", "zya"),
    ("ju", "zyu"),
    ("jo", "zyo"),
    ("shi", "si"),
    ("chi", "ti"),
    ("tsu", "tu"),
    ("fu", "hu"),
    ("ji", "zi"),
    ("di", "zi"),
];

/// ひらがなの読み `kana` のローマ字（訓令式）。促音（っ）は次の子音を重ね、長音（ー）と知らない字は落とす。
#[must_use]
pub fn romanize(kana: &str) -> String {
    let characters: alloc::vec::Vec<char> = kana.chars().collect();
    let mut out = String::new();
    let mut doubled = false;
    let mut index = 0_usize;
    while let Some(&character) = characters.get(index) {
        let pair: String = characters.iter().skip(index).take(2).collect();
        let (syllable, used) = match DIGRAPHS.iter().find(|(digraph, _)| *digraph == pair) {
            Some((_, roman)) => (Some(*roman), 2_usize),
            None => (
                MONOGRAPHS
                    .iter()
                    .find(|(letter, _)| *letter == character)
                    .map(|(_, roman)| *roman),
                1_usize,
            ),
        };
        index = index.saturating_add(used);
        if character == 'っ' {
            doubled = true;
            continue;
        }
        let Some(roman) = syllable else {
            continue;
        };
        if doubled {
            out.extend(roman.chars().next());
            doubled = false;
        }
        out.push_str(roman);
    }
    out
}

/// 検索のために寄せた形。小文字にし、英字と数字だけを残し、ヘボン式を訓令式へ、長音（`ou`・`oo`・`uu`・`ii`・`ee`・`aa`）を
/// 1 字へ寄せる（shi と si・tsu と tu・長音のありなしの、どちらで打っても当たる）。
#[must_use]
pub fn fold(text: &str) -> String {
    let mut folded: String = text
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|character| character.to_ascii_lowercase())
        .collect();
    for (from, to) in SPELLINGS {
        folded = folded.replace(from, to);
    }
    for (from, to) in [
        ("ou", "o"),
        ("oo", "o"),
        ("uu", "u"),
        ("ii", "i"),
        ("ee", "e"),
        ("aa", "a"),
    ] {
        folded = folded.replace(from, to);
    }
    folded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kana_become_kunrei_romaji() {
        assert_eq!(romanize("はいしょくをえらぶ"), "haisyokuoerabu");
        assert_eq!(romanize("もじをおおきくする"), "mozioookikusuru");
        assert_eq!(romanize("こぴーする"), "kopisuru");
        assert_eq!(romanize("ひょうのまっすぐ"), "hyounomassugu");
        assert_eq!(romanize("ちいさく"), "tiisaku");
        assert_eq!(romanize("abc"), "");
    }

    #[test]
    fn hepburn_and_long_vowels_fold_to_the_same_form() {
        assert_eq!(fold("haishoku"), fold(&romanize("はいしょく")));
        assert_eq!(fold("chiisaku"), fold(&romanize("ちいさく")));
        assert_eq!(fold("okiku"), fold(&romanize("おおきく")));
        assert_eq!(fold("shousai"), fold(&romanize("しょうさい")));
        assert_eq!(fold("Tsu-Ji Fu"), "tuzihu");
        assert_eq!(fold("Theme"), "theme");
    }
}
