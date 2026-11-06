//! 範囲を塊ごとに読み進めて畳み込む仕事。

// 🔴 中身を `#[cfg(test)]` の mod に入れる（reading.rs と同じ理由）。
#[cfg(test)]
mod folding {
    use core::ops::ControlFlow;
    use std::path::PathBuf;
    use std::sync::Arc;

    use nene_ledger_application::wakeup::Wakeup;
    use nene_ledger_core::dialect::Dialect;
    use nene_ledger_core::text_encoding::TextEncoding;
    use nene_ledger_core::text_format::TextFormat;
    use nene_ledger_core::unclosed_quote::UnclosedQuote;
    use nene_ledger_scan::block_size::BlockSize;
    use nene_ledger_scan::indexed_source::IndexedSource;
    use nene_ledger_scan::indexing::build_index;
    use nene_ledger_scan::range_folding::fold_range;
    use nene_ledger_scan::range_job::RangeJob;
    use nene_ledger_scan::range_outcome::RangeOutcome;
    use nene_ledger_scan::record_fold::RecordFold;
    use nene_ledger_scan::record_range::RecordRange;
    use nene_ledger_scan::source::Source;
    use nene_ledger_scan::sparse_index::SparseIndex;
    use nene_ledger_scan::window_record::WindowRecord;
    use nene_ledger_scan::windowing::read_records;

    /// 起こされても何もしない。
    #[derive(Debug)]
    struct Quiet;

    impl Wakeup for Quiet {
        fn wake(&self) {}
    }

    /// 渡されたレコードの番号と中身を残す。
    #[derive(Debug, Default, PartialEq, Eq)]
    struct Collect(Vec<(u64, Vec<u8>)>);

    impl RecordFold for Collect {
        fn record(&mut self, number: u64, record: &WindowRecord) {
            self.0.push((number, record.bytes().to_vec()));
        }
    }

