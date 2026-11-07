//! 検索の問い（ADR 0030 の決定 2・3）。

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::ops::Range;

use regex_automata::meta::Regex;
use regex_automata::util::interpolate;
use regex_automata::util::primitives::PatternID;
use regex_automata::util::syntax;

use crate::find_folding::{fold_for_find, fold_with_origins};
use crate::find_options::FindOptions;

/// 探す文字列と条件。セルの値（見えている値の文字列）が当たるかを決める。値は文字列のまま比べる（数や日付として
/// 比べない・KEEP-01）。正規表現の条件なら式を組み立てて持つ（ADR 0031）。読めない式はどのセルにも当たらない。
/// 等しさは打った文字列と条件で決める（式はそこから作るので）。
#[derive(Debug, Clone)]
pub struct FindQuery {
    text: String,
    folded: String,
    options: FindOptions,
    regex: Option<Regex>,
}

impl PartialEq for FindQuery {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text && self.options == other.options
    }
}

impl Eq for FindQuery {}

impl FindQuery {
    /// `text` を条件 `options` で探す問い。
    #[must_use]
    pub fn new(text: &str, options: FindOptions) -> Self {
        Self {
            text: String::from(text),
            folded: fold_for_find(text, options),
            options,
            regex: compiled(text, options),
        }
    }

    /// 正規表現の条件で、式が読めないか（どのセルにも当たらない・画面はそう知らせる）。
    #[must_use]
    pub fn is_broken(&self) -> bool {
        self.options.regex() && !self.folded.is_empty() && self.regex.is_none()
    }

    /// 打った文字列。
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// 条件。
    #[must_use]
    pub const fn options(&self) -> FindOptions {
        self.options
    }

