//! 保存の手順を、一時フォルダの実ファイルで確かめる（ADR 0025・SAVE-01・KEEP-02）。

// 🔴 中身を `#[cfg(test)]` の mod に入れる。補助の関数も clippy.toml の「テストは免除」に入る（RS-005）。
#[cfg(test)]
mod saving {
    use std::io::ErrorKind;
    use std::path::{Path, PathBuf};

    use nene_ledger_application::replace_failure::ReplaceFailure;
    use nene_ledger_application::replacing::Replacing;
    use nene_ledger_application::save_target::SaveTarget;
    use nene_ledger_application::source_bytes::SourceBytes;
    use nene_ledger_core::plan_sink::PlanSink;
    use nene_ledger_core::save_plan::SavePlan;
    use nene_ledger_core::text_encoding::TextEncoding;
    use nene_ledger_store::rename_replacing::RenameReplacing;
    use nene_ledger_store::save_failure::SaveFailure;
    use nene_ledger_store::saving::{discard, retry_replace, save, save_streamed};
    use nene_ledger_store::stamping::stamp_of;

    /// テストごとに別のフォルダ（テストは並列に走る）。
    fn folder(name: &str) -> PathBuf {
        let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("store-{name}"));
        drop(std::fs::remove_dir_all(&path));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    /// 記憶の中の原本。
    struct Bytes(Vec<u8>);

    impl SourceBytes for Bytes {
        type Failure = ErrorKind;

        fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> Result<usize, ErrorKind> {
            let start = usize::try_from(offset)
                .unwrap_or(usize::MAX)
                .min(self.0.len());
            let rest = &self.0[start..];
            let count = rest.len().min(buffer.len());
            buffer[..count].copy_from_slice(&rest[..count]);
            Ok(count)
        }
    }

    /// いつも失敗する置き換え。
    struct Refusing(ReplaceFailure);

    impl Replacing for Refusing {
        type Location = Path;

        fn replace(&self, _: &Path, _: &Path) -> Result<(), ReplaceFailure> {
            Err(self.0)
        }
    }

    fn whole(length: usize) -> SavePlan {
        let mut plan = SavePlan::empty();
        plan.copy(0, u64::try_from(length).unwrap());
        plan
    }

