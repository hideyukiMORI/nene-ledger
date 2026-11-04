//! 検査対象のファイル 1 つ。

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// 読み込み済みの検査対象ファイル。
///
/// 🔴 フィールドは非公開（RS-008）。パスは必ずリポジトリルートからの相対で持つ。
/// 絶対パスが違反メッセージに混ざると、実行した環境ごとに出力が変わる（RS-016 と同じ理由）。
#[derive(Debug, Clone)]
pub(crate) struct SourceFile {
    relative_path: String,
    text: String,
}

impl SourceFile {
    /// リポジトリルートからの相対パスと本文を保持する。
    pub(crate) fn new(relative_path: String, text: String) -> Self {
        Self {
            relative_path,
            text,
        }
    }

    /// リポジトリルートからの相対パス。
    pub(crate) fn path(&self) -> &str {
        &self.relative_path
    }

    /// 拡張子が一致するか調べる。
    pub(crate) fn has_extension(&self, extension: &str) -> bool {
        Path::new(&self.relative_path)
            .extension()
            .is_some_and(|found| found == extension)
    }

    /// ファイル名そのものが一致するか調べる。
    pub(crate) fn is_named(&self, name: &str) -> bool {
        Path::new(&self.relative_path)
            .file_name()
            .is_some_and(|found| found == name)
    }

    /// 1 始まりの行番号を付けて本文を走査する。
    pub(crate) fn numbered_lines(&self) -> impl Iterator<Item = (usize, &str)> {
        self.text
            .lines()
            .enumerate()
            .map(|(index, line)| (index.saturating_add(1), line))
    }
}

/// 検査から外すディレクトリ。
///
/// `target` と `out` は生成物、`.git` は履歴で、どれも我々が書いたコードではない。
/// 🔴 `.github` は外さない。PR テンプレートの文書内リンクも検査対象である。
const SKIPPED_DIRECTORIES: [&str; 3] = ["target", "out", ".git"];

/// 拡張子を持たないが検査対象にするファイル名。
const EXAMINED_NAMES: [&str; 1] = ["Makefile"];

/// `root` 以下の検査対象ファイルを、パス順に集める。
///
/// 🔴 並び順を決定的にする（RS-016）。`read_dir` の順序は OS 依存なので、
/// 並べ替えないと違反の報告順が実行ごとに変わり、差分が取れなくなる。
pub(crate) fn collect(root: &Path) -> Result<Vec<SourceFile>, io::Error> {
    let mut found = Vec::new();
    collect_into(root, root, &mut found)?;
    found.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(found)
}

/// `directory` を再帰的に辿り、`found` に積む。
fn collect_into(
    root: &Path,
    directory: &Path,
    found: &mut Vec<SourceFile>,
) -> Result<(), io::Error> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        let name = path.file_name().and_then(|raw| raw.to_str()).unwrap_or("");
        if path.is_dir() {
            if !SKIPPED_DIRECTORIES.contains(&name) {
                collect_into(root, &path, found)?;
            }
        } else if is_examined(&path) {
            push_file(root, &path, found)?;
        }
    }
    Ok(())
}

/// 検査対象の拡張子か調べる。
fn is_examined(path: &Path) -> bool {
    let by_extension = path
        .extension()
        .and_then(|raw| raw.to_str())
        .is_some_and(|extension| matches!(extension, "rs" | "md" | "toml" | "yml" | "yaml"));
    let by_name = path
        .file_name()
        .and_then(|raw| raw.to_str())
        .is_some_and(|name| EXAMINED_NAMES.contains(&name));
    by_extension || by_name
}

/// 1 ファイルを読んで `found` に積む。
fn push_file(root: &Path, path: &Path, found: &mut Vec<SourceFile>) -> Result<(), io::Error> {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let Some(relative_path) = relative.to_str() else {
        return Ok(());
    };
    let text = fs::read_to_string(path)?;
    found.push(SourceFile::new(relative_path.replace('\\', "/"), text));
    Ok(())
}

/// パスの結合。文書内リンクの解決に使う。
pub(crate) fn resolve_sibling(from: &str, link: &str) -> PathBuf {
    Path::new(from)
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(link)
}
