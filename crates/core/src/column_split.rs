//! 列の値を分ける規則と、つなぐ規則（列の分割と結合・ADR 0037・EDIT-08）。
//!
//! セルは文字列のまま扱う（KEEP-01）。分けた値の前後の空白も落とさない。

use alloc::string::String;
use alloc::vec::Vec;

use crate::find_options::FindOptions;
use crate::find_query::FindQuery;
use crate::split_kind::SplitKind;

/// 1 つの値を分けてできるセルの数の上限（足す列の数の上限・ADR 0037 の決定 2）。超えた分は最後のセルに残す。
pub const MAX_PARTS: usize = 64;

/// 列の値の分け方。打った文字列 `text` を、種類 `kind` で読んだもの。
#[derive(Debug, Clone)]
pub struct Splitter {
    kind: SplitKind,
    text: String,
    pattern: Option<FindQuery>,
    widths: Vec<usize>,
}

impl Splitter {
    /// 打った文字列 `text` を種類 `kind` の分け方として読む。空の文字列・切れない正規表現・読めない幅（数字とカンマの
    /// ほかの字・0）なら `None`。
    #[must_use]
    pub fn parse(kind: SplitKind, text: &str) -> Option<Self> {
        let (pattern, widths) = match kind {
            SplitKind::Literal if text.is_empty() => return None,
            SplitKind::Literal => (None, Vec::new()),
            SplitKind::Pattern => {
                // 式は打ったとおりに読む（大文字小文字と全角半角を区別する）。
                let options = FindOptions::standard()
                    .toggled_regex()
                    .toggled_case()
                    .toggled_width();
                let query = FindQuery::new(text, options);
                if query.is_empty() || query.is_broken() {
                    return None;
                }
                (Some(query), Vec::new())
            }
            SplitKind::Widths => (None, parsed_widths(text)?),
        };
        Some(Self {
            kind,
            text: String::from(text),
            pattern,
            widths,
        })
    }

    /// 値 `value` を分けたセル（左から順）。区切りが 1 つも無ければ、値そのものの 1 つ。[`MAX_PARTS`] を超える分は
    /// 最後のセルに残す。
    #[must_use]
    pub fn split(&self, value: &str) -> Vec<String> {
        let cuts: Vec<(usize, usize)> = match self.kind {
            SplitKind::Literal => value
                .match_indices(self.text.as_str())
                .map(|(start, found)| (start, start.saturating_add(found.len())))
                .collect(),
            SplitKind::Pattern => self
                .pattern
                .as_ref()
                .map(|pattern| pattern.spans(value))
                .unwrap_or_default()
                .into_iter()
                .map(|span| (span.start, span.end))
                .collect(),
            SplitKind::Widths => width_cuts(value, &self.widths),
        };
        let mut parts = Vec::with_capacity(cuts.len().saturating_add(1));
        let mut from = 0_usize;
        for (start, end) in cuts.into_iter().take(MAX_PARTS.saturating_sub(1)) {
            parts.push(String::from(value.get(from..start).unwrap_or_default()));
            from = end;
        }
        parts.push(String::from(value.get(from..).unwrap_or_default()));
        parts
    }
}

/// 幅の並び `3,5,2`（半角か全角の数字とカンマ・空白は読み飛ばす）。1 つも無い・0 がある・ほかの字があれば `None`。
fn parsed_widths(text: &str) -> Option<Vec<usize>> {
    let widths: Option<Vec<usize>> = text
        .split([',', '，', '、'])
        .map(|part| {
            let digits: String = part
                .trim()
                .chars()
                .map(|character| match character {
                    '０'..='９' => char::from_u32(u32::from(character).saturating_sub(0xFEE0))
                        .unwrap_or(character),
                    other => other,
                })
                .collect();
            digits.parse::<usize>().ok().filter(|&width| width > 0)
        })
        .collect();
    widths.filter(|widths| !widths.is_empty())
}

/// 値 `value` を、字の数 `widths` で切る位置（長さ 0 の切れ目の並び）。値が短ければ、そこまで。
fn width_cuts(value: &str, widths: &[usize]) -> Vec<(usize, usize)> {
    let starts: Vec<usize> = value.char_indices().map(|(at, _)| at).collect();
    let mut cuts = Vec::with_capacity(widths.len());
    let mut position = 0_usize;
    for &width in widths {
        position = position.saturating_add(width);
        match starts.get(position) {
            Some(&at) => cuts.push((at, at)),
            None => break,
        }
    }
    cuts
}

