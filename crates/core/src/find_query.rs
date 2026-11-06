//! 検索の問い（ADR 0030 の決定 2・3）。

use alloc::string::String;

use crate::find_folding::fold_for_find;
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