    /// 何も探さない（空の）問いか。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.folded.is_empty()
    }

    /// セルの値 `cell` が当たるか。空の問いはどのセルにも当たらない。
    #[must_use]
    pub fn matches(&self, cell: &str) -> bool {
        if self.folded.is_empty() {
            return false;
        }
        if self.options.regex() {
            return self.regex_matches(cell);
        }
        // 速い道：ASCII だけのセルは、畳んでも ASCII のまま（全角も半角カタカナも無い）。問いが ASCII でなければ
        // 当たらず、ASCII なら写しを作らずに比べる。
        if cell.is_ascii() {
            if !self.folded.is_ascii() {
                return false;
            }
            return self.ascii_matches(cell.as_bytes());
        }
        let folded = fold_for_find(cell, self.options);
        if self.options.whole_cell() {
            folded == self.folded
        } else {
            folded.contains(self.folded.as_str())
        }
    }

    /// セルの値 `cell` の当たった所を `replacement` に置き換えた値（置換・ADR 0030 の決定 6）。当たらなければ `None`
    /// （`matches` と同じ答え）。セル全体の一致ならセルの値をそのまま置き換える。部分一致なら、畳んだ写しで左から重ならずに
    /// 当たった所を元の文字列の範囲に戻して置き換え、当たらない所の字は元のまま残す（揃えない・KEEP-01）。
    #[must_use]
    pub fn replaced(&self, cell: &str, replacement: &str) -> Option<String> {
        if !self.matches(cell) {
            return None;
        }
        if let Some(regex) = &self.regex {
            return regex_replaced(regex, cell, replacement, self.options);
        }
        if self.options.whole_cell() {
            return Some(String::from(replacement));
        }
        let mut result = String::with_capacity(cell.len());
        let mut kept = 0_usize;
        for original in self.plain_spans(cell) {
            result.push_str(cell.get(kept..original.start)?);
            result.push_str(replacement);
            kept = original.end;
        }
        result.push_str(cell.get(kept..)?);
        Some(result)
    }

    /// セルの値 `cell` の中で当たった所の、元の文字列のバイトの範囲（左から順・重ならない・長さ 0 は無い）。画面が当たった
    /// 字に色を付けるのに使う（GitHub Issue #189）。置き換える所と同じ範囲で、当たらなければ空（`matches` と同じ答え）。
    /// セル全体の一致ならセルの全体。
    #[must_use]
    pub fn spans(&self, cell: &str) -> Vec<Range<usize>> {
        if self.folded.is_empty() {
            return Vec::new();
        }
        // 当たるかを先に `matches` で確かめると、同じセルを 2 回畳むことになる。範囲が空なら当たらない（`matches` と同じ
        // 答えであることはテストで固定している）。
        if self.options.regex() {
            let Some(regex) = &self.regex else {
                return Vec::new();
            };
            let (folded, origins) = fold_with_origins(cell, self.options);
            let found = regex
                .find_iter(folded.as_str())
                .filter(|found| !found.is_empty())
                .filter_map(|found| original_range(&origins, found.range()));
            return without_overlaps(found);
        }
        if self.options.whole_cell() {
            return if self.matches(cell) {
                alloc::vec![0..cell.len()]
            } else {
                Vec::new()
            };
        }
        self.plain_spans(cell)
    }

    /// 正規表現でない部分一致の、当たった所の元の範囲。畳んだ写しで左から重ならずに当たった所を元の文字列の範囲に戻す。
    /// 小文字にして 1 字が数字に増えた字の途中で切れても元の字を丸ごと含め、前と重なれば飛ばす。
    fn plain_spans(&self, cell: &str) -> Vec<Range<usize>> {
        let (folded, origins) = fold_with_origins(cell, self.options);
        let found = folded
            .match_indices(self.folded.as_str())
            .filter_map(|(at, part)| original_range(&origins, at..at.saturating_add(part.len())));
        without_overlaps(found)
    }

    /// 正規表現の条件で、セルの値 `cell` に長さのある当たりがあるか。ASCII だけのセルは写しを作らずに照らす（畳んでも
    /// 小文字になるだけで、式は大文字小文字の設定で同じ答えになる）。
    fn regex_matches(&self, cell: &str) -> bool {
        let Some(regex) = &self.regex else {
            return false;
        };
        if cell.is_ascii() {
            return has_long_match(regex, cell);
        }
        has_long_match(regex, &fold_for_find(cell, self.options))
    }

    /// ASCII だけの原本のレコードのバイト列 `raw`（区切りと引用を含む）のどのセルにも当たらないと言えるか。復号せずに
    /// 照らす速い道（ADR 0030 の決定 4）。ASCII のセルを畳んでも ASCII のままなので、問いが ASCII でなければ当たらない。
    /// ASCII の問いは、セルに当たればレコードのバイト列にも同じ字の並びがある（引用の中の `"` は二重になるので、`"` を
    /// 含む問いでは言えない）。
    #[must_use]
    pub fn rules_out_ascii(&self, raw: &[u8]) -> bool {
        if self.options.regex() {
            // 式の字の並びがレコードに無いとは言えない（読めない式はどこにも当たらない）。
            return self.regex.is_none();
        }
        if self.folded.is_empty() || !self.folded.is_ascii() {
            return true;
        }
        if self.folded.contains('"') {
            return false;
        }
        let wanted = self.folded.as_bytes();
        let Some(&first) = wanted.first() else {
            return true;
        };
        let case = self.options.match_case();
        let same = |left: u8, right: u8| {
            if case {
                left == right
            } else {
                left.eq_ignore_ascii_case(&right)
            }
        };
        !raw.iter().enumerate().any(|(at, &byte)| {
            same(byte, first)
                && raw
                    .get(at..at.saturating_add(wanted.len()))
                    .is_some_and(|part| {
                        part.len() == wanted.len()
                            && part.iter().zip(wanted).all(|(&a, &b)| same(a, b))
                    })
        })
    }

    /// ASCII だけのセル `cell` が当たるか（問いも ASCII）。
    fn ascii_matches(&self, cell: &[u8]) -> bool {
        let wanted = self.folded.as_bytes();
        let same = |part: &[u8]| {
            if self.options.match_case() {
                part == wanted
            } else {
                part.eq_ignore_ascii_case(wanted)
            }
        };
        if self.options.whole_cell() {
            return same(cell);
        }
        cell.windows(wanted.len()).any(same)
    }
}

/// 正規表現の条件なら、打った `text` を組み立てた式（読めなければ `None`）。式は大文字小文字を除いてセルと同じ畳みで畳む
/// （`\D` を `\d` にしないため）。大文字小文字は式の設定で扱う。セル全体の一致は式を頭と終わりに固定する。
fn compiled(text: &str, options: FindOptions) -> Option<Regex> {
    if !options.regex() || text.is_empty() {
        return None;
    }
    let folded = fold_for_find(text, options.matching_case());
    let pattern = if options.whole_cell() {
        format!("\\A(?:{folded})\\z")
    } else {
        folded
    };
    Regex::builder()
        .syntax(syntax::Config::new().case_insensitive(!options.match_case()))
        .build(&pattern)
        .ok()
}