    fn path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("folding-{name}.csv"))
    }

    fn sample(rows: usize) -> Vec<u8> {
        let mut out = Vec::new();
        for row in 0..rows {
            out.extend_from_slice(format!("{row},\"東京\n都\",x\r\n").as_bytes());
        }
        out
    }

    fn utf8() -> TextFormat {
        TextFormat::new(TextEncoding::Utf8, Dialect::rfc4180())
    }

    fn from(name: &str, rows: usize, index: SparseIndex) -> IndexedSource {
        std::fs::write(path(name), sample(rows)).unwrap();
        IndexedSource::new(Source::open(&path(name)).unwrap(), index, utf8())
    }

    fn whole(from: &IndexedSource, range: RecordRange) -> Vec<(u64, Vec<u8>)> {
        let window = read_records(from.source(), from.index(), from.format(), range).unwrap();
        window
            .records()
            .iter()
            .zip(range.first()..)
            .map(|(record, number)| (number, record.bytes().to_vec()))
            .collect()
    }

    #[test]
    fn chunks_give_the_same_records_as_one_read_with_or_without_an_index() {
        let plain = from("plain", 3000, SparseIndex::empty());
        let index = build_index(plain.source(), utf8(), BlockSize::MEBIBYTE, |_| {
            ControlFlow::Continue(())
        })
        .unwrap();
        let indexed = match index {
            nene_ledger_scan::index_outcome::IndexOutcome::Finished(index) => {
                from("indexed", 3000, index)
            }
            nene_ledger_scan::index_outcome::IndexOutcome::Cancelled => panic!("止まった"),
        };
        // 範囲がファイルの終わりを越えても、ある分だけ。
        let ranges = [RecordRange::new(0, 3000), RecordRange::new(1500, 2000)];
        for source in [&plain, &indexed] {
            for (range, size) in ranges
                .iter()
                .flat_map(|&range| [7_usize, 1024, 5000].map(|size| (range, size)))
            {
                assert_chunks_match(source, range, size);
            }
        }
    }

    /// `size` ずつの塊で畳み込んでも、一度に読んだのと同じレコードが同じ順で渡り、進み具合が増えていく。
    fn assert_chunks_match(source: &IndexedSource, range: RecordRange, size: usize) {
        let expected = whole(source, range);
        let mut collect = Collect::default();
        let mut reports = Vec::new();
        let flow = fold_range(source, range.chunks(size), &mut collect, |done| {
            reports.push(done);
            ControlFlow::Continue(())
        })
        .unwrap();
        assert_eq!(flow, ControlFlow::Continue(()));
        assert_eq!(collect.0, expected, "塊 {size}");
        assert!(reports.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(reports.last().copied(), u64::try_from(expected.len()).ok());
    }

    /// レコードの番号 `stop` まで受け取ったら、もう要らないと言う。
    #[derive(Debug, Default)]
    struct Until {
        stop: u64,
        seen: Vec<u64>,
    }

    impl RecordFold for Until {
        fn record(&mut self, number: u64, _record: &WindowRecord) {
            self.seen.push(number);
        }

        fn is_done(&self) -> bool {
            self.seen.last() == Some(&self.stop)
        }
    }

    /// 畳み込みが「もう要らない」と言えば、塊の途中でも残りを読まずに終える（中止ではない・ADR 0030 の決定 4）。
    #[test]
    fn a_fold_that_is_done_ends_without_reading_the_rest() {
        let plain = from("until", 3000, SparseIndex::empty());
        let mut until = Until {
            stop: 1234,
            seen: Vec::new(),
        };
        let mut reports = Vec::new();
        let flow = fold_range(
            &plain,
            RecordRange::new(1000, 2000).chunks(100),
            &mut until,
            |done| {
                reports.push(done);
                ControlFlow::Continue(())
            },
        );
        assert_eq!(flow.unwrap(), ControlFlow::Continue(()));
        assert_eq!(until.seen.first(), Some(&1000));
        assert_eq!(until.seen.last(), Some(&1234));
        assert_eq!(until.seen.len(), 235);
        assert_eq!(reports.last(), Some(&200));
    }

    #[test]
    fn a_break_stops_after_the_chunk() {
        let source = from("break", 100, SparseIndex::empty());
        let mut collect = Collect::default();
        let flow = fold_range(
            &source,
            RecordRange::new(0, 100).chunks(10),
            &mut collect,
            |_| ControlFlow::Break(()),
        )
        .unwrap();
        assert_eq!(flow, ControlFlow::Break(()));
        assert_eq!(collect.0.len(), 10);
    }

    #[test]
    fn the_job_runs_to_the_end_and_reports_the_records() {
        let source = from("job", 10_000, SparseIndex::empty());
        let job = RangeJob::spawn(
            source,
            RecordRange::new(5, 9_000),
            Collect::default(),
            Arc::new(Quiet),
        )
        .unwrap();
        let progress = loop {
            let progress = job.progress();
            if progress.done() == progress.total() {
                break progress;
            }
            std::thread::yield_now();
        };
        assert_eq!(progress.total(), 9_000);
        let RangeOutcome::Finished(collect) = job.wait().unwrap() else {
            panic!("止まった");
        };
        assert_eq!(collect.0.len(), 9_000);
        assert_eq!(collect.0.first().map(|(number, _)| *number), Some(5));
    }

    #[test]
    fn a_cancelled_job_returns_no_result() {
        let source = from("cancel", 200_000, SparseIndex::empty());
        let job = RangeJob::spawn(
            source,
            RecordRange::new(0, 200_000),
            Collect::default(),
            Arc::new(Quiet),
        )
        .unwrap();
        job.cancel();
        assert_eq!(job.wait().unwrap(), RangeOutcome::Cancelled);
    }

    /// 作り終えた索引で、`input` を `dialect` で読む。
    fn indexed(name: &str, input: &[u8], dialect: Dialect) -> IndexedSource {
        std::fs::write(path(name), input).unwrap();
        let format = TextFormat::new(TextEncoding::Utf8, dialect);
        let source = Source::open(&path(name)).unwrap();
        let outcome = build_index(&source, format, BlockSize::MEBIBYTE, |_| {
            ControlFlow::Continue(())
        })
        .unwrap();
        let nene_ledger_scan::index_outcome::IndexOutcome::Finished(index) = outcome else {
            panic!("止まった");
        };
        IndexedSource::new(source, index, format)
    }

    /// 索引が数えたレコードを全部、どの塊の大きさでも畳み込み、進み具合もその数で終わる。
    /// 塊ごとに原本を読み直すので、レコードの多いファイルでは小さな塊を使わない。
    fn assert_folds_every_record(from: &IndexedSource, sizes: &[usize]) {
        let total = from.index().records();
        let range = RecordRange::new(0, usize::try_from(total).unwrap());
        for &size in sizes {
            assert_chunks_match(from, range, size);
            assert_eq!(whole(from, range).len(), range.count(), "塊 {size}");
        }
    }

    #[test]
    fn the_records_after_a_long_closed_cell_are_folded() {
        // 2 MiB の閉じた引用のセルの後ろに 8 レコード（GitHub Issue #13 の 1。直す前は 2 件で「数え終えた」）。
        let mut input = b"h1,h2\n1,\"".to_vec();
        input.extend(core::iter::repeat_n(b'z', 2 << 20_u32));
        input.extend(b"\"\n");
        for row in 2..10_u32 {
            input.extend(format!("{row},x\n").as_bytes());
        }
        let from = indexed("long-closed-cell", &input, Dialect::rfc4180());
        assert_eq!(from.index().records(), 10);
        assert_folds_every_record(&from, &[1, 3, 1024]);
    }

    #[test]
    fn a_long_record_between_short_ones_does_not_end_the_fold() {
        // 見出し・短い 5 行・3 MiB の 1 行・短い 4 行（11 レコード。直す前は 6 件で「数え終えた」）。
        let mut input = b"id,v\n".to_vec();
        for row in 0..5_u32 {
            input.extend(format!("{row},a\n").as_bytes());
        }
        input.extend(b"5,");
        input.extend(core::iter::repeat_n(b'w', 3 << 20_u32));
        input.extend(b"\n");
        for row in 6..10_u32 {
            input.extend(format!("{row},b\n").as_bytes());
        }
        let from = indexed("long-unquoted", &input, Dialect::rfc4180());
        assert_eq!(from.index().records(), 11);
        assert_folds_every_record(&from, &[1, 2, 4096]);
    }

    #[test]
    fn the_records_after_a_long_literal_quote_are_folded() {
        // `Literal` で、1 MiB を超えて閉じない引用符の後ろ（GitHub Issue #13 の 2）。
        let mut input = b"a\n\"b\n".to_vec();
        input.extend(b"x,y\n".repeat(300_000));
        let literal = Dialect::rfc4180().with_unclosed_quote(UnclosedQuote::Literal);
        let from = indexed("long-literal", &input, literal);
        assert_eq!(from.index().records(), 300_002);
        assert_folds_every_record(&from, &[4096, 100_000]);
    }
}
