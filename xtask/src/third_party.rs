//! 配る exe に入る外部クレートの著作権表示の一覧（ADR 0032 の決定 6・GitHub Issue #181）。
//!
//! 🔴 一覧は `Cargo.lock` から作り直せること。手で書き足さない。依存を変えたら `xtask notices` で作り直し、
//! `xtask notices-check` が食い違いを落とす。

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 配る exe の組み立て先（ADR 0032 の決定 2・Makefile の `WINDOWS_TARGET`）。
const TARGET: &str = "x86_64-pc-windows-gnu";
/// 一覧のファイル（リポジトリの直下・zip にそのまま入れる）。
pub(crate) const FILE: &str = "THIRD-PARTY-NOTICES.txt";

const HEADER: &str = "NeNe Ledger が使っている外部のクレートの著作権表示とライセンスの本文です。\n\
NeNe Ledger 自身のライセンスは同梱の LICENSE を見てください。\n\
複数のライセンスから選べるクレートは MIT を選んでいます。\n\
\n\
Copyright notices and license texts of the third-party crates used by NeNe Ledger.\n\
For NeNe Ledger's own license, see the bundled LICENSE file.\n\
Where a crate offers a choice of licenses, the MIT license is chosen.\n\
\n\
このファイルは `cargo run -p xtask -- notices` で Cargo.lock から作り直す（手で書き足さない）。\n";

/// 一覧の全文を作る。クレートは名前の順。
pub(crate) fn render(root: &Path) -> Result<String, String> {
    let crates = linked_crates(root)?;
    let registry = registry_sources()?;
    let mut text = String::from(HEADER);
    for (name, version) in &crates {
        let directory = find_source(&registry, name, version)?;
        text.push_str(&section(name, version, &directory)?);
    }
    Ok(text)
}

/// 作り直してファイルに書く。書いたクレートの数の報告を返す。
pub(crate) fn write(root: &Path) -> Result<Vec<String>, String> {
    let text = render(root)?;
    let count = text.matches("\n==== ").count();
    fs::write(root.join(FILE), text).map_err(|error| format!("{FILE}: {error}"))?;
    Ok(vec![format!("{FILE} を作り直した（{count} クレート）")])
}

/// 作り直したものとファイルが同じかを見る。
pub(crate) fn check(root: &Path) -> Result<Vec<String>, String> {
    let wanted = render(root)?;
    let current = fs::read_to_string(root.join(FILE)).unwrap_or_default();
    if current == wanted {
        return Ok(vec![format!("{FILE} は Cargo.lock と一致する")]);
    }
    Err(format!(
        "ADR 0032: {FILE} が Cargo.lock の依存と食い違う。`cargo run -p xtask -- notices` で作り直す"
    ))
}

/// 配る exe に入る外部クレート（名前と版・重複なし・名前の順）。手続きマクロ（組み立て時だけ）も含める。
fn linked_crates(root: &Path) -> Result<Vec<(String, String)>, String> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| String::from("cargo"));
    let fetched = Command::new(&cargo)
        .current_dir(root)
        .args(["fetch", "--locked", "--target", TARGET])
        .status()
        .map_err(|error| format!("cargo fetch: {error}"))?;
    if !fetched.success() {
        return Err(String::from("cargo fetch が失敗した"));
    }
    let output = Command::new(&cargo)
        .current_dir(root)
        .args([
            "tree",
            "--locked",
            "--quiet",
            "--target",
            TARGET,
            "-e",
            "normal",
            "-p",
            "nene-ledger",
            "--prefix",
            "none",
        ])
        .output()
        .map_err(|error| format!("cargo tree: {error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    let listing = String::from_utf8_lossy(&output.stdout);
    let mut crates: Vec<(String, String)> = listing
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let name = parts.next()?;
            let version = parts.next()?.strip_prefix('v')?;
            // 自分のクレートは path の依存（括弧の中に道筋）で、外部ではない。
            (!name.starts_with("nene-ledger")).then(|| (name.to_owned(), version.to_owned()))
        })
        .collect();
    crates.sort();
    crates.dedup();
    Ok(crates)
}

/// crates.io の展開先（`$CARGO_HOME/registry/src/*`）。
fn registry_sources() -> Result<Vec<PathBuf>, String> {
    let home = env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo")))
        .ok_or_else(|| String::from("CARGO_HOME も HOME も無い"))?;
    let source = home.join("registry").join("src");
    let entries =
        fs::read_dir(&source).map_err(|error| format!("{}: {error}", source.display()))?;
    Ok(entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect())
}

/// クレート `name` の版 `version` の展開先。
fn find_source(registry: &[PathBuf], name: &str, version: &str) -> Result<PathBuf, String> {
    registry
        .iter()
        .map(|index| index.join(format!("{name}-{version}")))
        .find(|directory| directory.is_dir())
        .ok_or_else(|| format!("{name} {version} のソースが crates.io の展開先に無い"))
}

/// 1 クレートの節。ライセンスの式と、配布元の著作権表示（`COPYRIGHT`）・MIT の本文・式にあれば BSD-3-Clause と
/// Unicode-3.0 の本文。
fn section(name: &str, version: &str, directory: &Path) -> Result<String, String> {
    let manifest = fs::read_to_string(directory.join("Cargo.toml"))
        .map_err(|error| format!("{name}: {error}"))?;
    let license = manifest
        .lines()
        .find_map(|line| line.strip_prefix("license = "))
        .map(|value| value.trim().trim_matches('"').to_owned())
        .ok_or_else(|| format!("{name} {version} の Cargo.toml に license が無い"))?;
    let files = license_files(directory);
    let pick = |wanted: &[&str]| -> Option<&PathBuf> {
        files.iter().find(|path| {
            path.file_name()
                .and_then(|file| file.to_str())
                .is_some_and(|file| wanted.contains(&file.to_ascii_uppercase().as_str()))
        })
    };
    let mut parts: Vec<&PathBuf> = Vec::new();
    if let Some(copyright) = pick(&["COPYRIGHT"]) {
        parts.push(copyright);
    }
    let mit = pick(&["LICENSE-MIT", "LICENSE-MIT.TXT", "LICENSE-MIT.MD"])
        .ok_or_else(|| format!("{name} {version} に MIT の本文が無い（式: {license}）"))?;
    parts.push(mit);
    if license.contains("BSD-3-Clause") {
        parts.push(
            pick(&["LICENSE-WHATWG"])
                .ok_or_else(|| format!("{name}: BSD-3-Clause の本文が無い"))?,
        );
    }
    if license.contains("Unicode-3.0") {
        parts.push(
            pick(&["LICENSE-UNICODE"])
                .ok_or_else(|| format!("{name}: Unicode-3.0 の本文が無い"))?,
        );
    }
    let mut text = format!("\n==== {name} {version} ====\nLicense: {license}\n");
    for path in parts {
        let body =
            fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
        let file = path
            .file_name()
            .and_then(|file| file.to_str())
            .unwrap_or("");
        writeln!(text, "\n---- {file} ----\n{}", body.trim_end())
            .map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(text)
}

/// 展開先の直下の、ライセンスと著作権表示のファイル（名前の順）。
fn license_files(directory: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(directory)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.is_file())
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    files
}