/// `haystack` に長さのある当たりがあるか（長さ 0 の当たりは当たりにしない・空の式がどのセルにも当たらないように）。
fn has_long_match(regex: &Regex, haystack: &str) -> bool {
    regex.find_iter(haystack).any(|found| !found.is_empty())
}

/// 左から並んだ元の範囲 `ranges` のうち、前に残した範囲と重なるものを飛ばした並び。
fn without_overlaps(ranges: impl Iterator<Item = Range<usize>>) -> Vec<Range<usize>> {
    let mut kept: Vec<Range<usize>> = Vec::new();
    for range in ranges {
        if kept.last().is_none_or(|last| last.end <= range.start) {
            kept.push(range);
        }
    }
    kept
}

/// 写しのバイトの範囲 `span` を含む字の、元の範囲（写しの字ごとの元の範囲 `origins` から）。
fn original_range(origins: &[(usize, Range<usize>)], span: Range<usize>) -> Option<Range<usize>> {
    let at = |position: usize| {
        let index = origins.partition_point(|(start, _)| *start <= position);
        origins
            .get(index.saturating_sub(1))
            .map(|(_, range)| range.clone())
    };
    let first = at(span.start)?;
    let last = at(span.end.checked_sub(1)?)?;
    Some(first.start..last.end)
}