/// セルの値 `cells`（左から順。「セルが無い」所は `None`）を、区切り `separator` でつないだ値（ADR 0037 の決定 4）。
/// 「セルが無い」所は飛ばす（区切りも入れない）。空のセルは空の値としてつなぐ。
#[must_use]
pub fn joined<'cell, I: IntoIterator<Item = Option<&'cell str>>>(
    cells: I,
    separator: &str,
) -> String {
    let mut out = String::new();
    for (index, cell) in cells.into_iter().flatten().enumerate() {
        if index > 0 {
            out.push_str(separator);
        }
        out.push_str(cell);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn split(kind: SplitKind, text: &str, value: &str) -> Vec<String> {
        Splitter::parse(kind, text).unwrap().split(value)
    }

    #[test]
    fn a_literal_separator_splits_at_every_occurrence_and_keeps_spaces() {
        assert_eq!(
            split(SplitKind::Literal, "-", "東京都-千代田区- 1"),
            ["東京都", "千代田区", " 1"]
        );
        assert_eq!(
            split(SplitKind::Literal, " ", "山田 太郎"),
            ["山田", "太郎"]
        );
        assert_eq!(
            split(SplitKind::Literal, "/", "2026/11/16"),
            ["2026", "11", "16"]
        );
        // 区切りが無ければ値そのもの。端の区切りは空のセルになる。正規表現の字も文字どおり。
        assert_eq!(split(SplitKind::Literal, ",", "abc"), ["abc"]);
        assert_eq!(split(SplitKind::Literal, ",", ",a,"), ["", "a", ""]);
        assert_eq!(split(SplitKind::Literal, ".", "a.b"), ["a", "b"]);
    }

    #[test]
    fn an_empty_or_broken_way_is_refused() {
        assert!(Splitter::parse(SplitKind::Literal, "").is_none());
        assert!(Splitter::parse(SplitKind::Pattern, "").is_none());
        assert!(Splitter::parse(SplitKind::Pattern, "(").is_none());
        for widths in ["", "a", "3,0", "3,,2", "-1"] {
            assert!(
                Splitter::parse(SplitKind::Widths, widths).is_none(),
                "{widths}"
            );
        }
    }

    #[test]
    fn a_pattern_splits_at_its_matches_as_typed() {
        assert_eq!(
            split(SplitKind::Pattern, "[-/年月]", "2026年11月16"),
            ["2026", "11", "16"]
        );
        assert_eq!(
            split(SplitKind::Pattern, r"\s+", "a   b\tc"),
            ["a", "b", "c"]
        );
        // 打ったとおりに読む（大文字小文字と全角半角を畳まない）。
        assert_eq!(split(SplitKind::Pattern, "x", "aXbxc"), ["aXb", "c"]);
        assert_eq!(split(SplitKind::Pattern, "-", "a－b-c"), ["a－b", "c"]);
    }

    #[test]
    fn widths_count_characters_and_leave_the_rest_in_the_last_cell() {
        assert_eq!(
            split(SplitKind::Widths, "3,5,2", "ABC12345XYrest"),
            ["ABC", "12345", "XY", "rest"]
        );
        assert_eq!(
            split(SplitKind::Widths, "２、1", "東京都千代田区"),
            ["東京", "都", "千代田区"]
        );
        // 値が短ければ、そこまで。
        assert_eq!(split(SplitKind::Widths, "3,5", "AB"), ["AB"]);
        assert_eq!(split(SplitKind::Widths, "3,5", "ABCDE"), ["ABC", "DE"]);
        assert_eq!(split(SplitKind::Widths, "3", "ABC"), ["ABC"]);
    }

    #[test]
    fn more_parts_than_the_limit_stay_in_the_last_cell() {
        let value = vec!["x"; 100].join(",");
        let parts = split(SplitKind::Literal, ",", &value);
        assert_eq!(parts.len(), MAX_PARTS);
        assert_eq!(parts.last().unwrap(), &vec!["x"; 37].join(","));
    }

    #[test]
    fn joining_skips_missing_cells_and_keeps_empty_ones() {
        assert_eq!(joined([Some("山田"), Some("太郎")], " "), "山田 太郎");
        assert_eq!(joined([Some("a"), None, Some("c")], "-"), "a-c");
        assert_eq!(joined([Some("a"), Some(""), Some("c")], "-"), "a--c");
        assert_eq!(joined([Some("a"), Some("b")], ""), "ab");
        assert_eq!(joined([None, None], "-"), "");
    }

    /// 種を固定した xorshift（テストは乱数の名前に頼らない・ARC-007）。
    struct Xorshift(u64);

    impl Xorshift {
        fn below(&mut self, bound: usize) -> usize {
            self.0 ^= self.0 << 13_u32;
            self.0 ^= self.0 >> 7_u32;
            self.0 ^= self.0 << 17_u32;
            usize::try_from(self.0 % u64::try_from(bound).unwrap()).unwrap()
        }
    }

    /// 文字どおりの区切りで分けた値を、同じ区切りでつなぐと元の値に戻る（乱数の値・ADR 0037 の強制の 1 つ目）。
    #[test]
    fn splitting_then_joining_with_the_same_separator_gives_the_value_back() {
        const PARTS: [&str; 8] = ["a", "-", "東京", " ", "--", "", "1,2", "／"];
        let mut rng = Xorshift(0x9E37_79B9_7F4A_7C15);
        for _ in 0..3_000_usize {
            let value: String = (0..rng.below(12)).map(|_| PARTS[rng.below(8)]).collect();
            let separator = ["-", " ", "--", "／", ","][rng.below(5)];
            let parts = split(SplitKind::Literal, separator, &value);
            assert!(parts.len() <= MAX_PARTS);
            let back = joined(parts.iter().map(|part| Some(part.as_str())), separator);
            assert_eq!(back, value, "{separator:?}");
        }
    }
}
