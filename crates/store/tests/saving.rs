//! 保存の手順を、一時フォルダの実ファイルで確かめる（ADR 0025・SAVE-01・KEEP-02）。

// 🔴 中身を `#[cfg(test)]` の mod に入れる。補助の関数も clippy.toml の「テストは免除」に入る（RS-005）。
#[cfg(test)]
mod saving {
    use std::io::ErrorKind;
    use std::path::{Path, PathBuf};

    use nene_ledger_application::replace_failure::ReplaceFailure;
    use nene_ledger_application::replacing::Replacing;
    use nene_ledger_application::source_bytes::SourceBytes;
    use nene_ledger_core::save_plan::SavePlan;
    use nene_ledger_core::text_encoding::TextEncoding;
    use nene_ledger_store::rename_replacing::RenameReplacing;
    use nene_ledger_store::save_failure::SaveFailure;
    use nene_ledger_store::saving::{discard, retry_replace, save};
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
                stamp_of(target).map_err(SaveFailure::Target)?,
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
            (&plan, TextEncoding::Utf8WithBom, stamp_of(&target).unwrap()),
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
            (&plan, TextEncoding::Utf8, stamp_of(&target).unwrap()),
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
            (&plan, TextEncoding::ShiftJis, stamp_of(&target).unwrap()),
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
            (&whole(6), TextEncoding::Utf8, stamp_of(&target).unwrap()),
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
            (&plan, TextEncoding::Utf8, stamp_of(&target).unwrap()),
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
            (&whole(2), TextEncoding::Utf8, stamp_of(&target).unwrap()),
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
            (&plan, TextEncoding::Utf8, opened),
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
            (&whole(2), TextEncoding::Utf8, opened),
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
            (plan, TextEncoding::Utf8, stamp_of(&target).unwrap()),
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
            (plan, TextEncoding::Utf8, opened),
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
}
