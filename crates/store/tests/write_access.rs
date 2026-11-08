//! 開くときに、原本を保存で置き換えられるかを、一時フォルダの実ファイルで確かめる（FILE-04・GitHub Issue #240）。

// 🔴 中身を `#[cfg(test)]` の mod に入れる。補助の関数も clippy.toml の「テストは免除」に入る（RS-005）。
#[cfg(test)]
mod write_access {
    use std::path::PathBuf;

    use nene_ledger_application::replace_failure::ReplaceFailure;
    use nene_ledger_store::write_access::write_blocker;

    /// テストごとに別のファイル（テストは並列に走る）。
    fn file(name: &str) -> PathBuf {
        let folder =
            PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("write-access-{name}"));
        drop(std::fs::remove_dir_all(&folder));
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("table.csv");
        std::fs::write(&path, b"a,b\r\n1,2\r\n").unwrap();
        path
    }

    /// ふつうのファイルは、置き換えられる。
    #[test]
    fn a_plain_file_is_not_blocked() {
        assert_eq!(write_blocker(&file("plain")), None);
    }

    /// ほかのアプリが共有を許さずに開いている間はロック、閉じれば置き換えられる（Excel が開いているのと同じ形）。
    #[cfg(windows)]
    #[test]
    fn a_file_opened_without_sharing_is_locked_until_closed() {
        use std::os::windows::fs::OpenOptionsExt;
        let path = file("locked");
        let held = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(0)
            .open(&path)
            .unwrap();
        assert_eq!(write_blocker(&path), Some(ReplaceFailure::Locked));
        drop(held);
        assert_eq!(write_blocker(&path), None);
    }

    /// 書き込みの権限が誰にも無いファイルは、権限なし。戻せば置き換えられる。
    #[cfg(unix)]
    #[test]
    fn a_file_without_write_permission_is_denied() {
        use std::os::unix::fs::PermissionsExt;
        let path = file("denied");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o444)).unwrap();
        assert_eq!(write_blocker(&path), Some(ReplaceFailure::Denied));
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(write_blocker(&path), None);
    }
}