/// 正規表現の当たった所を `replacement` に置き換えた値。`$1`・`${name}` は当たった組の**元の字**で埋める（揃えない・
/// KEEP-01）。長さ 0 の当たりと、前の当たりと元の字で重なる当たりは飛ばす。
fn regex_replaced(
    regex: &Regex,
    cell: &str,
    replacement: &str,
    options: FindOptions,
) -> Option<String> {
    let (folded, origins) = fold_with_origins(cell, options);
    let mut result = String::with_capacity(cell.len());
    let mut kept = 0_usize;
    for captures in regex.captures_iter(folded.as_str()) {
        let Some(found) = captures.get_match() else {
            continue;
        };
        if found.is_empty() {
            continue;
        }
        let Some(original) = original_range(&origins, found.range()) else {
            continue;
        };
        if original.start < kept {
            continue;
        }
        result.push_str(cell.get(kept..original.start)?);
        let groups: Vec<Option<Range<usize>>> = (0..captures.group_len())
            .map(|index| {
                captures
                    .get_group(index)
                    .filter(|span| !span.is_empty())
                    .and_then(|span| original_range(&origins, span.range()))
            })
            .collect();
        interpolate::string(
            replacement,
            |index, out| {
                if let Some(text) = groups
                    .get(index)
                    .cloned()
                    .flatten()
                    .and_then(|range| cell.get(range))
                {
                    out.push_str(text);
                }
            },
            |name| captures.group_info().to_index(PatternID::ZERO, name),
            &mut result,
        );
        kept = original.end;
    }
    result.push_str(cell.get(kept..)?);
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query(text: &str) -> FindQuery {
        FindQuery::new(text, FindOptions::standard())
    }

    #[test]
    fn the_standard_finds_parts_ignoring_case_and_width() {
        assert!(query("tok").matches("Tokyo"));
        assert!(query("ＴＯＫ").matches("tokyo"));
        assert!(query("tok").matches("ＴＯＫＹＯ"));
        assert!(query("ｶﾞｽ").matches("都市ガス"));
        assert!(query("0012").matches("00120"));
        assert!(!query("ガス").matches("がす"));
        assert!(!query("").matches("anything"));
        assert!(!query("東").matches("tokyo"));
    }

    #[test]
    fn options_make_the_match_stricter_or_looser() {
        let options = FindOptions::standard();
        let case = FindQuery::new("Tok", options.toggled_case());
        assert!(case.matches("Tokyo") && !case.matches("tokyo"));
        let whole = FindQuery::new("tokyo", options.toggled_whole_cell());
        assert!(whole.matches("TOKYO") && !whole.matches("tokyo2") && whole.matches("ＴＯＫＹＯ"));
        let width = FindQuery::new("ＡＢ", options.toggled_width());
        assert!(width.matches("ａｂ") && !width.matches("ab"));
        let kana = FindQuery::new("がす", options.toggled_kana());
        assert!(kana.matches("ガス") && kana.matches("ｶﾞｽ"));
    }

    /// レコードのバイト列で除けるのは、どのセルにも当たらないときだけ（引用の `"` を含む問いは除かない）。
    #[test]
    fn a_record_is_ruled_out_only_when_no_cell_can_match() {
        let find = query("tokyo");
        assert!(!find.rules_out_ascii(b"1,TOKYO,x"));
        assert!(!find.rules_out_ascii(b"1,\"tokyo, jp\",x"));
        assert!(find.rules_out_ascii(b"1,osaka,x"));
        assert!(query("東京").rules_out_ascii(b"1,tokyo"));
        assert!(!query("a\"b").rules_out_ascii(b"1,\"a\"\"b\""));
        let case = FindQuery::new("Tok", FindOptions::standard().toggled_case());
        assert!(case.rules_out_ascii(b"tokyo") && !case.rules_out_ascii(b"Tokyo"));
    }

    /// 当たった所だけを替え、ほかの字（全角・半角カタカナ・大文字）は元のまま残す。
    #[test]
    fn replacing_keeps_the_unmatched_characters_as_they_were() {
        assert_eq!(
            query("tok").replaced("Tokyo ＴＯＫ", "X"),
            Some(String::from("Xyo X"))
        );
        assert_eq!(
            query("ガ").replaced("ｶﾞｽ ｶﾞ", "が"),
            Some(String::from("がｽ が"))
        );
        assert_eq!(query("aa").replaced("aaa", "b"), Some(String::from("ba")));
        assert_eq!(
            query("ＡＢ").replaced("xab-AB", ""),
            Some(String::from("x-"))
        );
        assert_eq!(query("tok").replaced("osaka", "X"), None);
        assert_eq!(query("").replaced("anything", "X"), None);
        let whole = FindQuery::new("tokyo", FindOptions::standard().toggled_whole_cell());
        assert_eq!(
            whole.replaced("ＴＯＫＹＯ", "東京"),
            Some(String::from("東京"))
        );
        assert_eq!(whole.replaced("tokyo2", "東京"), None);
        let case = FindQuery::new("Tok", FindOptions::standard().toggled_case());
        assert_eq!(case.replaced("tok Tok", "-"), Some(String::from("tok -")));
    }

    /// 置き換えられるのは当たるセルだけ。置き換えた後に元の問いを探すと、置き換えた字に問いが無ければ当たらない。
    #[test]
    fn replacing_agrees_with_matching() {
        let cells = [
            "Tokyo",
            "ｶﾞｽ",
            "ＴＯＫ",
            "x-tok-y",
            "",
            "ガス",
            "がす",
            "İstanbul",
            "ﾄｰｷｮｰ",
        ];
        let queries = ["tok", "ｶﾞ", "ガス", "o", "i", "ｰ", "zz"];
        let standard = FindOptions::standard();
        for options in [
            standard,
            standard.toggled_case(),
            standard.toggled_whole_cell(),
            standard.toggled_width(),
            standard.toggled_kana(),
        ] {
            for (text, cell) in queries
                .iter()
                .flat_map(|&text| cells.iter().map(move |&cell| (text, cell)))
            {
                let find = FindQuery::new(text, options);
                let replaced = find.replaced(cell, "#");
                assert_eq!(
                    replaced.is_some(),
                    find.matches(cell),
                    "{text:?} {cell:?} {options:?}"
                );
                let still = replaced.as_deref().is_some_and(|after| find.matches(after));
                assert!(!still, "{text:?} {cell:?} → {replaced:?} {options:?}");
            }
        }
    }

    /// 当たった所の元の範囲は、元の字の境目にあり、左から並び、重ならない（GitHub Issue #189）。
    #[test]
    fn spans_are_the_matched_parts_of_the_original_cell() {
        assert_eq!(query("tok").spans("Tokyo ＴＯＫ"), [0..3, 6..15]);
        assert_eq!(query("ガ").spans("ｶﾞｽ ｶﾞ"), [0..6, 10..16]);
        assert_eq!(query("aa").spans("aaa"), alloc::vec![0..2]);
        assert!(query("tok").spans("osaka").is_empty());
        assert!(query("").spans("anything").is_empty());
        let whole = FindQuery::new("tokyo", FindOptions::standard().toggled_whole_cell());
        assert_eq!(whole.spans("ＴＯＫＹＯ"), alloc::vec![0..15]);
        assert_eq!(regex(r"\d+").spans("a12b3"), [1..3, 4..5]);
        assert!(regex(r"x*").spans("abc").is_empty());
    }

    /// 範囲を `#` に替えた値。範囲は左から並び、重ならず、長さがある。
    fn rewritten(cell: &str, spans: &[Range<usize>]) -> String {
        let mut rebuilt = String::new();
        let mut kept = 0_usize;
        for span in spans {
            assert!(span.start >= kept && span.start < span.end, "{spans:?}");
            rebuilt.push_str(cell.get(kept..span.start).unwrap_or("?"));
            rebuilt.push('#');
            kept = span.end;
        }
        rebuilt.push_str(cell.get(kept..).unwrap_or("?"));
        rebuilt
    }

    /// 当たった所の範囲は置き換える所と同じ（範囲を `#` に替えると `replaced` と同じ値）で、当たるときだけ空でない。
    #[test]
    fn spans_are_the_parts_that_replacing_rewrites() {
        const CELLS: [&str; 8] = [
            "Tokyo",
            "ｶﾞｽ",
            "ＴＯＫ",
            "x-tok-y",
            "",
            "İstanbul",
            "ﾄｰｷｮｰ",
            "a1b22",
        ];
        let standard = FindOptions::standard();
        let all = [
            standard,
            standard.toggled_case(),
            standard.toggled_whole_cell(),
            standard.toggled_width(),
            standard.toggled_kana(),
            standard.toggled_regex(),
        ];
        let texts = ["tok", "ｶﾞ", "o", "i", "ｰ", "zz", r"\d", "."];
        let cases = all.iter().flat_map(|&options| {
            texts
                .iter()
                .flat_map(move |&text| CELLS.iter().map(move |&cell| (options, text, cell)))
        });
        for (options, text, cell) in cases {
            let find = FindQuery::new(text, options);
            let spans = find.spans(cell);
            assert_eq!(!spans.is_empty(), find.matches(cell), "{text:?} {cell:?}");
            assert_eq!(
                find.replaced(cell, "#")
                    .unwrap_or_else(|| String::from(cell)),
                rewritten(cell, &spans),
                "{text:?} {cell:?} {options:?}"
            );
        }
    }

    fn regex(text: &str) -> FindQuery {
        FindQuery::new(text, FindOptions::standard().toggled_regex())
    }

    /// 正規表現も同じ畳みで照らす。`\D` などの書き方は小文字に畳まない（ADR 0031 の決定 3）。
    #[test]
    fn a_regex_matches_folded_cells_and_keeps_its_escapes() {
        assert!(regex(r"tok(yo)?\d").matches("Tokyo2"));
        assert!(!regex(r"tok(yo)?\d").matches("tokyo"));
        assert!(regex(r"tok(yo)?\d").matches("ｔｏｋｙｏ１"));
        assert!(regex(r"^\d+$").matches("１２３"));
        assert!(!regex(r"^\D+$").matches("12"));
        assert!(regex(r"^\D+$").matches("ABC"));
        assert!(regex(r"ガ").matches("ｶﾞｽ"));
    }

    /// 正規表現の大文字小文字・セル全体の条件と、長さ 0 の当たり。
    #[test]
    fn regex_options_and_empty_matches() {
        let case = FindQuery::new(
            "Tok",
            FindOptions::standard().toggled_regex().toggled_case(),
        );
        assert!(case.matches("Tokyo") && !case.matches("tokyo"));
        let whole = FindQuery::new(
            "tok.*",
            FindOptions::standard().toggled_regex().toggled_whole_cell(),
        );
        assert!(whole.matches("TOKYO") && !whole.matches("xtokyo"));
        // 長さ 0 の当たりは当たりにしない。
        assert!(!regex("a*").matches("bbb") && regex("a*").matches("baa"));
    }

    /// 置換の `$1`・`${name}` は当たった組の元の字（全角のまま）で埋める。
    #[test]
    fn a_regex_replacement_fills_groups_with_the_original_characters() {
        assert_eq!(
            regex(r"(\d+)-(\d+)").replaced("１２-３４ 5-6", "$2-$1"),
            Some(String::from("３４-１２ 6-5"))
        );
        assert_eq!(
            regex(r"(?P<y>\d{4})/(?P<m>\d{2})").replaced("2026/11 です", "${m}月${y}年"),
            Some(String::from("11月2026年 です"))
        );
        assert_eq!(regex("a*").replaced("baab", "-"), Some(String::from("b-b")));
        assert_eq!(regex(r"\d").replaced("abc", "-"), None);
    }

    /// 読めない式はどのセルにも当たらず、そう分かる。空の式は読めないのではなく、何も探さない。
    #[test]
    fn a_broken_regex_matches_nothing_and_says_so() {
        let broken = regex("(tok");
        assert!(broken.is_broken());
        assert!(!broken.matches("(tok"));
        assert!(broken.rules_out_ascii(b"(tok"));
        assert_eq!(broken.replaced("(tok", "x"), None);
        assert!(!regex("").is_broken() && regex("").is_empty());
        assert!(!query("(tok").is_broken() && query("(tok").matches("x(tokyo"));
        // 正規表現ではレコードのバイト列で除けない。
        assert!(!regex("t.k").rules_out_ascii(b"1,tak,x"));
    }

    /// 正規表現でも、置き換えられるのは当たるセルだけ。ASCII の速い道と畳んだ写しの道は同じ答え。
    #[test]
    fn a_regex_replacement_agrees_with_matching() {
        let cells = [
            "Tokyo2",
            "ｔｏｋｙｏ１",
            "abc",
            "",
            "１２-３４",
            "ｶﾞｽ",
            "x-tok-y",
            "TOK",
        ];
        let patterns = [
            r"tok\w*",
            r"\d+",
            r"^.$",
            "ガ",
            r"(\d)-(\d)",
            "o|k",
            "[a-c]+",
        ];
        let standard = FindOptions::standard().toggled_regex();
        for options in [
            standard,
            standard.toggled_case(),
            standard.toggled_whole_cell(),
            standard.toggled_width(),
        ] {
            for (text, cell) in patterns
                .iter()
                .flat_map(|&text| cells.iter().map(move |&cell| (text, cell)))
            {
                let find = FindQuery::new(text, options);
                let replaced = find.replaced(cell, "$1#");
                assert_eq!(
                    replaced.is_some(),
                    find.matches(cell),
                    "{text:?} {cell:?} {options:?}"
                );
                let slow =
                    has_long_match(find.regex.as_ref().unwrap(), &fold_for_find(cell, options));
                assert_eq!(find.matches(cell), slow, "{text:?} {cell:?} {options:?}");
            }
        }
    }

    /// 逆戻りで止まる式（`(a*)*b`・`(x+x+)+y`）も、長い入力で終わる（入力の長さに比例する・SEC-04）。
    #[test]
    fn pathological_patterns_finish_on_long_cells() {
        let long = "a".repeat(200_000);
        assert!(!regex("(a*)*b").matches(&long));
        let xs = "x".repeat(200_000);
        assert!(!regex("(x+x+)+y").matches(&xs));
        assert_eq!(regex("(a*)*b").replaced(&long, "-"), None);
    }

    /// 遅い道：両方を畳んで比べる。
    fn folded_match(cell: &str, text: &str, options: FindOptions) -> bool {
        let folded = fold_for_find(cell, options);
        let wanted = fold_for_find(text, options);
        if options.whole_cell() {
            folded == wanted
        } else {
            folded.contains(wanted.as_str())
        }
    }

    /// 速い道（ASCII のセル）と遅い道（畳んだ写し）は同じ答え。
    #[test]
    fn the_ascii_shortcut_agrees_with_folding() {
        let cells = ["Tokyo", "tok", "TOKYO", "x-tok-y", "", "To", "ab", "AB"];
        let queries = ["tok", "TOK", "o", "tokyo", "ab", "B", "zz"];
        let standard = FindOptions::standard();
        for options in [
            standard,
            standard.toggled_case(),
            standard.toggled_whole_cell(),
            standard.toggled_case().toggled_whole_cell(),
        ] {
            for (text, cell) in queries
                .iter()
                .flat_map(|&text| cells.iter().map(move |&cell| (text, cell)))
            {
                let find = FindQuery::new(text, options);
                let slow = folded_match(cell, text, options);
                assert_eq!(find.matches(cell), slow, "{text:?} {cell:?} {options:?}");
            }
        }
    }
}
