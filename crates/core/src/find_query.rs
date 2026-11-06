//! 検索の問い（ADR 0030 の決定 2・3）。

use alloc::string::String;

use crate::find_folding::{fold_for_find, fold_with_origins};
use crate::find_options::FindOptions;

/// 探す文字列と条件。セルの値（見えている値の文字列）が当たるかを決める。値は文字列のまま比べる（数や日付として
/// 比べない・KEEP-01）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindQuery {
    text: String,
    folded: String,
    options: FindOptions,
}

impl FindQuery {
    /// `text` を条件 `options` で探す問い。
    #[must_use]
    pub fn new(text: &str, options: FindOptions) -> Self {
        Self {
            text: String::from(text),
            folded: fold_for_find(text, options),
            options,
        }
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
        if self.options.whole_cell() {
            return Some(String::from(replacement));
        }
        let (folded, origins) = fold_with_origins(cell, self.options);
        // 写しのバイトの位置 `at` を含む字の、元の範囲。
        let origin = |at: usize| {
            let index = origins.partition_point(|(start, _)| *start <= at);
            origins
                .get(index.saturating_sub(1))
                .map(|(_, range)| range.clone())
        };
        let mut result = String::with_capacity(cell.len());
        let mut kept = 0_usize;
        for (at, part) in folded.match_indices(self.folded.as_str()) {
            let last = at.saturating_add(part.len()).saturating_sub(1);
            let (Some(first), Some(end)) = (origin(at), origin(last)) else {
                continue;
            };
            // 小文字にして 1 字が数字に増えた字の途中で切れても、元の字を丸ごと置き換える。前と重なれば飛ばす。
            if first.start < kept {
                continue;
            }
            result.push_str(cell.get(kept..first.start)?);
            result.push_str(replacement);
            kept = end.end;
        }
        result.push_str(cell.get(kept..)?);
        Some(result)
    }

    /// ASCII だけの原本のレコードのバイト列 `raw`（区切りと引用を含む）のどのセルにも当たらないと言えるか。復号せずに
    /// 照らす速い道（ADR 0030 の決定 4）。ASCII のセルを畳んでも ASCII のままなので、問いが ASCII でなければ当たらない。
    /// ASCII の問いは、セルに当たればレコードのバイト列にも同じ字の並びがある（引用の中の `"` は二重になるので、`"` を
    /// 含む問いでは言えない）。
    #[must_use]
    pub fn rules_out_ascii(&self, raw: &[u8]) -> bool {
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
