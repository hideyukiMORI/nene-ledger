//! 実ファイル（`target/tmp` の一時ファイル）で、塊の読み出し・索引・中止・範囲の取り出しを確かめる。

// 🔴 中身を `#[cfg(test)]` の mod に入れる。補助の関数も clippy.toml の「テストは免除」に入り（RS-005）、
// CNF-015 の検査器もテストとして読む（一時ファイルを書くのはテストだけ）。
#[cfg(test)]
mod reading {
    use std::ops::ControlFlow;
    use std::path::PathBuf;

    use nene_ledger_core::dialect::Dialect;
    use nene_ledger_core::record::Record;
    use nene_ledger_core::records::Records;
    use nene_ledger_core::text_encoding::TextEncoding;
    use nene_ledger_core::text_format::TextFormat;
    use nene_ledger_core::unclosed_quote::UnclosedQuote;
    use nene_ledger_core::value::value;
    use nene_ledger_scan::block_size::BlockSize;
    use nene_ledger_scan::index_outcome::IndexOutcome;
    use nene_ledger_scan::indexing::build_index;
    use nene_ledger_scan::job_control::JobControl;
    use nene_ledger_scan::progress::Progress;
    use nene_ledger_scan::record_length::RecordLength;
    use nene_ledger_scan::record_range::RecordRange;
    use nene_ledger_scan::scan_failure::ScanFailure;
    use nene_ledger_scan::source::Source;
    use nene_ledger_scan::sparse_index::SparseIndex;
    use nene_ledger_scan::window_record::{RECORD_READ_LIMIT, WindowRecord};
    use nene_ledger_scan::windowing::read_records;