    fn leftovers(folder: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(folder)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| {
                Path::new(name)
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("tmp"))
            })
            .collect();
        names.sort();
        names
    }

    fn saved_text(target: &Path, text: &str, encoding: TextEncoding) -> Result<(), SaveFailure> {
        let mut plan = SavePlan::empty();
        plan.text(text);
        save(
            target,
            (
                &plan,
                encoding,
                SaveTarget::Existing(stamp_of(target).map_err(SaveFailure::Target)?),
            ),
            &mut Bytes(Vec::new()),
            &RenameReplacing,
        )
    }

    #[test]
    fn saving_without_edits_writes_the_same_bytes_and_leaves_nothing_behind() {
        let dir = folder("unchanged");
        let original = b"\xEF\xBB\xBFid,name\r\n0012,\"a\"\"b\"\r\nlast".to_vec();
        let target = dir.join("t.csv");
        std::fs::write(&target, &original).unwrap();
        let mut source = Bytes(original.clone());
        let plan = whole(original.len());
        save(
            &target,
            (
                &plan,
                TextEncoding::Utf8WithBom,
                SaveTarget::Existing(stamp_of(&target).unwrap()),
            ),
            &mut source,
            &RenameReplacing,
        )
        .unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), original);
        assert!(leftovers(&dir).is_empty());
    }

    #[test]
    fn copies_longer_than_one_chunk_are_written_whole() {
        let dir = folder("long");
        let original: Vec<u8> = (0..3_000_000_u32)
            .map(|n| b"abc,\n"[usize::try_from(n % 5).unwrap()])
            .collect();
        let target = dir.join("t.csv");
        std::fs::write(&target, &original).unwrap();
        let mut plan = SavePlan::empty();
        plan.copy(0, 1_500_000);
        plan.text("X");
        plan.copy(1_500_001, 1_499_999);
        save(
            &target,
            (
                &plan,
                TextEncoding::Utf8,
                SaveTarget::Existing(stamp_of(&target).unwrap()),
            ),
            &mut Bytes(original.clone()),
            &RenameReplacing,
        )
        .unwrap();
        let mut expected = original;
        expected[1_500_000] = b'X';
        assert_eq!(std::fs::read(&target).unwrap(), expected);
    }

    #[test]
    fn a_rewritten_cell_is_encoded_in_the_file_encoding() {
        let dir = folder("sjis");
        let original = b"a,\x93\xfa\r\nb,c\r\n".to_vec();
        let target = dir.join("t.csv");
        std::fs::write(&target, &original).unwrap();
        let mut plan = SavePlan::empty();
        plan.copy(0, 8);
        plan.text("本");
        plan.copy(9, 2);
        let mut source = Bytes(original);
        save(
            &target,
            (
                &plan,
                TextEncoding::ShiftJis,
                SaveTarget::Existing(stamp_of(&target).unwrap()),
            ),
            &mut source,
            &RenameReplacing,
        )
        .unwrap();
        assert_eq!(
            std::fs::read(&target).unwrap(),
            b"a,\x93\xfa\r\nb,\x96\x7b\r\n"
        );
    }

    #[test]
    fn an_unrepresentable_value_writes_nothing() {
        let dir = folder("unrepresentable");
        let target = dir.join("t.csv");
        std::fs::write(&target, b"a\n").unwrap();
        let failure = saved_text(&target, "😀", TextEncoding::ShiftJis);
        assert_eq!(
            failure,
            Err(SaveFailure::Unrepresentable(String::from("😀")))
        );
        assert_eq!(std::fs::read(&target).unwrap(), b"a\n");
        assert!(leftovers(&dir).is_empty());
    }

    #[test]
    fn a_shrunk_source_stops_the_save_and_removes_the_temporary_file() {
        let dir = folder("shrunk");
        let target = dir.join("t.csv");
        std::fs::write(&target, b"abcdef").unwrap();
        let mut source = Bytes(b"abc".to_vec());
        let failure = save(
            &target,
            (
                &whole(6),
                TextEncoding::Utf8,
                SaveTarget::Existing(stamp_of(&target).unwrap()),
            ),
            &mut source,
            &RenameReplacing,
        );
        assert_eq!(failure, Err(SaveFailure::SourceShrank));
        assert_eq!(std::fs::read(&target).unwrap(), b"abcdef");
        assert!(leftovers(&dir).is_empty());
    }

    #[test]
    fn a_failed_replacement_keeps_the_original_and_the_temporary_file() {
        let dir = folder("locked");
        let target = dir.join("t.csv");
        std::fs::write(&target, b"old\n").unwrap();
        let mut plan = SavePlan::empty();
        plan.text("new\n");
        let refusing = Refusing(ReplaceFailure::Locked);
        let failure = save(
            &target,
            (
                &plan,
                TextEncoding::Utf8,
                SaveTarget::Existing(stamp_of(&target).unwrap()),
            ),
            &mut Bytes(Vec::new()),
            &refusing,
        );
        let Err(SaveFailure::Replace {
            failure: ReplaceFailure::Locked,
            temporary,
        }) = failure
        else {
            panic!("{failure:?}");
        };
        assert_eq!(std::fs::read(&target).unwrap(), b"old\n");
        assert_eq!(std::fs::read(&temporary).unwrap(), b"new\n");
        assert_eq!(leftovers(&dir).len(), 1);
        retry_replace(&RenameReplacing, temporary, &target).unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"new\n");
        assert!(leftovers(&dir).is_empty());
        discard(&dir.join("absent.tmp"));
    }

    #[cfg(unix)]
    #[test]
    fn a_symbolic_link_saves_its_target_and_hard_links_are_refused() {
        let dir = folder("links");
        let real = dir.join("real.csv");
        std::fs::write(&real, b"x\n").unwrap();
        let link = dir.join("link.csv");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        saved_text(&link, "y\n", TextEncoding::Utf8).unwrap();
        assert!(
            std::fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(std::fs::read(&real).unwrap(), b"y\n");
        let hard = dir.join("hard.csv");
        std::fs::hard_link(&real, &hard).unwrap();
        let failure = saved_text(&hard, "z\n", TextEncoding::Utf8);
        assert_eq!(failure, Err(SaveFailure::HardLinked(2)));
        assert_eq!(std::fs::read(&real).unwrap(), b"y\n");
    }

    #[cfg(unix)]
    #[test]
    fn the_permissions_of_the_original_are_kept() {
        use std::os::unix::fs::PermissionsExt;
        let dir = folder("mode");
        let target = dir.join("t.csv");
        std::fs::write(&target, b"a\n").unwrap();
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o640)).unwrap();
        let mut source = Bytes(b"a\n".to_vec());
        save(
            &target,
            (
                &whole(2),
                TextEncoding::Utf8,
                SaveTarget::Existing(stamp_of(&target).unwrap()),
            ),
            &mut source,
            &RenameReplacing,
        )
        .unwrap();
        let mode = std::fs::metadata(&target).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o640);
    }

    #[test]
    fn a_file_changed_after_opening_is_not_overwritten() {
        let dir = folder("changed");
        let target = dir.join("t.csv");
        std::fs::write(&target, b"a,b\n").unwrap();
        let opened = stamp_of(&target).unwrap();
        std::fs::write(&target, b"a,b,c\n").unwrap();
        let mut plan = SavePlan::empty();
        plan.text("x\n");
        let failure = save(
            &target,
            (&plan, TextEncoding::Utf8, SaveTarget::Existing(opened)),
            &mut Bytes(Vec::new()),
            &RenameReplacing,
        );
        assert_eq!(failure, Err(SaveFailure::ChangedOutside));
        assert_eq!(std::fs::read(&target).unwrap(), b"a,b,c\n");
        assert!(leftovers(&dir).is_empty());
    }

    /// 書いている間に外から変えられたら、置き換えない（原本を読むポートの中で原本を書き換えて再現する）。
    struct Meddling {
        target: PathBuf,
        bytes: Vec<u8>,
    }

    impl SourceBytes for Meddling {
        type Failure = ErrorKind;

        fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> Result<usize, ErrorKind> {
            std::fs::write(&self.target, b"changed while saving\n")
                .map_err(|error| error.kind())?;
            Bytes(self.bytes.clone()).read_at(offset, buffer)
        }
    }

    #[test]
    fn a_change_while_writing_stops_before_the_replacement() {
        let dir = folder("meddled");
        let target = dir.join("t.csv");
        std::fs::write(&target, b"a\n").unwrap();
        let opened = stamp_of(&target).unwrap();
        let mut source = Meddling {
            target: target.clone(),
            bytes: b"a\n".to_vec(),
        };
        let failure = save(
            &target,
            (&whole(2), TextEncoding::Utf8, SaveTarget::Existing(opened)),
            &mut source,
            &RenameReplacing,
        );
        assert_eq!(failure, Err(SaveFailure::ChangedOutside));
        assert_eq!(std::fs::read(&target).unwrap(), b"changed while saving\n");
        assert!(leftovers(&dir).is_empty());
    }

    /// 起こされた回数を数える口。
    struct Counting(std::sync::atomic::AtomicUsize);

    impl nene_ledger_application::wakeup::Wakeup for Counting {
        fn wake(&self) {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }

    #[test]
    fn a_save_job_writes_in_the_background_and_wakes_the_window() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
        let dir = folder("job");
        let target = dir.join("t.csv");
        std::fs::write(&target, b"a,b\n").unwrap();
        let mut plan = SavePlan::empty();
        plan.copy(0, 2);
        plan.text("x\n");
        let wakeup = Arc::new(Counting(AtomicUsize::new(0)));
        let shared: Arc<dyn nene_ledger_application::wakeup::Wakeup> = wakeup.clone();
        let job = nene_ledger_store::save_job::SaveJob::spawn(
            target.clone(),
            (
                plan,
                TextEncoding::Utf8,
                SaveTarget::Existing(stamp_of(&target).unwrap()),
            ),
            (Bytes(b"a,b\n".to_vec()), RenameReplacing),
            shared,
        )
        .unwrap();
        assert_eq!(job.wait(), Ok(()));
        assert_eq!(std::fs::read(&target).unwrap(), b"a,x\n");
        assert_eq!(wakeup.0.load(Ordering::SeqCst), 1);
        assert!(leftovers(&dir).is_empty());
    }

    #[test]
    fn a_failed_save_job_reports_the_failure_once() {
        use std::sync::Arc;
        use std::sync::atomic::AtomicUsize;
        let dir = folder("job-failed");
        let target = dir.join("t.csv");
        std::fs::write(&target, b"a\n").unwrap();
        let opened = stamp_of(&target).unwrap();
        std::fs::write(&target, b"changed\n").unwrap();
        let mut plan = SavePlan::empty();
        plan.text("x\n");
        let mut job = nene_ledger_store::save_job::SaveJob::spawn(
            target.clone(),
            (plan, TextEncoding::Utf8, SaveTarget::Existing(opened)),
            (Bytes(Vec::new()), RenameReplacing),
            Arc::new(Counting(AtomicUsize::new(0))),
        )
        .unwrap();
        let outcome = loop {
            if let Some(outcome) = job.take_result() {
                break outcome;
            }
            std::thread::yield_now();
        };
        assert_eq!(outcome, Err(SaveFailure::ChangedOutside));
        assert_eq!(job.take_result(), None);
        assert_eq!(std::fs::read(&target).unwrap(), b"changed\n");
    }

    /// まだ無いファイルは、置き換えずに作る。元にした原本は変えない（名前を付けて保存・ADR 0028 の決定 5 と強制の 2 つ目）。
    #[test]
    fn saving_as_a_new_file_creates_it_and_leaves_the_original_alone() {
        let dir = folder("absent");
        let original = b"\xEF\xBB\xBFid,name\r\n1,a\r\n".to_vec();
        let source_path = dir.join("source.csv");
        std::fs::write(&source_path, &original).unwrap();
        let target = dir.join("copy.csv");
        let mut plan = whole(original.len());
        plan.text("2,b\r\n");
        save(
            &target,
            (&plan, TextEncoding::Utf8WithBom, SaveTarget::Absent),
            &mut Bytes(original.clone()),
            &Refusing(ReplaceFailure::Other),
        )
        .unwrap();
        let mut expected = original.clone();
        expected.extend_from_slice(b"2,b\r\n");
        assert_eq!(std::fs::read(&target).unwrap(), expected);
        assert_eq!(std::fs::read(&source_path).unwrap(), original);
        assert!(leftovers(&dir).is_empty());
    }

    /// まだ無いはずのファイルができていれば、書かない（上書きの確認を飛ばさない）。
    #[test]
    fn saving_as_a_new_file_refuses_a_file_that_appeared() {
        let dir = folder("appeared");
        let target = dir.join("t.csv");
        std::fs::write(&target, b"theirs").unwrap();
        let refused = save(
            &target,
            (&whole(2), TextEncoding::Utf8, SaveTarget::Absent),
            &mut Bytes(b"ab".to_vec()),
            &RenameReplacing,
        );
        assert_eq!(refused, Err(SaveFailure::ChangedOutside));
        assert_eq!(std::fs::read(&target).unwrap(), b"theirs");
        assert!(leftovers(&dir).is_empty());
        // 無いフォルダには作れない。
        let missing = save(
            &dir.join("no-such-folder").join("t.csv"),
            (&whole(2), TextEncoding::Utf8, SaveTarget::Absent),
            &mut Bytes(b"ab".to_vec()),
            &RenameReplacing,
        );
        assert_eq!(missing, Err(SaveFailure::Target(ErrorKind::NotFound)));
    }

    /// 書いている間に原本が外から変わったら、名前を付けて保存は書かない（GitHub Issue #126）。
    struct ChangedSource(Bytes);

    impl SourceBytes for ChangedSource {
        type Failure = ErrorKind;

        fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> Result<usize, ErrorKind> {
            self.0.read_at(offset, buffer)
        }

        fn unchanged(&mut self) -> bool {
            false
        }
    }

    #[test]
    fn a_source_changed_while_writing_writes_nothing() {
        let dir = folder("source-changed");
        let absent = dir.join("new.csv");
        let refused = save(
            &absent,
            (&whole(2), TextEncoding::Utf8, SaveTarget::Absent),
            &mut ChangedSource(Bytes(b"ab".to_vec())),
            &RenameReplacing,
        );
        assert_eq!(refused, Err(SaveFailure::ChangedOutside));
        assert!(!absent.exists());
        let existing = dir.join("other.csv");
        std::fs::write(&existing, b"theirs").unwrap();
        let kept = save(
            &existing,
            (
                &whole(2),
                TextEncoding::Utf8,
                SaveTarget::Existing(stamp_of(&existing).unwrap()),
            ),
            &mut ChangedSource(Bytes(b"ab".to_vec())),
            &RenameReplacing,
        );
        assert_eq!(kept, Err(SaveFailure::ChangedOutside));
        assert_eq!(std::fs::read(&existing).unwrap(), b"theirs");
        assert!(leftovers(&dir).is_empty());
    }

    /// 計画を記憶に持たずに書く経路は、小さな写す区間がばらばらに続いても（列を足し消しした表の形）、記憶に持つ計画と
    /// 同じバイト列を書く（GitHub Issue #130）。
    #[test]
    fn a_streamed_plan_writes_the_same_bytes_as_a_stored_one() {
        let dir = folder("streamed");
        let original: Vec<u8> = (0..3_000_000_u32)
            .map(|n| b"abcdefgh,\n"[usize::try_from(n % 10).unwrap()])
            .collect();
        let target = dir.join("t.csv");
        std::fs::write(&target, &original).unwrap();
        // 3 バイトずつ写し、間に区切りを足す（塊の境目もまたぐ）。最後に 1 つ前へ戻って写す。
        let mut plan = SavePlan::empty();
        let mut offset = 0_u64;
        while offset + 3 <= 2_500_000 {
            plan.copy(offset, 3);
            plan.text(";");
            offset += 7;
        }
        plan.copy(10, 5);
        let stored = dir.join("stored.csv");
        std::fs::write(&stored, b"").unwrap();
        save(
            &stored,
            (
                &plan,
                TextEncoding::Utf8,
                SaveTarget::Existing(stamp_of(&stored).unwrap()),
            ),
            &mut Bytes(original.clone()),
            &RenameReplacing,
        )
        .unwrap();
        save_streamed(
            &target,
            (
                TextEncoding::Utf8,
                SaveTarget::Existing(stamp_of(&target).unwrap()),
            ),
            (&mut Bytes(original.clone()), &RenameReplacing),
            |sink: &mut dyn PlanSink| {
                plan.replay(sink);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(
            std::fs::read(&target).unwrap(),
            std::fs::read(&stored).unwrap()
        );
        assert!(leftovers(&dir).is_empty());
    }

    /// 計画を作れなかった・原本が短かった・表せない字は、何も置き換えず一時ファイルを残さない。
    #[test]
    fn a_failing_stream_replaces_nothing() {
        let dir = folder("streamed-failures");
        let target = dir.join("t.csv");
        std::fs::write(&target, b"keep").unwrap();
        let stamp = || SaveTarget::Existing(stamp_of(&target).unwrap());
        let unplannable = save_streamed(
            &target,
            (TextEncoding::Utf8, stamp()),
            (&mut Bytes(b"keep".to_vec()), &RenameReplacing),
            |sink: &mut dyn PlanSink| {
                sink.text("partial");
                Err(SaveFailure::Unplannable(String::from("TooLong(3)")))
            },
        );
        assert_eq!(
            unplannable,
            Err(SaveFailure::Unplannable(String::from("TooLong(3)")))
        );
        let shrank = save_streamed(
            &target,
            (TextEncoding::Utf8, stamp()),
            (&mut Bytes(b"ke".to_vec()), &RenameReplacing),
            |sink: &mut dyn PlanSink| {
                sink.copy(0, 4);
                Ok(())
            },
        );
        assert_eq!(shrank, Err(SaveFailure::SourceShrank));
        let unrepresentable = save_streamed(
            &target,
            (TextEncoding::ShiftJis, stamp()),
            (&mut Bytes(Vec::new()), &RenameReplacing),
            |sink: &mut dyn PlanSink| {
                sink.text("a");
                sink.text("😀");
                assert!(sink.stopped());
                Ok(())
            },
        );
        assert_eq!(
            unrepresentable,
            Err(SaveFailure::Unrepresentable(String::from("😀")))
        );
        assert_eq!(std::fs::read(&target).unwrap(), b"keep");
        assert!(leftovers(&dir).is_empty());
    }

    /// 起こさない口（テスト）。
    struct Quiet;

    impl nene_ledger_application::wakeup::Wakeup for Quiet {
        fn wake(&self) {}
    }

    /// 書いている途中で保存をやめると、何も置き換えず一時ファイルを残さない（SAVE-02・ADR 0025 の施主の判断 4）。
    #[test]
    fn a_cancelled_save_job_leaves_the_file_alone() {
        use nene_ledger_store::save_job::SaveJob;
        use std::sync::Arc;
        let dir = folder("cancelled");
        let target = dir.join("t.csv");
        std::fs::write(&target, b"keep").unwrap();
        let (started, wait) = std::sync::mpsc::channel::<()>();
        let job = SaveJob::spawn_streamed(
            target.clone(),
            (
                TextEncoding::Utf8,
                SaveTarget::Existing(stamp_of(&target).unwrap()),
            ),
            (
                Bytes(Vec::new()),
                RenameReplacing,
                move |sink: &mut dyn PlanSink| {
                    started.send(()).unwrap();
                    // 止められるまで書き続ける（止められなければ 10 秒で終える）。
                    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
                    while !sink.stopped() && std::time::Instant::now() < deadline {
                        sink.text("row,row,row\n");
                    }
                    Ok(())
                },
            ),
            Arc::new(Quiet),
        )
        .unwrap();
        wait.recv().unwrap();
        while job.written() == 0 {
            std::thread::yield_now();
        }
        job.cancel();
        assert_eq!(job.wait(), Err(SaveFailure::Cancelled));
        assert_eq!(std::fs::read(&target).unwrap(), b"keep");
        assert!(leftovers(&dir).is_empty());
    }
}
