//! 方言の判別の結果。

use alloc::vec::Vec;

use crate::detecting::{
    SAMPLE_LIMIT, candidate_dialects, header_guess, preamble_rows, sample_records, score,
    segmentation,
};
use crate::dialect::Dialect;
use crate::dialect_candidate::DialectCandidate;
use crate::header_guess::HeaderGuess;
use crate::line_ending_counts::LineEndingCounts;
use crate::quoting::Quoting;
use crate::text_units::TextUnits;

/// 確信ありとする最低の点数（千分率）。🔴 300 では、空のセル・`-`・段の見出しの多い統計の表（e-Stat）で選んだ方言が
/// 正しくても確信なしになった（実データ 392 で 78。200 で 33・誤りは増えない・GitHub Issue #24）。
const CONFIDENT_SCORE: u32 = 200;

/// 確信ありとする、2 番目との最低の差（千分率）。
const CONFIDENT_MARGIN: u32 = 100;

/// 方言の判別の結果（OPEN-01 / 02 / 05 / 06 / 08 / 11）。値は変えない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Detection {
    candidates: Vec<DialectCandidate>,
    confident: bool,
    line_endings: LineEndingCounts,
    header: HeaderGuess,
    preamble: usize,
}

impl Detection {
    /// 入力の先頭（最大 [`SAMPLE_LIMIT`] バイト）から方言を推し量る。文字はどれも 1 バイトとみなす。
    #[must_use]
    pub fn of(input: &[u8]) -> Self {
        Self::of_encoded(input, TextUnits::SingleByte)
    }

    /// 入力の先頭から、推し量った文字コードの文字の単位 `units` で候補を試して方言を推し量る（ADR 0019）。
    /// 候補の方言は `units` を持つ。
    #[must_use]
    pub fn of_encoded(input: &[u8], units: TextUnits) -> Self {
        let complete = input.len() <= SAMPLE_LIMIT;
        let sample = input.get(..SAMPLE_LIMIT).unwrap_or(input);
        let mut seen: Vec<Vec<Vec<Vec<u8>>>> = Vec::new();
        let mut candidates: Vec<DialectCandidate> = Vec::new();
        for dialect in candidate_dialects()
            .into_iter()
            .map(|dialect| dialect.with_units(units))
        {
            let cut = segmentation(sample, dialect, complete);
            if !seen.contains(&cut) {
                seen.push(cut);
                candidates.push(score(sample, dialect, complete));
            }
        }
        candidates.sort_by_key(|candidate| core::cmp::Reverse(candidate.score()));
        let best = candidates.first().map_or_else(
            || Dialect::rfc4180().with_units(units),
            |candidate| candidate.dialect(),
        );
        // 前書き・見出し・改行は、ファイルを実際に読む方言（`best`・厳格）で切ったレコードで数える。🔴 点を付ける試し切り
        // （`trial_dialect`・閉じない引用符を文字として読むことがある）で数えると、レコードの番号が実際の読みとずれ、
        // 迷子の引用符の後ろで前書きの数が表のレコードの数を超えて、表が空になる（GitHub Issue #12）。
        let records = sample_records(sample, best, complete);
        let preamble = preamble_rows(sample, &records, &best);
        let table = records.get(preamble..).unwrap_or_default();
        Self {
            confident: Self::is_confident(&candidates),
            line_endings: LineEndingCounts::tally(
                records.iter().map(crate::record::Record::terminator),
            ),
            header: header_guess(sample, table, &best),
            preamble,
            candidates,
        }
    }

    /// 1 位が十分に高く、競う相手（`rival`）と十分に離れているか。
    fn is_confident(candidates: &[DialectCandidate]) -> bool {
        let top = candidates.first().map_or(0, |c| c.score());
        let second = rival_of(candidates).map_or(0, DialectCandidate::score);
        top >= CONFIDENT_SCORE && top.saturating_sub(second) >= CONFIDENT_MARGIN
    }

