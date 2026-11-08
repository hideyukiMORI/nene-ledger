//! 検索の問い（ADR 0030 の決定 2・3）。

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::ops::Range;

use regex_automata::meta::Regex;
use regex_automata::util::interpolate;
use regex_automata::util::primitives::PatternID;
use regex_automata::util::syntax;
use regex_syntax::hir::literal::{ExtractKind, Extractor};

use crate::dialect::Dialect;
use crate::find_folding::{fold_for_find, fold_with_origins};
use crate::find_options::FindOptions;
use crate::quoting::Quoting;
use crate::text_units::TextUnits;

/// 探す文字列と条件。セルの値（見えている値の文字列）が当たるかを決める。値は文字列のまま比べる（数や日付として
/// 比べない・KEEP-01）。正規表現の条件なら式を組み立てて持つ（ADR 0031）。読めない式はどのセルにも当たらない。
/// 等しさは打った文字列と条件で決める（式も先探しの字の並びもそこから作るので）。
#[derive(Debug, Clone)]
pub struct FindQuery {
    text: String,
    folded: String,
    options: FindOptions,
    regex: Option<Regex>,
    /// ASCII のセルに当たるなら、そのセルに必ずどれかがある ASCII の字の並び（空でない）。大文字小文字の条件が偽なら
    /// ASCII の大文字小文字を区別せずに比べる。正規表現でない問いは畳んだ問いそのもの、正規表現は式から取り出した
    /// 当たりの頭の字の並び（`prefilter`・GitHub Issue #223）。言えなければ `None`。
    needles: Option<Vec<Vec<u8>>>,
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
        let folded = fold_for_find(text, options);
        let needles = if options.regex() {
            prefilter(text, options)
        } else {
            (!folded.is_empty() && folded.is_ascii())
                .then(|| alloc::vec![folded.as_bytes().to_vec()])
        };
        Self {
            text: String::from(text),
            folded,
            options,
            regex: compiled(text, options),
            needles,
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
    /// 小文字になるだけで、式は大文字小文字の設定で同じ答えになる）。先探しの字の並びのどれも無い ASCII のセルには式を
    /// 当てない（GitHub Issue #223）。
    fn regex_matches(&self, cell: &str) -> bool {
        let Some(regex) = &self.regex else {
            return false;
        };
        if cell.is_ascii() {
            if let Some(needles) = &self.needles
                && !contains_any(cell.as_bytes(), needles, self.options.match_case())
            {
                return false;
            }
            return has_long_match(regex, cell);
        }
        has_long_match(regex, &fold_for_find(cell, self.options))
    }

    /// 方言 `dialect` で書いた、ASCII だけの原本のレコードのバイト列 `raw`（区切りと引用を含む）のどのセルにも当たらないと
    /// 言えるか。復号せずに照らす速い道（ADR 0030 の決定 4）。ASCII のセルを畳んでも ASCII のままなので、問いが ASCII で
    /// なければ当たらない。ASCII の問いは、セルに当たれば、そのセルの値の字の並びがレコードに残っているかを方言で確かめる
    /// （`record_lacks`・GitHub Issue #226）。正規表現は、式から取り出した先探しの字の並びで同じことを確かめる（取り出せない
    /// 式は除かない・GitHub Issue #223）。
    #[must_use]
    pub fn rules_out_ascii(&self, raw: &[u8], dialect: &Dialect) -> bool {
        // 正規表現：読めない式はどこにも当たらない。正規表現でない問い：空の問いと ASCII でない問いは ASCII のセルに当たらない。
        if self.options.regex() {
            if self.regex.is_none() {
                return true;
            }
        } else if self.folded.is_empty() || !self.folded.is_ascii() {
            return true;
        }
        self.needles
            .as_ref()
            .is_some_and(|needles| record_lacks(raw, *dialect, needles, self.options.match_case()))
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
    let pattern = pattern(text, options)?;
    Regex::builder()
        .syntax(syntax::Config::new().case_insensitive(!options.match_case()))
        .build(&pattern)
        .ok()
}

/// 正規表現の条件で、打った `text` から組み立てる式の文字列（空の問いと正規表現でない条件は `None`）。
fn pattern(text: &str, options: FindOptions) -> Option<String> {
    if !options.regex() || text.is_empty() {
        return None;
    }
    let folded = fold_for_find(text, options.matching_case());
    Some(if options.whole_cell() {
        format!("\\A(?:{folded})\\z")
    } else {
        folded
    })
}

/// 先探しの字の並びの数の上限。超えると取り出しは字の並びを短くして数を減らす（`Extractor::limit_total`）。
const PREFILTER_LIMIT: usize = 16;

/// 正規表現の条件で、ASCII のセルの長さのある当たりが必ずどれかで始まる、ASCII の字の並び（空でない・GitHub Issue #223・
/// benchmarks 第 11 節の案 C）。取り出せなければ `None`（その式は全部のセルに当てる）。
///
/// 式は大文字小文字を**区別して**読み、当たりの頭の字の並びの有限の集合を取り出す。大文字小文字を区別しない条件では、
/// ASCII の大文字小文字を区別せずに比べる。区別しない式の字の組は、区別して読んだ字の組を大文字小文字で畳んで広げたもの
/// なので、ASCII のセルの字がそれに入るなら、区別して読んだ組に ASCII の大文字小文字を除いて同じ字があるか、組に ASCII で
/// ない字（ケルビンの記号 K・長い s の ſ は畳むと `k`・`s` に当たる）がある。ASCII でない字の並びが 1 つでもあれば取り出さ
/// ない。否定の組（`[^k]`）は、区別しないほうが狭い。
/// - 空の字の並び（長さ 0 で当たりうる・`a*`・`x|`）と、有限でない集合（頭が `\d`・`.` などの大きな組）は取り出さない
/// - `^`・`$`・`\b` は長さ 0 なので、当たりの字の並びを変えない
/// - ASCII でないセルには使わない（畳んだ写しにケルビンの記号などが残りうる）
fn prefilter(text: &str, options: FindOptions) -> Option<Vec<Vec<u8>>> {
    let pattern = pattern(text, options)?;
    let hir = syntax::parse_with(&pattern, &syntax::Config::new()).ok()?;
    let mut extractor = Extractor::new();
    extractor
        .kind(ExtractKind::Prefix)
        .limit_total(PREFILTER_LIMIT);
    let found = extractor.extract(&hir);
    let literals = found.literals()?;
    if literals.is_empty() {
        return None;
    }
    literals
        .iter()
        .map(|literal| {
            let bytes = literal.as_bytes();
            (!bytes.is_empty() && bytes.is_ascii()).then(|| bytes.to_vec())
        })
        .collect()
}

/// 方言 `dialect` で書いた ASCII だけのレコードのバイト列 `raw` の、どのセルの値にも ASCII の字の並び `wanted`（どれも空で
/// ない）のどれも無いと言えるか（`match_case` が偽なら ASCII の大文字小文字を区別しない）。GitHub Issue #226。
///
/// 言えるのは、セルの値が「バイト列から引用符をいくつか抜いた字の並び」になるときだけ。
/// - バックスラッシュで逃がす流儀で `\` があれば、逃がした字（`\,`・`\t`）が値では別の字になるので言えない
/// - 引用符を使う流儀でバイト列に引用符があれば、引用符を含む問いは言えない（`""` は値で 1 つになる）。引用符を含まない
///   問いは、バイト列から引用符を**すべて**抜いた並びで探す（閉じた引用符の後ろに字が続くと、値では前の字とつながる）。
///   セルの値に問いがあれば、問いに引用符が無いので、引用符を抜いた並びにも同じ字の並びがある
/// - 文字の単位が 2 バイトの文字コード（UTF-16）は ASCII のバイトが字を表さないので言えない（呼び手も除いている）
fn record_lacks(raw: &[u8], dialect: Dialect, wanted: &[Vec<u8>], match_case: bool) -> bool {
    match dialect.units() {
        TextUnits::SingleByte | TextUnits::ShiftJis => {}
        TextUnits::Utf16Le | TextUnits::Utf16Be => return false,
    }
    let (escapes, quotes) = match dialect.quoting() {
        Quoting::Doubled => (false, true),
        Quoting::QuotedBackslash => (true, true),
        Quoting::Backslash => (true, false),
        Quoting::Plain => (false, false),
    };
    if escapes && raw.contains(&b'\\') {
        return false;
    }
    let quote = dialect.quote();
    if !quotes || !raw.contains(&quote) {
        return !contains_any(raw, wanted, match_case);
    }
    if wanted.iter().any(|part| part.contains(&quote)) {
        return false;
    }
    let unquoted: Vec<u8> = raw.iter().copied().filter(|&byte| byte != quote).collect();
    !contains_any(&unquoted, wanted, match_case)
}

/// バイト列 `haystack` に字の並び `wanted` のどれかがあるか。
fn contains_any(haystack: &[u8], wanted: &[Vec<u8>], match_case: bool) -> bool {
    wanted
        .iter()
        .any(|part| contains_ascii(haystack, part, match_case))
}

/// バイト列 `haystack` に字の並び `wanted` があるか（`match_case` が偽なら ASCII の大文字小文字を区別しない）。空の並びは
/// どこにでもある。
fn contains_ascii(haystack: &[u8], wanted: &[u8], match_case: bool) -> bool {
    if wanted.is_empty() {
        return true;
    }
    haystack.windows(wanted.len()).any(|part| {
        if match_case {
            part == wanted
        } else {
            part.eq_ignore_ascii_case(wanted)
        }
    })
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
    use crate::delimiter::Delimiter;

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
        let csv = Dialect::rfc4180();
        let find = query("tokyo");
        assert!(!find.rules_out_ascii(b"1,TOKYO,x", &csv));
        assert!(!find.rules_out_ascii(b"1,\"tokyo, jp\",x", &csv));
        assert!(find.rules_out_ascii(b"1,osaka,x", &csv));
        assert!(query("東京").rules_out_ascii(b"1,tokyo", &csv));
        assert!(!query("a\"b").rules_out_ascii(b"1,\"a\"\"b\"", &csv));
        let case = FindQuery::new("Tok", FindOptions::standard().toggled_case());
        assert!(case.rules_out_ascii(b"tokyo", &csv) && !case.rules_out_ascii(b"Tokyo", &csv));
    }

    /// セルの値がバイト列と違う字の並びになる方言では、その字の並びの問いでレコードを除かない（GitHub Issue #226）。
    #[test]
    fn a_record_is_not_ruled_out_where_the_dialect_rewrites_the_cell() {
        let escaped = Dialect::new(Delimiter::COMMA, Quoting::Backslash, b'"').unwrap();
        let single = Dialect::new(Delimiter::COMMA, Quoting::Doubled, b'\'').unwrap();
        let csv = Dialect::rfc4180();
        assert!(!query("a,b").rules_out_ascii(b"x,a\\,b,y", &escaped));
        assert!(!query("a'b").rules_out_ascii(b"x,'a''b'", &single));
        assert!(!query("bc").rules_out_ascii(b"x,\"ab\"cd", &csv));
        // 引用符の無いレコードと、逃がす字の無いレコードは、今までどおり除ける。
        assert!(query("bc").rules_out_ascii(b"x,ab,cd", &csv));
        assert!(query("ab").rules_out_ascii(b"x,a,b", &escaped));
        assert!(query("zz").rules_out_ascii(b"x,\"ab\"cd", &csv));
    }

    /// 種を固定した xorshift（テストは乱数の名前に頼らない・ARC-007）。
    struct Xorshift(u64);

    impl Xorshift {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13_u32;
            self.0 ^= self.0 >> 7_u32;
            self.0 ^= self.0 << 17_u32;
            self.0
        }

        fn below(&mut self, bound: usize) -> usize {
            usize::try_from(self.next() % u64::try_from(bound).unwrap()).unwrap()
        }

        fn pick<'a>(&mut self, choices: &[&'a str]) -> &'a str {
            choices[self.below(choices.len())]
        }
    }

    /// 無作為の字の並び（`length` 個まで）。
    fn random_text(rng: &mut Xorshift, alphabet: &[&str], length: usize) -> String {
        let count = rng.below(length.saturating_add(1));
        (0..count).map(|_| rng.pick(alphabet)).collect()
    }

    /// レコードのバイト列 `raw` を方言 `dialect` で切ったセルの値（製品と同じ道：`sample_records` で切り、`value` で値に）。
    fn cells_of(raw: &[u8], dialect: Dialect) -> Vec<String> {
        let mut cells = Vec::new();
        for record in crate::detecting::sample_records(raw, dialect, true) {
            for &field in record.fields() {
                cells.push(String::from_utf8(crate::value::value(raw, field, &dialect)).unwrap());
            }
        }
        cells
    }

    /// どの流儀 × 引用符 2 つ × 閉じない引用符の読み方 2 つの方言。
    fn every_dialect() -> Vec<Dialect> {
        use crate::unclosed_quote::UnclosedQuote;
        let mut dialects = Vec::new();
        for quoting in [
            Quoting::Doubled,
            Quoting::QuotedBackslash,
            Quoting::Backslash,
            Quoting::Plain,
        ] {
            for quote in *b"\"'" {
                let dialect = Dialect::new(Delimiter::COMMA, quoting, quote).unwrap();
                dialects.push(dialect);
                dialects.push(dialect.with_unclosed_quote(UnclosedQuote::Literal));
            }
        }
        dialects
    }

    /// 無作為の ASCII のレコード（`RECORD` の字から）× 方言 × 問い（`texts` の字から・条件は `options` から）を `rounds` 回
    /// 試し、除いたレコードのどのセルにも問いが当たらないことを確かめる。除いた回数を返す。
    fn ruled_out_records(
        rng: &mut Xorshift,
        texts: &[&str],
        options: &[FindOptions],
        rounds: u32,
    ) -> usize {
        const RECORD: [&str; 12] = [
            "a", "b", "A", ",", "\"", "'", "\\", "t", "\t", " ", "k", "1",
        ];
        let dialects = every_dialect();
        let mut ruled_out = 0_usize;
        for _ in 0..rounds {
            let raw = random_text(rng, &RECORD, 12);
            let text = random_text(rng, texts, 4);
            let dialect = dialects[rng.below(dialects.len())];
            let find = FindQuery::new(&text, options[rng.below(options.len())]);
            if !find.rules_out_ascii(raw.as_bytes(), &dialect) {
                continue;
            }
            ruled_out = ruled_out.saturating_add(1);
            for cell in cells_of(raw.as_bytes(), dialect) {
                assert!(
                    !find.matches(&cell),
                    "{raw:?} {dialect:?} {text:?} {:?} → {cell:?}",
                    find.options()
                );
            }
        }
        ruled_out
    }

    /// 無作為の ASCII のレコード × 方言 × 問いで、除いたレコードのどのセルにも問いが当たらない（GitHub Issue #226）。
    /// セルは製品と同じ道（`sample_records` で切り、`value` で値にする）で取り出す。
    #[test]
    fn a_ruled_out_record_has_no_matching_cell() {
        const TEXT: [&str; 8] = ["a", "b", "B", ",", "\"", "'", "\\", "\t"];
        let standard = FindOptions::standard();
        let options = [
            standard,
            standard.toggled_case(),
            standard.toggled_whole_cell(),
        ];
        let ruled_out = ruled_out_records(&mut Xorshift(0x2026_1108_0226), &TEXT, &options, 20_000);
        // 除ける場合を十分に試している（どれも除かない実装でも通ってしまわないように）。
        assert!(ruled_out > 5_000, "{ruled_out}");
    }

    /// 式の字の素（読めない並びも混ぜる）。ケルビンの記号 K（U+212A）と長い s（ſ）は、大文字小文字を区別しない式で
    /// ASCII の `k`・`s` に当たる。
    const PATTERN_PIECES: [&str; 30] = [
        "a", "b", "B", "k", "K", "s", "\u{212A}", "ſ", "\"", "'", ",", "1", "[ab]", "[^a]", r"\d",
        ".", "x*", "(a|bc)", "a?", "^", "$", r"\b", "(?i)", "(?-i)", "|", "+", "(", ")", "é", "ｂ",
    ];

    /// 正規表現の条件の組み合わせ。
    fn regex_options() -> [FindOptions; 4] {
        let regex = FindOptions::standard().toggled_regex();
        [
            regex,
            regex.toggled_case(),
            regex.toggled_whole_cell(),
            regex.toggled_case().toggled_whole_cell(),
        ]
    }

    /// 正規表現でも、除いたレコードのどのセルにも式が当たらない（先探し・GitHub Issue #223）。
    #[test]
    fn a_ruled_out_record_has_no_cell_matching_the_regex() {
        let ruled_out = ruled_out_records(
            &mut Xorshift(0x2026_1108_0223),
            &PATTERN_PIECES,
            &regex_options(),
            30_000,
        );
        assert!(ruled_out > 5_000, "{ruled_out}");
    }

    /// 無作為の式 × セルで、先探しをした照合と、先探しをせずに畳んだ写しに式を当てた答えが同じ（GitHub Issue #223）。
    #[test]
    fn the_prefilter_agrees_with_matching_every_cell() {
        const CELL: [&str; 18] = [
            "a", "A", "b", "B", "k", "K", "s", "S", "\u{212A}", "ſ", "x", "1", "é", "ｂ", " ",
            "\"", "bc", "１",
        ];
        let mut rng = Xorshift(0x2026_1108_2231);
        let options = regex_options();
        let mut skipped = 0_usize;
        for _ in 0..30_000_u32 {
            let text = random_text(&mut rng, &PATTERN_PIECES, 4);
            let cell = random_text(&mut rng, &CELL, 8);
            let find = FindQuery::new(&text, options[rng.below(options.len())]);
            let Some(compiled) = &find.regex else {
                assert!(!find.matches(&cell));
                continue;
            };
            let slow = has_long_match(compiled, &fold_for_find(&cell, find.options()));
            assert_eq!(
                find.matches(&cell),
                slow,
                "{text:?} {cell:?} {:?} {:?}",
                find.options(),
                find.needles
            );
            let ascii = cell.is_ascii();
            if let Some(needles) = find.needles.as_ref().filter(|_| ascii)
                && !contains_any(cell.as_bytes(), needles, find.options().match_case())
            {
                skipped = skipped.saturating_add(1);
            }
        }
        // 先探しで式を当てずに済んだセルが十分にある。
        assert!(skipped > 3_000, "{skipped}");
    }

    /// 取り出す字の並び：当たりの頭の字の並び。長さ 0 で当たりうる式・頭が大きな組の式・ASCII でない字の並びは取り出さない。
    #[test]
    fn the_prefilter_takes_the_heads_of_matches() {
        let standard = FindOptions::standard().toggled_regex();
        let case = standard.toggled_case();
        let cases: [(&str, FindOptions, Option<&[&str]>); 14] = [
            (r"Name12(\d)", standard, Some(&["Name12"])),
            (r"R1222485\d$", standard, Some(&["R1222485"])),
            ("zz+notthere", standard, Some(&["zz"])),
            ("ab|cd", standard, Some(&["ab", "cd"])),
            // 全角の式は畳んでから読む。
            ("ｔｏｋ", standard, Some(&["tok"])),
            // 大文字小文字を区別する条件で `(?i)` と書けば、大文字小文字の組み合わせを取り出す。`k` はケルビンの記号を
            // 含むので取り出さない。
            ("(?i)ab", case, Some(&["AB", "Ab", "aB", "ab"])),
            ("(?i)ak", case, None),
            (r"\d{4}-\d\d", standard, None),
            ("a*", standard, None),
            ("x|", standard, None),
            ("東京", standard, None),
            ("\u{212A}", standard, None),
            ("(tok", standard, None),
            ("", standard, None),
        ];
        for (text, options, expected) in cases {
            let mut found: Option<Vec<String>> = FindQuery::new(text, options).needles.map(|all| {
                all.into_iter()
                    .map(|part| String::from_utf8(part).unwrap())
                    .collect()
            });
            if let Some(parts) = found.as_mut() {
                parts.sort();
            }
            let expected: Option<Vec<String>> =
                expected.map(|parts| parts.iter().map(|&part| String::from(part)).collect());
            assert_eq!(found, expected, "{text:?} {options:?}");
        }
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
        let csv = Dialect::rfc4180();
        let broken = regex("(tok");
        assert!(broken.is_broken());
        assert!(!broken.matches("(tok"));
        assert!(broken.rules_out_ascii(b"(tok", &csv));
        assert_eq!(broken.replaced("(tok", "x"), None);
        assert!(!regex("").is_broken() && regex("").is_empty());
        assert!(!query("(tok").is_broken() && query("(tok").matches("x(tokyo"));
        // 正規表現ではレコードのバイト列で除けない。
        assert!(!regex("t.k").rules_out_ascii(b"1,tak,x", &csv));
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