    /// テストごとに別の名前で一時ファイルを置く（テストは並列に走る）。
    fn fixture(name: &str, contents: &[u8]) -> PathBuf {
        let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("scan-{name}.csv"));
        std::fs::write(&path, contents).unwrap();
        path
    }

    /// 引用符の中の改行・`""`・CRLF と LF の混在・日本語を含む、`rows` 行の CSV。
    fn sample(rows: usize) -> Vec<u8> {
        let mut out = Vec::new();
        for row in 0..rows {
            let line = match row % 4 {
                0 => format!("{row},\"東京都\n千代田区\",x\r\n"),
                1 => format!("{row},\"a\"\"b\",,\n"),
                2 => format!("{row},0012,1.20E+03\r\n"),
                _ => format!("{row}\n"),
            };
            out.extend_from_slice(line.as_bytes());
        }
        out
    }

    fn block(size: usize) -> BlockSize {
        BlockSize::new(size).unwrap()
    }

    fn finished(outcome: IndexOutcome) -> SparseIndex {
        match outcome {
            IndexOutcome::Finished(index) => index,
            IndexOutcome::Cancelled => panic!("中止していないのに止まった"),
        }
    }

    /// BOM の無い UTF-8 で読む。
    fn utf8(dialect: Dialect) -> TextFormat {
        TextFormat::new(TextEncoding::Utf8, dialect)
    }

    fn literal() -> Dialect {
        Dialect::rfc4180().with_unclosed_quote(UnclosedQuote::Literal)
    }

    fn index_with(source: &Source, dialect: Dialect, size: usize) -> SparseIndex {
        finished(
            build_index(source, utf8(dialect), block(size), |_| {
                ControlFlow::Continue(())
            })
            .unwrap(),
        )
    }

    fn index_of(source: &Source, size: usize) -> SparseIndex {
        finished(
            build_index(source, utf8(Dialect::rfc4180()), block(size), |_| {
                ControlFlow::Continue(())
            })
            .unwrap(),
        )
    }

    /// 裏のスレッドで索引を作り、`control` で進み具合と中止をやり取りする（ADR 0014）。
    fn in_background(source: &Source, control: &JobControl) -> IndexOutcome {
        let work = || {
            build_index(source, utf8(Dialect::rfc4180()), block(4096), |p| {
                control.report(p)
            })
        };
        std::thread::scope(|scope| scope.spawn(work).join().unwrap()).unwrap()
    }

    /// 参照実装で全体を切ったときの目印とレコードの数。
    fn reference_marks(input: &[u8]) -> (Vec<u64>, u64) {
        let records: Vec<Record> = Records::new(input, Dialect::rfc4180()).collect();
        let marks = records
            .iter()
            .step_by(1024)
            .map(|record| u64::try_from(record.raw().start()).unwrap())
            .collect();
        (marks, u64::try_from(records.len()).unwrap())
    }

    #[test]
    fn index_matches_the_reference_for_any_block_size() {
        let input = sample(5000);
        let source = Source::open(&fixture("index", &input)).unwrap();
        let (marks, records) = reference_marks(&input);
        for size in [1_usize, 7, 4096, 65_536, 1_usize << 20_u32] {
            let index = index_of(&source, size);
            assert_eq!(index.marks(), marks.as_slice(), "{size}");
            assert_eq!(index.records(), records);
            assert_eq!(index.bytes(), source.len());
        }
    }

    #[test]
    fn empty_file_has_no_records() {
        let source = Source::open(&fixture("empty", b"")).unwrap();
        assert!(source.is_empty());
        let index = index_of(&source, 16);
        assert_eq!((index.records(), index.marks().len()), (0, 0));
    }

    #[test]
    fn missing_file_is_an_open_failure() {
        let path =
            PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("scan-missing-never-written.csv");
        assert_eq!(
            Source::open(&path).unwrap_err(),
            ScanFailure::Open(std::io::ErrorKind::NotFound)
        );
    }

    #[test]
    fn progress_grows_and_cancelling_stops_midway() {
        let input = sample(2000);
        let source = Source::open(&fixture("cancel", &input)).unwrap();
        let mut seen: Vec<Progress> = Vec::new();
        let outcome = build_index(&source, utf8(Dialect::rfc4180()), block(1000), |progress| {
            seen.push(progress);
            if seen.len() == 3 {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        })
        .unwrap();
        assert_eq!(outcome, IndexOutcome::Cancelled);
        let done: Vec<u64> = seen.iter().map(|progress| progress.done()).collect();
        assert_eq!(done, vec![1000, 2000, 3000]);
        assert!(seen.iter().all(|progress| progress.total() == source.len()));
        // 数えたレコードの数も添えて増えていく（総数の見積もりに使う）。
        let records: Vec<u64> = seen.iter().map(|progress| progress.records()).collect();
        assert!(records[0] > 0 && records.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn runs_on_a_background_thread_and_reports_through_the_control() {
        let input = sample(3000);
        let source = Source::open(&fixture("thread", &input)).unwrap();
        let control = JobControl::new(source.len());
        assert_eq!(control.progress(), Progress::new(0, source.len(), None));
        let outcome = in_background(&source, &control);
        assert_eq!(finished(outcome).records(), 3000);
        assert_eq!(
            control.progress(),
            Progress::new(source.len(), source.len(), None).with_records(3000)
        );
    }

    #[test]
    fn cancelled_control_stops_after_the_first_block() {
        let input = sample(3000);
        let source = Source::open(&fixture("control-cancel", &input)).unwrap();
        let control = JobControl::new(source.len());
        control.cancel();
        let outcome = in_background(&source, &control);
        assert_eq!(outcome, IndexOutcome::Cancelled);
        assert_eq!(control.progress().done(), 4096);
    }

    /// 切り詰めていないレコードの終わり方・形・セルの値が、参照実装と同じ。
    fn assert_same_cells(got: &WindowRecord, want: &Record, input: &[u8], dialect: Dialect) {
        let record = got.record();
        assert_eq!(
            (record.terminator(), record.shape()),
            (want.terminator(), want.shape())
        );
        let got_values: Vec<Vec<u8>> = record
            .fields()
            .iter()
            .map(|&f| value(got.bytes(), f, &dialect))
            .collect();
        let want_values: Vec<Vec<u8>> = want
            .fields()
            .iter()
            .map(|&f| value(input, f, &dialect))
            .collect();
        assert_eq!(got_values, want_values);
    }

    /// 範囲の取り出しが、参照実装で全体を切った結果の同じ範囲と、原本のバイト列とセルの値で一致する。
    /// 上限を超えたレコードは、先頭の上限分が原本と同じで、全体の長さが合っている。
    fn assert_window_matches(
        input: &[u8],
        source: &Source,
        index: &SparseIndex,
        range: RecordRange,
    ) {
        assert_window_matches_in(input, source, (index, Dialect::rfc4180()), range);
    }

    /// 作り終えた索引があれば、上限を超えたレコードがあっても、範囲のレコードは全部返る（GitHub Issue #13）。
    /// 長さが分からない（`MoreThan`）のは、索引がその後ろを数えていない最後のレコードだけ。
    fn assert_count(got: &[WindowRecord], expected: usize, range: RecordRange) {
        assert_eq!(got.len(), expected, "{range:?}");
        let unknown = got
            .iter()
            .rev()
            .skip(1)
            .any(|record| matches!(record.length(), RecordLength::MoreThan(_)));
        assert!(!unknown, "{range:?}");
    }

    /// 長さは、終わりまで見つけたなら一致し、上限を超えたなら実際の長さが上限より長い。
    fn assert_length(got: RecordLength, actual: usize) {
        match got {
            RecordLength::Exact(length) => assert_eq!(usize::try_from(length).unwrap(), actual),
            RecordLength::MoreThan(limit) => assert!(actual > usize::try_from(limit).unwrap()),
        }
    }

    /// 方言を指定した `assert_window_matches`。
    fn assert_window_matches_in(
        input: &[u8],
        source: &Source,
        (index, dialect): (&SparseIndex, Dialect),
        range: RecordRange,
    ) {
        let expected: Vec<Record> = Records::new(input, dialect)
            .skip(usize::try_from(range.first()).unwrap())
            .take(range.count())
            .collect();
        let window = read_records(source, index, utf8(dialect), range).unwrap();
        assert_count(window.records(), expected.len(), range);
        for (got, want) in window.records().iter().zip(&expected) {
            let start = want.raw().start();
            let end = want.raw().end() + want.terminator().len();
            assert_eq!(usize::try_from(got.offset()).unwrap(), start);
            assert_length(got.length(), end - start);
            if got.is_truncated() {
                assert_eq!(got.bytes().len(), RECORD_READ_LIMIT.get());
                assert_eq!(got.bytes(), &input[start..start + got.bytes().len()]);
                continue;
            }
            assert_eq!(got.bytes(), &input[start..end]);
            assert_same_cells(got, want, input, dialect);
        }
    }

    #[test]
    fn ranges_match_the_reference_including_across_marks_and_past_the_end() {
        let input = sample(5000);
        let source = Source::open(&fixture("window", &input)).unwrap();
        let index = index_of(&source, 4096);
        for (first, count) in [
            (0_u64, 10_usize),
            (1020, 10),
            (1024, 1),
            (2047, 3),
            (4990, 50),
            (5000, 5),
            (9999, 1),
            (0, 0),
        ] {
            assert_window_matches(&input, &source, &index, RecordRange::new(first, count));
        }
    }

    #[test]
    fn a_record_longer_than_the_limit_is_read_only_up_to_the_limit() {
        let mut input = b"head\n\"".to_vec();
        input.extend(b"x,y\n".repeat(700_000));
        input.extend(b"\",tail\nlast\n");
        let source = Source::open(&fixture("long-record", &input)).unwrap();
        let index = index_of(&source, 1_usize << 20);
        assert_eq!(index.records(), 3);
        let window = read_records(
            &source,
            &index,
            utf8(Dialect::rfc4180()),
            RecordRange::new(0, 3),
        )
        .unwrap();
        let truncated: Vec<bool> = window
            .records()
            .iter()
            .map(WindowRecord::is_truncated)
            .collect();
        // 2 つ目は上限を超えたので先頭の分だけ。索引が後ろに 3 つ目を数えているので、終わりまで切り進めて
        // 3 つ目も返す（GitHub Issue #13。直す前はここで止まり、3 つ目が消えていた）。
        assert_eq!(truncated, vec![false, true, false]);
        assert_eq!(
            window.records()[1].length(),
            RecordLength::Exact(u64::try_from(input.len() - 10).unwrap())
        );
        assert_window_matches(&input, &source, &index, RecordRange::new(0, 3));
        assert_window_matches(&input, &source, &index, RecordRange::new(2, 1));
    }

    #[test]
    fn an_unclosed_quote_to_the_end_is_one_truncated_record() {
        let mut input = b"a,b\n\"".to_vec();
        input.extend(b"x,y\n".repeat(600_000));
        let source = Source::open(&fixture("unclosed", &input)).unwrap();
        let index = index_of(&source, 4096);
        assert_eq!(index.records(), 2);
        assert_window_matches(&input, &source, &index, RecordRange::new(0, 5));
    }

    #[test]
    fn unclosed_quote_is_remembered_and_a_long_quoted_cell_is_reported() {
        let mut input = b"a,b\n\"".to_vec();
        input.extend(b"x,y\n".repeat(600_000));
        let source = Source::open(&fixture("notice", &input)).unwrap();
        let mut notices = Vec::new();
        let outcome = build_index(
            &source,
            utf8(Dialect::rfc4180()),
            block(1 << 20),
            |progress| {
                notices.push(progress.long_quoted_cell());
                ControlFlow::Continue(())
            },
        )
        .unwrap();
        assert_eq!(finished(outcome).unclosed_quotes(), &[4]);
        assert_eq!(notices.first(), Some(&None));
        assert_eq!(notices.last(), Some(&Some(4)));
    }

    #[test]
    fn literal_reads_on_after_the_unclosed_quote_in_the_index_and_the_window() {
        let mut input = b"a,b\n\"x".to_vec();
        input.extend(b",y\n1,2\n".repeat(3000));
        let source = Source::open(&fixture("literal", &input)).unwrap();
        for size in [7_usize, 4096] {
            let index = index_with(&source, literal(), size);
            let expected = Records::new(&input, literal()).count();
            assert_eq!(index.records(), u64::try_from(expected).unwrap());
            assert_eq!(expected, 6001);
            assert_eq!(index.unclosed_quotes(), &[4]);
            for (first, count) in [(0_u64, 3_usize), (1023, 4), (5990, 20)] {
                let range = RecordRange::new(first, count);
                assert_window_matches_in(&input, &source, (&index, literal()), range);
            }
        }
    }

    /// 閉じた引用のセルが 2 MiB ある 10 レコードの表（GitHub Issue #13 の 1）。
    fn long_closed_cell() -> Vec<u8> {
        let mut input = b"h1,h2\n1,\"".to_vec();
        input.extend(core::iter::repeat_n(b'z', 2 << 20_u32));
        input.extend(b"\"\n");
        for row in 2..10_u32 {
            input.extend(format!("{row},x\n").as_bytes());
        }
        input
    }

    #[test]
    fn the_records_after_a_long_closed_cell_are_returned() {
        let input = long_closed_cell();
        let source = Source::open(&fixture("long-closed-cell", &input)).unwrap();
        let index = index_of(&source, 1 << 20);
        assert_eq!(index.records(), 10);
        let window = read_records(
            &source,
            &index,
            utf8(Dialect::rfc4180()),
            RecordRange::new(0, 10),
        )
        .unwrap();
        let truncated: Vec<bool> = window
            .records()
            .iter()
            .map(WindowRecord::is_truncated)
            .collect();
        // 直す前は 2 件で止まり、3 件目から後ろが消えていた。
        let mut expected = vec![false; 10];
        expected[1] = true;
        assert_eq!(truncated, expected);
        for (first, count) in [(0_u64, 10_usize), (1, 9), (1, 1), (2, 8), (0, 3)] {
            assert_window_matches(&input, &source, &index, RecordRange::new(first, count));
        }
    }

    #[test]
    fn literal_reads_on_after_an_unclosed_quote_longer_than_the_limit() {
        // 引用符が閉じないまま 1 MiB を超えて続く（GitHub Issue #13 の 2）。
        let mut input = b"a\n\"b\n".to_vec();
        input.extend(b"x,y\n".repeat(300_000));
        let source = Source::open(&fixture("literal-long", &input)).unwrap();
        let index = index_with(&source, literal(), 1 << 20);
        assert_eq!(index.records(), 300_002);
        assert_eq!(index.unclosed_quotes(), &[2]);
        let window =
            read_records(&source, &index, utf8(literal()), RecordRange::new(0, 5)).unwrap();
        // 直す前は 2 件だけで、3 バイトの 2 件目を切り詰めと示していた。
        assert_eq!(window.records().len(), 5);
        assert!(!window.records().iter().any(WindowRecord::is_truncated));
        for (first, count) in [(0_u64, 5_usize), (1, 1), (2, 3), (1023, 3), (299_990, 20)] {
            let range = RecordRange::new(first, count);
            assert_window_matches_in(&input, &source, (&index, literal()), range);
        }
    }

    #[test]
    fn without_the_index_a_long_record_stops_the_window_at_the_limit() {
        // 索引を作る前は、後ろにレコードがあるかを知らないので、上限で止める（BIG-01・公開前の Issue #23）。
        let input = long_closed_cell();
        let source = Source::open(&fixture("long-closed-cell-unindexed", &input)).unwrap();
        let window = read_records(
            &source,
            &SparseIndex::empty(),
            utf8(Dialect::rfc4180()),
            RecordRange::new(0, 10),
        )
        .unwrap();
        let lengths: Vec<RecordLength> =
            window.records().iter().map(WindowRecord::length).collect();
        assert_eq!(
            lengths,
            vec![
                RecordLength::Exact(6),
                RecordLength::MoreThan(u64::try_from(RECORD_READ_LIMIT.get()).unwrap())
            ]
        );
    }

    #[test]
    fn an_unclosed_quote_to_the_end_stops_at_the_limit_even_with_the_index() {
        // 索引が最後のレコードと知っているので、終わりを探しに行かない（BIG-01）。
        let mut input = b"a,b\n\"".to_vec();
        input.extend(b"x,y\n".repeat(600_000));
        let source = Source::open(&fixture("unclosed-last", &input)).unwrap();
        let index = index_of(&source, 1 << 20);
        let window = read_records(
            &source,
            &index,
            utf8(Dialect::rfc4180()),
            RecordRange::new(1, 5),
        )
        .unwrap();
        let lengths: Vec<RecordLength> =
            window.records().iter().map(WindowRecord::length).collect();
        let limit = u64::try_from(RECORD_READ_LIMIT.get()).unwrap();
        assert_eq!(lengths, vec![RecordLength::MoreThan(limit)]);
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
    }

    /// 乱数の表の字。引用符は少なめ（引用が長く続きすぎない）、改行は多め（目印をまたぐ数のレコード）。
    const TABLE_ALPHABET: [&[u8]; 20] = [
        b"a",
        b"a",
        b"a",
        b"a",
        b"a",
        b"a",
        b"b",
        b"0",
        b",",
        b",",
        b",",
        b"\n",
        b"\n",
        b"\n",
        b"\n",
        b"\r\n",
        b"\r",
        "\u{3042}".as_bytes(),
        b"\\",
        b"\"",
    ];

    /// `tokens` 個の字の乱数の表。`long` なら、どこかに上限の 2 倍を少し超える `z` の並びを挟む
    /// （閉じた引用の中・閉じない引用の中・引用の外のどこに落ちるかも乱数）。範囲の取り出しは 1 MiB の塊の
    /// 終わりで長さを見るので、上限を少し超えるだけでは、塊の境目の位置によっては上限に当たらない。
    fn random_table(rng: &mut Xorshift, tokens: usize, long: bool) -> Vec<u8> {
        let at = rng.below(tokens);
        let mut out = Vec::new();
        for token in 0..tokens {
            if long && token == at {
                let extra = rng.below(64);
                out.extend(core::iter::repeat_n(
                    b'z',
                    2 * RECORD_READ_LIMIT.get() + 1 + extra,
                ));
            }
            out.extend_from_slice(TABLE_ALPHABET[rng.below(20)]);
        }
        out
    }

    fn random_dialects() -> [Dialect; 3] {
        let backslash = Dialect::new(
            nene_ledger_core::delimiter::Delimiter::COMMA,
            nene_ledger_core::quoting::Quoting::QuotedBackslash,
            b'"',
        )
        .unwrap();
        [
            Dialect::rfc4180(),
            literal(),
            backslash.with_unclosed_quote(UnclosedQuote::Literal),
        ]
    }

    /// 全体と、乱数の 6 つの範囲（始まりはファイルの終わりを少し越えるまで・長さは目印を 2 つまたぐまで）。
    fn random_ranges(rng: &mut Xorshift, records: usize) -> Vec<RecordRange> {
        let picked = (0..6_u32).map(|_| {
            let first = u64::try_from(rng.below(records + 3)).unwrap();
            RecordRange::new(first, rng.below(2100))
        });
        core::iter::once(RecordRange::new(0, records))
            .chain(picked)
            .collect()
    }

    /// 乱数の表を、索引の塊の大きさ `sizes` ごとに索引を作り、乱数の範囲で参照実装と突き合わせる。
    fn assert_random_ranges(rng: &mut Xorshift, input: &[u8], name: &str, sizes: &[usize]) {
        let source = Source::open(&fixture(name, input)).unwrap();
        let cases = random_dialects()
            .into_iter()
            .flat_map(|dialect| sizes.iter().map(move |&size| (dialect, size)));
        for (dialect, size) in cases {
            let records = Records::new(input, dialect).count();
            let index = index_with(&source, dialect, size);
            assert_eq!(index.records(), u64::try_from(records).unwrap());
            for range in random_ranges(rng, records) {
                assert_window_matches_in(input, &source, (&index, dialect), range);
            }
        }
    }

    #[test]
    fn random_ranges_match_the_reference_for_every_index_block_size() {
        // 範囲の始まり・目印をまたぐ長さ・ファイルの終わりを越える範囲を乱数で選ぶ（QLT-008）。
        let mut rng = Xorshift(0x2545_F491_4F6C_DD1D);
        for round in 0..12_u32 {
            let input = random_table(&mut rng, 9000, false);
            assert_random_ranges(
                &mut rng,
                &input,
                &format!("random-{round}"),
                &[7, 4096, 1 << 20_u32],
            );
        }
    }

    #[test]
    fn random_ranges_around_a_record_longer_than_the_limit_match_the_reference() {
        // 上限を超える並びが、閉じた引用・閉じない引用・引用の外のどこに落ちても、索引が数えたレコードは全部返る
        // （GitHub Issue #13）。
        let mut rng = Xorshift(0x9E6C_63D0_676A_9A99);
        for round in 0..5_u32 {
            let input = random_table(&mut rng, 6000, true);
            assert_random_ranges(
                &mut rng,
                &input,
                &format!("random-long-{round}"),
                &[1 << 20_u32],
            );
        }
    }

    /// `Literal` で読んでも、上限を超える閉じた引用のセルを切り詰めた行は、厳格に読むのと同じセルになる（GitHub Issue #34。
    /// 直す前は、切った所に閉じる引用符が無いのでセルの中のカンマで割れ、349,526 のセルになった）。
    #[test]
    fn a_long_closed_quoted_cell_is_not_split_when_read_literally() {
        let mut input = b"a,b,c\n1,\"".to_vec();
        while input.len() < 2 * RECORD_READ_LIMIT.get() {
            input.extend_from_slice(b"x,y");
        }
        input.extend_from_slice(b"\",3\n2,q,r\n");
        let source = Source::open(&fixture("long-closed-quote-literal", &input)).unwrap();
        let cells = |dialect: Dialect| -> Vec<(usize, bool)> {
            let index = index_with(&source, dialect, 1 << 20_u32);
            read_records(&source, &index, utf8(dialect), RecordRange::new(0, 3))
                .unwrap()
                .records()
                .iter()
                .map(|record| (record.record().fields().len(), record.is_truncated()))
                .collect()
        };
        let strict = cells(Dialect::rfc4180());
        assert_eq!(strict, [(3, false), (2, true), (3, false)]);
        assert_eq!(cells(literal()), strict);
    }

    /// 本当に閉じない引用符のある長い行は、`Literal` を選んだなら引用符を文字として読み、区切りで割る。
    #[test]
    fn a_long_line_with_a_really_unclosed_quote_is_still_read_literally() {
        let mut input = b"a,b,c\n1,\"".to_vec();
        while input.len() < 2 * RECORD_READ_LIMIT.get() {
            input.extend_from_slice(b"x,y");
        }
        input.extend_from_slice(b"\n");
        let source = Source::open(&fixture("long-unclosed-quote-literal", &input)).unwrap();
        let index = index_with(&source, literal(), 1 << 20_u32);
        assert_eq!(index.unclosed_quotes().len(), 1);
        let window =
            read_records(&source, &index, utf8(literal()), RecordRange::new(0, 2)).unwrap();
        let long = &window.records()[1];
        assert!(long.is_truncated());
        assert!(long.record().fields().len() > 3);
    }
}