    /// 1 位と競う候補: 1 位と区切りか引用符の字が違う候補のうち、いちばん点の高いもの（無ければ `None`）。
    ///
    /// 🔴 区切りと引用符が同じで、引用の中の引用符の逃がし方（`""` か `\"` か）だけが違う候補は競わない。表の形（列と行）は
    /// ほぼ同じで、違うのは逃がした引用符を含む少数のセルだけ。Pollock の確信なし 883 のうち 880 がこの組で、
    /// 表を正しく読めていても知らせが出ていた（GitHub Issue #24）。逃がし方は候補の一覧（パレット）で選び直せる。
    #[must_use]
    pub fn rival(&self) -> Option<DialectCandidate> {
        rival_of(&self.candidates)
    }

    /// いちばんもっともらしい方言。候補が全部 0 点なら RFC 4180 の CSV（確信なし）。
    #[must_use]
    pub fn best(&self) -> Dialect {
        self.candidates
            .first()
            .filter(|candidate| candidate.score() > 0)
            .map_or_else(Dialect::rfc4180, |candidate| candidate.dialect())
    }

    /// 候補（点数の高い順・同点は優先の順）。そのファイルで同じ切り方になる候補は、優先の高い 1 つにまとめてある。
    /// 利用者が選び直すために示す（OPEN-02）。
    #[must_use]
    pub fn candidates(&self) -> &[DialectCandidate] {
        &self.candidates
    }

    /// 確信があるか。無ければ黙って決めず、候補を示して選ばせる（FR-023）。
    #[must_use]
    pub fn confident(&self) -> bool {
        self.confident
    }

    /// 改行の内訳（いちばんもっともらしい方言で切ったとき）。
    #[must_use]
    pub fn line_endings(&self) -> LineEndingCounts {
        self.line_endings
    }

    /// 見出しの行の有無の推定（前書きの後ろの最初の行について）。
    #[must_use]
    pub fn header(&self) -> HeaderGuess {
        self.header
    }

    /// 見出しの前の前書きの行の数（OPEN-11）。表として読み込むときは飛ばす。原本からは消さない。
    #[must_use]
    pub fn preamble_rows(&self) -> usize {
        self.preamble
    }
}

/// 引用符で包んだセルを読む流儀か（`Doubled`・`QuotedBackslash`）。
fn quoted(quoting: Quoting) -> bool {
    match quoting {
        Quoting::Doubled | Quoting::QuotedBackslash => true,
        Quoting::Backslash | Quoting::Plain => false,
    }
}

/// 1 位と競う候補（`Detection::rival`）。
fn rival_of(candidates: &[DialectCandidate]) -> Option<DialectCandidate> {
    let (top, rest) = candidates.split_first()?;
    rest.iter()
        .copied()
        .find(|candidate| !escapes_differently(top.dialect(), candidate.dialect()))
}

