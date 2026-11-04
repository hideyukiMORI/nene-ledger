//! CNF-014 — `unsafe` を許す 3 クレート（Windows の 2 つと scan）の lint の表は、workspace の表と同じで、違うのは `unsafe_code` だけ（ADR 0002 / ADR 0006）。
//!
//! Cargo は継承した lint の表の 1 項目だけを変えられないので、3 クレートは表を丸ごと写す。写しが workspace とずれると、
//! 「そのクレートだけ緩い」が黙って生まれる。それをここで拒む。

use std::collections::BTreeMap;

use crate::source_file::SourceFile;
use crate::violation::Violation;

/// workspace のマニフェスト。
const WORKSPACE_MANIFEST: &str = "Cargo.toml";
/// `unsafe` を許すクレートのマニフェスト。
const UNSAFE_CRATES: [&str; 3] = [
    "crates/adapters-windows/Cargo.toml",
    "crates/ui-win32/Cargo.toml",
    "crates/scan/Cargo.toml",
];
/// 違ってよい唯一の項目と、3 クレートでの値。
const UNSAFE_KEY: &str = "unsafe_code";
const UNSAFE_VALUE: &str = "\"deny\"";

/// `header` の節の `key = value` を、空白と注釈を除いて集める。
fn table(file: &SourceFile, header: &str) -> BTreeMap<String, String> {
    let mut inside = false;
    let mut entries = BTreeMap::new();
    for (_, line) in file.numbered_lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            inside = trimmed == header;
            continue;
        }
        if !inside || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            let value = value.split('#').next().unwrap_or_default();
            entries.insert(key.trim().to_owned(), value.trim().to_owned());
        }
    }
    entries
}

/// CNF-014 の検査。
pub(crate) fn unsafe_crates_mirror_workspace(files: &[SourceFile]) -> Vec<Violation> {
    let Some(workspace) = files.iter().find(|file| file.path() == WORKSPACE_MANIFEST) else {
        return Vec::new();
    };
    let mut expected_rust = table(workspace, "[workspace.lints.rust]");
    expected_rust.insert(UNSAFE_KEY.to_owned(), UNSAFE_VALUE.to_owned());
    let expected_clippy = table(workspace, "[workspace.lints.clippy]");
    files
        .iter()
        .filter(|file| UNSAFE_CRATES.contains(&file.path()))
        .flat_map(|file| {
            let mut found = Vec::new();
            for (header, expected) in [
                ("[lints.rust]", &expected_rust),
                ("[lints.clippy]", &expected_clippy),
            ] {
                if &table(file, header) != expected {
                    found.push(Violation::new(
                        "CNF-014",
                        file.path(),
                        1,
                        format!(
                            "{header} が workspace の表と違う。違ってよいのは unsafe_code = \"deny\" だけ（ADR 0002 / ADR 0006）"
                        ),
                    ));
                }
            }
            found
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(path: &str, text: &str) -> SourceFile {
        SourceFile::new(path.to_owned(), text.to_owned())
    }

    const WORKSPACE: &str = "[workspace.lints.rust]\nunsafe_code = \"forbid\"\nmissing_docs = \"deny\"\n\n[workspace.lints.clippy]\nunwrap_used = \"forbid\"\n";

    #[test]
    fn cnf014_accepts_a_faithful_copy_with_unsafe_denied() {
        let files = [
            fixture("Cargo.toml", WORKSPACE),
            fixture(
                "crates/ui-win32/Cargo.toml",
                "[lints.rust]\n# Win32 を呼ぶ\nunsafe_code = \"deny\"\nmissing_docs = \"deny\"\n\n[lints.clippy]\nunwrap_used = \"forbid\"\n",
            ),
        ];
        assert!(unsafe_crates_mirror_workspace(&files).is_empty());
    }

    #[test]
    fn cnf014_catches_a_loosened_or_missing_lint() {
        let files = [
            fixture("Cargo.toml", WORKSPACE),
            fixture(
                "crates/scan/Cargo.toml",
                "[lints.rust]\nunsafe_code = \"deny\"\n\n[lints.clippy]\nunwrap_used = \"deny\"\n",
            ),
        ];
        let found = unsafe_crates_mirror_workspace(&files);
        assert_eq!(found.len(), 2);
        assert!(found.iter().all(|violation| violation.rule() == "CNF-014"));
    }

    #[test]
    fn cnf014_ignores_other_crates() {
        let files = [
            fixture("Cargo.toml", WORKSPACE),
            fixture("crates/ui-x11/Cargo.toml", "[lints]\nworkspace = true\n"),
        ];
        assert!(unsafe_crates_mirror_workspace(&files).is_empty());
    }
}
