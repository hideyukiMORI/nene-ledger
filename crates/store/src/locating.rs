//! 保存先を選ばせるときの、初めのフォルダ（ADR 0028 の決定 4）。

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
}