/// 区切りと引用符が同じで、引用の中の引用符の逃がし方（`Doubled` と `QuotedBackslash`）だけが違うか。
fn escapes_differently(one: Dialect, other: Dialect) -> bool {
    quoted(one.quoting())
        && quoted(other.quoting())
        && one.delimiter() == other.delimiter()
        && one.quote() == other.quote()
        && one.quoting() != other.quoting()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detecting::SAMPLE_LIMIT;
    use crate::line_ending::LineEnding;
    use crate::quoting::Quoting;
    use crate::records::Records;
    use alloc::string::String;
    use alloc::vec::Vec;

    /// 返す方言（`best`）で `input` を切ったレコードの数。
    fn strict_records(input: &[u8], detection: &Detection) -> usize {
        Records::new(input, detection.best()).count()
    }

    /// 前書きは、返す方言で切ったレコードを少なくとも 1 つ残す（GitHub Issue #12）。
    #[test]
    fn a_stray_quote_before_the_header_does_not_count_a_preamble_past_the_strict_records() {
        let input = b"\"Report Q3\nid,name,price\n1,a,2\n2,b,3\n3,c,4\n";
        let detection = Detection::of(input);
        assert_eq!(
            detection.best().unclosed_quote(),
            crate::unclosed_quote::UnclosedQuote::ToEnd
        );
        assert_eq!(strict_records(input, &detection), 1);
        assert_eq!(detection.preamble_rows(), 0);
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
    }

    /// 前書きらしい行・表の行・迷子の引用符を混ぜた入力。
    fn random_table(rng: &mut Xorshift) -> Vec<u8> {
        const PIECES: [&[u8]; 8] = [
            b"title\n",
            b"a,b,c,d\n",
            b"1,2,3,4\n",
            b",,,\n",
            b"\"",
            b"x,\"y\",z,w\n",
            b"note,,,\n",
            b"\n",
        ];
        let length = usize::try_from(rng.next() % 24).unwrap();
        let mut out = Vec::new();
        for _ in 0..length {
            out.extend_from_slice(PIECES[usize::try_from(rng.next() % 8).unwrap()]);
        }
        out
    }

    #[test]
    fn random_preambles_never_reach_past_the_records_of_the_returned_dialect() {
        let mut rng = Xorshift(0x9E37_79B9_7F4A_7C15);
        for _ in 0_u32..3_000 {
            let input = random_table(&mut rng);
            let detection = Detection::of(&input);
            let records = strict_records(&input, &detection);
            assert!(
                detection.preamble_rows() == 0 || detection.preamble_rows() < records,
                "{:?}: preamble {} of {records}",
                String::from_utf8_lossy(&input),
                detection.preamble_rows(),
            );
        }
    }

    fn best(input: &str) -> (u8, Quoting, u8) {
        let dialect = Detection::of(input.as_bytes()).best();
        (
            dialect.delimiter().byte(),
            dialect.quoting(),
            dialect.quote(),
        )
    }

    #[test]
    fn comma_csv_with_quotes() {
        let input = "id,name,price\n1,\"Tokyo, JP\",100\n2,Osaka,200\n3,\"Nagoya\",300\n";
        assert_eq!(best(input), (b',', Quoting::Doubled, b'"'));
        let detection = Detection::of(input.as_bytes());
        assert!(detection.confident());
        assert_eq!(detection.candidates().first().map(|c| c.columns()), Some(3));
    }

    #[test]
    fn semicolon_with_decimal_commas() {
        let input = "name;price;qty\nApfel;1,5;3\nBirne;2,75;10\nKiwi;0,9;7\n";
        assert_eq!(best(input).0, b';');
    }

    #[test]
    fn tab_separated() {
        let input = "a\tb\tc\n1\t2\t3\n4\t5\t6\n";
        assert_eq!(best(input).0, b'\t');
    }

    #[test]
    fn pipe_separated() {
        let input = "a|b|c\n1|2|3\nx|y|z\n";
        assert_eq!(best(input).0, b'|');
    }

    #[test]
    fn japanese_comma_csv() {
        let input = "支店,売上,日付\n東京,1000,2026/1/2\n大阪,2000,2026/1/3\n";
        let detection = Detection::of(input.as_bytes());
        assert_eq!(detection.best().delimiter().byte(), b',');
        assert_eq!(detection.header(), HeaderGuess::Present);
    }

    #[test]
    fn single_column_falls_back_without_confidence() {
        let detection = Detection::of(b"alpha\nbeta\ngamma\n");
        assert!(!detection.confident());
        assert_eq!(detection.best(), Dialect::rfc4180());
    }

    /// 引用の中の引用符の逃がし方だけが違う候補（`""` と `\"`）は競わず、区切りか引用符の違う候補と比べる
    /// （GitHub Issue #24）。
    #[test]
    fn a_rival_differs_in_the_delimiter_or_the_quote_not_only_in_the_escape() {
        use core::fmt::Write as _;
        let mut input = String::from("id,name,note\n");
        for row in 0..20_u32 {
            writeln!(input, "{row},\"a, b\",\"say \"\"hi\"\"\"").unwrap();
        }
        let detection = Detection::of(input.as_bytes());
        let best = detection.best();
        assert_eq!(
            (best.delimiter().byte(), best.quoting()),
            (b',', Quoting::Doubled)
        );
        let rival = detection.rival().unwrap().dialect();
        assert!(rival.delimiter() != best.delimiter() || rival.quote() != best.quote());
        assert!(detection.confident());
    }

    #[test]
    fn empty_input_falls_back() {
        let detection = Detection::of(b"");
        assert!(!detection.confident());
        assert_eq!(detection.best(), Dialect::rfc4180());
        assert_eq!(detection.header(), HeaderGuess::Unknown);
    }

    #[test]
    fn an_unclosed_quote_is_read_as_text_for_scoring_and_lowers_the_score() {
        let broken = Detection::of(b"a,b\n\"c,d\n");
        let clean = Detection::of(b"a,b\nc,d\n");
        let score_of = |detection: &Detection| {
            detection
                .candidates()
                .iter()
                .find(|c| c.dialect() == Dialect::rfc4180())
                .map(|c| c.score())
        };
        assert_eq!(broken.best(), Dialect::rfc4180());
        assert!(score_of(&broken) < score_of(&clean));
        assert!(score_of(&broken) > Some(0));
    }

    /// 見出し・正しい引用の行・閉じない引用符の行・普通の行を並べる。
    fn early_unclosed(normal_rows: usize) -> String {
        let mut text =
            String::from("id,name,memo\n1,\"Smith, John\",ok\n2,\"Doe, Jane\",ok\n3,x,\"broken\n");
        for _ in 0..normal_rows {
            text.push_str("4,y,z\n");
        }
        text
    }

    #[test]
    fn one_unclosed_quote_does_not_turn_the_dialect_into_plain() {
        // 標本の中（短いファイル）と、標本より長いファイル（最後のレコードを捨てる）の両方で。
        for rows in [50_usize, 5000] {
            let input = early_unclosed(rows);
            let best = Detection::of(input.as_bytes()).best();
            assert_eq!(best, Dialect::rfc4180(), "{rows}");
            assert_eq!(
                best.unclosed_quote(),
                crate::unclosed_quote::UnclosedQuote::ToEnd
            );
        }
    }

    #[test]
    fn backslash_escapes_without_delimiters_inside_quotes_are_detected() {
        let input = r#"id,name,note
1,"He said \"hi\" ok",a
2,"x \"y\" w",b
3,"plain",c
"#;
        assert_eq!(best(input), (b',', Quoting::QuotedBackslash, b'"'));
    }

    #[test]
    fn tsv_with_csv_style_quotes_is_read_with_doubled_quotes() {
        let input =
            "id\tname\tnote\n1\t\"Tokyo\tJP\"\tok\n2\t\"say \"\"hi\"\"\"\tok\n3\tplain\tok\n";
        assert_eq!(best(input), (b'\t', Quoting::Doubled, b'"'));
    }

    #[test]
    fn tsv_with_escaped_tabs_is_read_with_backslash_escapes() {
        let input = "id\tname\tnote\n1\tTokyo\\\tJP\tok\n2\tline\\nbreak\tok\n3\tback\\\\slash\tok\n4\tplain\tok\n";
        assert_eq!(best(input), (b'\t', Quoting::Backslash, b'"'));
    }

    #[test]
    fn tsv_with_stray_quotes_is_read_without_quoting() {
        // 同じ切り方の候補はまとめて優先の順の代表を返す（`'` の引用は、`'` の無いこのファイルでは引用なしと同じ）。
        // 名前ではなく、どのセルも `"` を字のまま読むことを確かめる。
        let input = "id\tname\tnote\n1\t\"quoted start\tok\n2\t5\" disk\tok\n3\tsay \"hi\"\tok\n4\t\"x\tok\n";
        let dialect = Detection::of(input.as_bytes()).best();
        let read: Vec<Vec<Vec<u8>>> = crate::records::Records::new(input.as_bytes(), dialect)
            .map(|record| {
                record
                    .fields()
                    .iter()
                    .map(|&field| crate::value::value(input.as_bytes(), field, &dialect))
                    .collect()
            })
            .collect();
        let literal: Vec<Vec<Vec<u8>>> = input
            .lines()
            .map(|line| {
                line.split('\t')
                    .map(|cell| cell.as_bytes().to_vec())
                    .collect()
            })
            .collect();
        assert_eq!(dialect.delimiter().byte(), b'\t');
        assert_eq!(read, literal);
    }

    #[test]
    fn backslash_letters_without_an_escaped_delimiter_keep_their_bytes() {
        // `\t` が字の `\` と `t` か、タブを逃がしたものかは行の形からは分からない。迷えば値を変えない流派で読む。
        let input = "id\tname\tnote\n1\tTokyo\\tJP\tok\n2\tline\\nbreak\tok\n3\tplain\tok\n";
        let (delimiter, quoting, _) = best(input);
        assert_eq!(delimiter, b'\t');
        assert_ne!(quoting, Quoting::Backslash);
    }

    #[test]
    fn doubled_quotes_without_delimiters_inside_stay_doubled() {
        let input = r#"id,name,note
1,"He said ""hi"" ok",a
2,"x ""y"" w",b
3,"plain",c
"#;
        assert_eq!(best(input), (b',', Quoting::Doubled, b'"'));
    }

    #[test]
    fn one_trailing_quote_does_not_turn_the_dialect_into_plain() {
        let mut input = String::from("id,name,memo\n1,\"Smith, John\",ok\n2,\"a\"b,ok\n");
        for _ in 0..20_usize {
            input.push_str("3,\"Doe, Jane\",ok\n");
        }
        assert_eq!(best(&input), (b',', Quoting::Doubled, b'"'));
    }

    fn preamble(input: &str) -> usize {
        Detection::of(input.as_bytes()).preamble_rows()
    }

    #[test]
    fn leading_sparse_rows_are_a_preamble() {
        let table = "id,name,city,price\n1,a,x,10\n2,b,y,20\n3,c,z,30\n";
        assert_eq!(preamble(table), 0);
        assert_eq!(preamble(&alloc::format!("PREAMBLE,,,\n,,,\n{table}")), 2);
        assert_eq!(
            preamble(&alloc::format!("売上の一覧\n作成日 2026-10-23\n\n{table}")),
            3
        );
        let detection = Detection::of(alloc::format!("Report,,,\n{table}").as_bytes());
        assert_eq!(detection.header(), HeaderGuess::Present);
    }

    #[test]
    fn a_header_missing_one_delimiter_is_not_a_preamble() {
        assert_eq!(
            preamble("id,name,city price\n1,a,x,10\n2,b,y,20\n3,c,z,30\n"),
            0
        );
    }

    #[test]
    fn a_header_broken_by_a_stray_quote_is_not_a_preamble() {
        // 見出しの迷子の引用符が 1 行目の引用符と組み違え、見出しは 2 つのセル（2 つめは次の行に食い込む）。
        let input = "id,\"name,city,price\n1,a,\"x\",10\n2,b,y,20\n3,c,z,30\n4,d,w,40\n";
        assert_eq!(preamble(input), 0);
    }

    /// e-Stat の人口動態の形: 2 つのセルの表題・1 つのセルの表題・注の行の後ろに、段に分けた見出し（GitHub Issue #15）。
    /// 表題と注だけを前書きとし、段の見出し（左端が空で中身が散る）は表に残す。
    #[test]
    fn short_title_rows_are_a_preamble_but_a_grouped_header_is_not() {
        let body = ",総数,病院,,,診療所,,\n,,総数,医師,助産師,総数,医師,助産師\n滋賀,10,4,3,1,6,5,1\n大津,20,8,6,2,12,10,2\n草津,30,9,8,1,21,20,1\n";
        let input =
            alloc::format!("平成27年,人口動態調査,,\n保管統計表　第１表\n注：数値は確定値\n{body}");
        assert_eq!(preamble(&input), 3);
        // 表題の右端の単位（`(単位 : %)`）があっても表題。
        let units = "国債金利情報,,,,,(単位 : %)\n基準日,1年,2年,3年,4年,5年\nR8.11.2,1.6,1.8,1.9,2.1,2.2\nR8.11.4,1.6,1.8,1.9,2.1,2.2\n";
        assert_eq!(preamble(units), 1);
    }

    /// 右に使われていない列が続く表で、見出しとデータを前書きとみなさない（GitHub Issue #15・東京都の表で 32 行を飛ばした）。
    /// 全角の空白だけのセルは空とみなす。
    #[test]
    fn unused_right_columns_and_ideographic_spaces_do_not_widen_the_table() {
        use core::fmt::Write as _;
        let mut input = String::from("地域,国,人口,,,,,,,\n");
        for row in 0..40_u32 {
            writeln!(input, "アジア,国{row},{row}00,,,,,,,").unwrap();
        }
        assert_eq!(preamble(&input), 0);
        input.push_str("第２表 国籍別人口,,,,,,,,,\u{3000}\u{3000}\n");
        assert_eq!(preamble(&input), 0);
    }

    /// 見出しの先頭の迷子の引用符が、1 行のまま隣のセルを飲み込んで短い行になっても、前書きにしない（Pollock の
    /// `row_extra_quote0_col0` の形）。
    #[test]
    fn a_header_that_swallowed_its_cells_on_one_line_is_not_a_preamble() {
        let input = "\"id,name,city,\"price\",unit,note\n1,a,x,10,kg,-\n2,b,y,20,kg,-\n3,c,z,30,kg,-\n4,d,w,40,kg,-\n";
        assert_eq!(preamble(input), 0);
    }

    #[test]
    fn narrow_or_wholly_sparse_tables_have_no_preamble() {
        assert_eq!(preamble("title\na,1\nb,2\nc,3\n"), 0);
        // 全部の行が疎なら、表の行を 1 行残す。
        assert_eq!(preamble("x,,,\n,y,,\n,,z,\n"), 2);
    }

    #[test]
    fn a_stray_apostrophe_does_not_make_the_single_quote_win() {
        let mut input = String::from("a,'tis\n");
        for _ in 0..20_u32 {
            input.push_str("1,b\n");
        }
        assert_eq!(Detection::of(input.as_bytes()).best().quote(), b'"');
    }

    #[test]
    fn mixed_line_endings_are_counted() {
        let detection = Detection::of(b"a,b\r\nc,d\ne,f\r\n");
        let endings = detection.line_endings();
        assert_eq!((endings.crlf(), endings.lf(), endings.cr()), (2, 1, 0));
        assert!(endings.mixed());
        assert_eq!(endings.dominant(), Some(LineEnding::CrLf));
    }

    #[test]
    fn header_absent_when_the_first_row_is_numeric_too() {
        let detection = Detection::of(b"1,2\n3,4\n5,6\n");
        assert_eq!(detection.header(), HeaderGuess::Absent);
    }

    #[test]
    fn truncated_sample_drops_the_partial_last_record() {
        let mut input = String::new();
        while input.len() <= SAMPLE_LIMIT {
            input.push_str("1,\"x\ny\",3\n");
        }
        let detection = Detection::of(input.as_bytes());
        assert_eq!(detection.best(), Dialect::rfc4180());
        assert!(detection.confident());
    }

    #[test]
    fn candidates_are_ordered_by_score() {
        let detection = Detection::of(b"a,b,c\n1,2,3\n");
        let scores: Vec<u32> = detection.candidates().iter().map(|c| c.score()).collect();
        let mut sorted = scores.clone();
        sorted.sort_by(|a, b| b.cmp(a));
        assert_eq!(scores, sorted);
    }

    #[test]
    fn shift_jis_samples_are_tried_with_shift_jis_units() {
        // 「ポ」（83 7C）と「表」（95 5C）を含む `|` 区切り。2 バイト目に `|` があっても列の数はそろう。
        let mut sample = Vec::new();
        for _ in 0..20_u32 {
            sample.extend_from_slice(b"id|\x83\x7C\x83\x7C|\x95\x5C\n1|\x83\x7C|x\n");
        }
        let detection = Detection::of_encoded(&sample, TextUnits::ShiftJis);
        let best = detection.best();
        assert_eq!(best.delimiter().byte(), b'|');
        assert_eq!(best.units(), TextUnits::ShiftJis);
        assert!(
            detection
                .candidates()
                .iter()
                .all(|candidate| candidate.dialect().units() == TextUnits::ShiftJis)
        );
        assert_eq!(detection.candidates().first().map(|c| c.columns()), Some(3));
    }
}
