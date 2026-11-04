//! 裏のスレッドの索引の仕事。

// 🔴 中身を `#[cfg(test)]` の mod に入れる（reading.rs と同じ理由）。
#[cfg(test)]
mod background {
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use nene_ledger_application::wakeup::Wakeup;
    use nene_ledger_core::dialect::Dialect;
    use nene_ledger_core::records::Records;
    use nene_ledger_core::text_encoding::TextEncoding;
    use nene_ledger_core::text_format::TextFormat;
    use nene_ledger_scan::block_size::BlockSize;
    use nene_ledger_scan::index_job::IndexJob;
    use nene_ledger_scan::index_outcome::IndexOutcome;
    use nene_ledger_scan::source::Source;

    /// 起こされた回数を数える。
    #[derive(Debug, Default)]
    struct Counting(AtomicUsize);

    impl Wakeup for Counting {
        fn wake(&self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn fixture(name: &str, contents: &[u8]) -> Source {
        let path =
            PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("background-{name}.csv"));
        std::fs::write(&path, contents).unwrap();
        Source::open(&path).unwrap()
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

    fn records(outcome: IndexOutcome) -> u64 {
        match outcome {
            IndexOutcome::Finished(index) => index.records(),
            IndexOutcome::Cancelled => panic!("止まった"),
        }
    }

    #[test]
    fn runs_to_the_end_and_wakes_the_screen() {
        let input = sample(5000);
        let counting = Arc::new(Counting::default());
        let job = IndexJob::spawn(
            fixture("wait", &input),
            utf8(),
            BlockSize::new(4096).unwrap(),
            counting.clone(),
        )
        .unwrap();
        let expected = Records::new(&input, Dialect::rfc4180()).count();
        assert_eq!(
            records(job.wait().unwrap()),
            u64::try_from(expected).unwrap()
        );
        // 塊ごとに 1 回 ＋ 終わりに 1 回。
        let blocks = input.len().div_ceil(4096);
        assert_eq!(counting.0.load(Ordering::SeqCst), blocks + 1);
    }

    #[test]
    fn the_result_is_taken_once_when_finished() {
        let source = fixture("take", &sample(100));
        let total = source.len();
        let mut job = IndexJob::spawn(
            source,
            utf8(),
            BlockSize::MEBIBYTE,
            Arc::new(Counting::default()),
        )
        .unwrap();
        let outcome = loop {
            if let Some(outcome) = job.take_result() {
                break outcome;
            }
            std::thread::yield_now();
        };
        assert_eq!(records(outcome.unwrap()), 100);
        assert!(job.take_result().is_none());
        assert_eq!(job.progress().done(), total);
    }

    #[test]
    fn dropping_a_running_job_cancels_and_joins() {
        let counting = Arc::new(Counting::default());
        let job = IndexJob::spawn(
            fixture("drop", &sample(200_000)),
            utf8(),
            BlockSize::new(64).unwrap(),
            counting.clone(),
        )
        .unwrap();
        job.cancel();
        drop(job);
        // 合流した後は、もう起こされない。
        let after = counting.0.load(Ordering::SeqCst);
        std::thread::yield_now();
        assert_eq!(counting.0.load(Ordering::SeqCst), after);
    }
}
