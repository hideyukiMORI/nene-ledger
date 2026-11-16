//! 保存先を選ばせるときの、初めのフォルダ（ADR 0028 の決定 4）と、2 つの道筋が同じファイルかの見分け（ADR 0036 の決定 2）。

use std::path::{Path, PathBuf};

/// ファイル `current` のフォルダの絶対の道筋。ファイルが無ければ（新しい表）今の作業のフォルダ。解けなければ `.`。
#[must_use]
pub fn folder_of(current: Option<&Path>) -> PathBuf {
    let folder = current
        .and_then(Path::parent)
        .filter(|folder| !folder.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::canonicalize(folder).unwrap_or_else(|_error| folder.to_path_buf())
}

/// `target` が、今のファイル `current` と同じファイルか（道筋を解いて比べる。どちらかが無ければ同じでない）。
#[must_use]
pub fn same_file(current: Option<&Path>, target: &Path) -> bool {
    let resolved = |path: &Path| std::fs::canonicalize(path).ok();
    current
        .and_then(resolved)
        .is_some_and(|current| resolved(target) == Some(current))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_folder_is_absolute_and_falls_back_to_the_working_folder() {
        let here = std::fs::canonicalize(".").unwrap();
        assert_eq!(folder_of(None), here);
        assert_eq!(folder_of(Some(Path::new("a.csv"))), here);
        assert_eq!(folder_of(Some(Path::new("./a.csv"))), here);
        assert!(folder_of(Some(Path::new("/no/such/folder/a.csv"))).ends_with("folder"));
    }

    #[test]
    fn two_spellings_of_one_file_are_the_same_file() {
        let folder =
            std::env::temp_dir().join(format!("nene-ledger-store-locating-{}", std::process::id()));
        std::fs::create_dir_all(folder.join("inner")).unwrap();
        let file = folder.join("a.csv");
        std::fs::write(&file, b"a").unwrap();
        let dotted = folder.join("inner").join("..").join("a.csv");
        assert!(same_file(Some(&file), &dotted));
        assert!(!same_file(Some(&file), &folder.join("none.csv")));
        assert!(!same_file(None, &file));
    }
}
