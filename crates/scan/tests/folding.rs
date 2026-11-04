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
}
