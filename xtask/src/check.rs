//! `docs/CODING_RULES.md` と憲章の CNF-0xx を実際に見る検査（ソース単位）。
//!
//! 🔴 検査対象は **本番コードだけ** である。`.rs` ファイルは、桁 0 の
//! `#[cfg(test)]` が現れた行より後を見ない。テストは意図的な違反を書く場所であり、
//! そこを検査すると「検査器のテストが検査器に落とされる」ことになる。
//!
//! 🔴 検出語は `concat!` で分割して書く。検査語をそのまま書くと、検査器が自分自身を
//! 違反として報告する（xi-tools の初版で 7 件の自己検出が出た）。ファイル単位の除外で
//! 黙らせると、そのファイルだけ他の CNF も効かなくなるので採らない。

use std::collections::BTreeSet;
use std::path::Path;

use crate::source_file::SourceFile;
use crate::violation::Violation;

/// 本番コードの行だけを、1 始まりの行番号付きで返す。
///
/// 🔴 終わりは「桁 0 の `#[cfg(test)]` の次の行が `mod` で始まる所」。`#[cfg(test)] extern crate std;` のような
/// 1 項目だけの印で止まると、`lib.rs` の残り全部が検査から外れる（NeNeDocket Issue #4 で実ファイルで見つけた穴）。
fn production_lines(file: &SourceFile) -> impl Iterator<Item = (usize, &str)> {
    let end = test_module_start(file).unwrap_or(usize::MAX);
    file.numbered_lines()
        .take_while(move |&(number, _)| number < end)
}

/// テストのモジュールの `#[cfg(test)]` の行番号。無ければ `None`。
fn test_module_start(file: &SourceFile) -> Option<usize> {
    let mut lines = file.numbered_lines().peekable();
    while let Some((number, line)) = lines.next() {
        if line.trim_end() != "#[cfg(test)]" {
            continue;
        }
        let next_is_module = lines.peek().is_some_and(|&(_, next)| {
            next.starts_with("mod ") || next.starts_with("pub(crate) mod ")
        });
        if next_is_module {
            return Some(number);
        }
    }
    None
}

/// CNF-001 — `Default` はファクトリを迂回する門なので、実装も derive も書かせない（RS-003）。
pub(crate) fn no_default_construction(file: &SourceFile) -> Vec<Violation> {
    if !file.has_extension("rs") {
        return Vec::new();
    }
    let mut found = Vec::new();
    for (number, line) in production_lines(file) {
        let trimmed = line.trim();
        let problem = if trimmed.starts_with("#[derive(") && trimmed.contains("Default") {
            Some("Default を derive している")
        } else if trimmed.starts_with("impl Default for") {
            Some("Default を実装している")
        } else if trimmed.contains(concat!("..Default::", "default()")) {
            Some("Default で構造体の穴を埋めている（E0063 を無効化する書き方）")
        } else {
            None
        };
        if let Some(message) = problem {
            found.push(Violation::new(
                "CNF-001",
                file.path(),
                number,
                format!("{message}。生成経路は唯一のファクトリに限る（RS-003）"),
            ));
        }
    }
    found
}

/// CNF-002 が禁じる構文と、その理由。
const FORBIDDEN_CONSTRUCTS: [(&str, &str); 7] = [
    (concat!("dyn ", "Any"), "型の代用に使われる（RS-006）"),
    (
        concat!("Once", "Lock"),
        "可変グローバルの遅延生成である（RS-009）",
    ),
    (
        concat!("Lazy", "Lock"),
        "可変グローバルの遅延生成である（RS-009）",
    ),
    (
        concat!("lazy_", "static"),
        "可変グローバルの遅延生成である（RS-009）",
    ),
    (
        concat!("proc-macro", " = true"),
        "手続きマクロは書かない（RS-010）",
    ),
    (
        concat!("Xlib", "Surface"),
        "描画は ImageSurface に完成させてから転送する。X の surface は unsafe を要する（ADR 0002）",
    ),
    (
        concat!("XCB", "Surface"),
        "描画は ImageSurface に完成させてから転送する。X の surface は unsafe を要する（ADR 0002）",
    ),
];

/// CNF-002 — 汎用データバッグ・遅延グローバル・言語マジックを禁じる。
pub(crate) fn no_forbidden_constructs(file: &SourceFile) -> Vec<Violation> {
    if !(file.has_extension("rs") || file.has_extension("toml")) {
        return Vec::new();
    }
    let mut found = Vec::new();
    for (number, line) in production_lines(file) {
        for &(needle, reason) in &FORBIDDEN_CONSTRUCTS {
            if line.contains(needle) {
                found.push(Violation::new(
                    "CNF-002",
                    file.path(),
                    number,
                    format!("`{needle}` を書いている。{reason}"),
                ));
            }
        }
    }
    found
}

/// 型を宣言するキーワード。桁 0 に現れたものだけを主要宣言として数える。
const TYPE_KEYWORDS: [&str; 4] = ["struct ", "enum ", "trait ", "union "];

/// 桁 0 の型宣言なら、その型名を返す。
fn declared_type_name(line: &str) -> Option<&str> {
    if line.starts_with(char::is_whitespace) {
        return None;
    }
    let body = line
        .strip_prefix("pub(crate) ")
        .or_else(|| line.strip_prefix("pub "))
        .unwrap_or(line);
    let rest = TYPE_KEYWORDS
        .iter()
        .find_map(|keyword| body.strip_prefix(keyword))?;
    let name = rest
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .next()?;
    (!name.is_empty()).then_some(name)
}

/// CNF-003 — 1 ファイル 1 主要宣言。寄せ集めのファイルを作らせない（RS-012）。
pub(crate) fn one_primary_declaration(file: &SourceFile) -> Vec<Violation> {
    if !file.has_extension("rs") {
        return Vec::new();
    }
    let declared: Vec<(usize, &str)> = production_lines(file)
        .filter_map(|(number, line)| declared_type_name(line).map(|name| (number, name)))
        .collect();
    let Some(&(_, first)) = declared.first() else {
        return Vec::new();
    };
    declared
        .iter()
        .skip(1)
        .map(|&(number, name)| {
            Violation::new(
                "CNF-003",
                file.path(),
                number,
                format!(
                    "`{name}` は 2 つ目の主要宣言（1 つ目は `{first}`）。ファイルを分ける（RS-012）"
                ),
            )
        })
        .collect()
}

/// 常に禁止する型名の語尾。文脈次第で妥当な語（`Processor` 等）は入れない。
const FORBIDDEN_TYPE_SUFFIXES: [&str; 5] = ["Manager", "Helper", "Util", "Utils", "Common"];

/// 常に禁止するモジュール名。
const FORBIDDEN_MODULE_NAMES: [&str; 5] = ["utils", "helpers", "managers", "misc", "common"];

/// CNF-004 — 役割を語らない名前を拒む（RS-013）。
pub(crate) fn role_bearing_names(file: &SourceFile) -> Vec<Violation> {
    if !file.has_extension("rs") {
        return Vec::new();
    }
    let mut found: Vec<Violation> = production_lines(file)
        .filter_map(|(number, line)| {
            let name = declared_type_name(line)?;
            let suffix = FORBIDDEN_TYPE_SUFFIXES
                .iter()
                .find(|candidate| name.ends_with(*candidate))?;
            Some(Violation::new(
                "CNF-004",
                file.path(),
                number,
                format!("型名 `{name}` の語尾 `{suffix}` は役割を語らない（RS-013）"),
            ))
        })
        .collect();
    found.extend(forbidden_module_name(file));
    found
}

/// ファイル名がそのままモジュール名になるので、禁止名を弾く。
fn forbidden_module_name(file: &SourceFile) -> Option<Violation> {
    let stem = Path::new(file.path()).file_stem()?.to_str()?;
    FORBIDDEN_MODULE_NAMES.contains(&stem).then(|| {
        Violation::new(
            "CNF-004",
            file.path(),
            0,
            format!("モジュール名 `{stem}` は役割を語らない（RS-013）"),
        )
    })
}

/// CNF-006 — `#[expect]` の `reason` は、実在する規則 ID を引く（RS-019）。
pub(crate) fn suppression_cites_rule(
    file: &SourceFile,
    known: &BTreeSet<String>,
) -> Vec<Violation> {
    if !file.has_extension("rs") {
        return Vec::new();
    }
    let mut found = Vec::new();
    for (number, block) in expect_attributes(file) {
        let cited = block
            .split_once("reason = \"")
            .and_then(|(_, rest)| rest.split_once(':'))
            .map(|(id, _)| id.trim().to_owned());
        match cited {
            Some(id) if known.contains(&id) => {}
            Some(id) => found.push(Violation::new(
                "CNF-006",
                file.path(),
                number,
                format!("`{id}` は規範文書に無い規則 ID である"),
            )),
            None => found.push(Violation::new(
                "CNF-006",
                file.path(),
                number,
                "reason が `<規則 ID>: <理由>` の形になっていない（RS-019）".to_owned(),
            )),
        }
    }
    found
}

/// `#[expect(...)]` を、複数行にまたがるものも含めて 1 件ずつ取り出す。
fn expect_attributes(file: &SourceFile) -> Vec<(usize, String)> {
    let mut found = Vec::new();
    let mut open: Option<(usize, String)> = None;
    for (number, line) in production_lines(file) {
        let trimmed = line.trim();
        match open.take() {
            Some((start, mut buffer)) => {
                buffer.push_str(trimmed);
                if trimmed.ends_with(")]") {
                    found.push((start, buffer));
                } else {
                    open = Some((start, buffer));
                }
            }
            None if trimmed.starts_with("#[expect(") => {
                if trimmed.ends_with(")]") {
                    found.push((number, trimmed.to_owned()));
                } else {
                    open = Some((number, trimmed.to_owned()));
                }
            }
            None => {}
        }
    }
    found
}

/// 中核クレートに要求する宣言。
const NO_STD_ATTRIBUTE: &str = "#![no_std]";

/// マニフェストの `[package] name`。
fn package_name(file: &SourceFile) -> Option<&str> {
    file.numbered_lines()
        .find_map(|(_, line)| line.strip_prefix("name = \""))
        .and_then(|rest| rest.split_once('"'))
        .map(|(name, _)| name)
}

/// 桁 0 の**最初の属性行群**に `#![no_std]` があるか調べる。
fn declares_no_std(file: &SourceFile) -> bool {
    file.numbered_lines()
        .map(|(_, line)| line.trim_end())
        .take_while(|line| line.is_empty() || line.starts_with("//") || line.starts_with("#!["))
        .any(|line| line == NO_STD_ATTRIBUTE)
}

/// CNF-007 — `-core` / `-application` で終わるクレートは `#![no_std]` を宣言する（ARC-003）。
///
/// 🔑 `no_std` にすると `std::fs` / `std::env` / `std::time` が lint 違反ではなく
/// **名前解決エラー**になる。この宣言が消えた瞬間、環境への到達不能性が黙って失われる。
pub(crate) fn core_crates_declare_no_std(files: &[SourceFile]) -> Vec<Violation> {
    files
        .iter()
        .filter_map(|manifest| {
            let name = package_name(manifest)?;
            let directory = manifest.path().strip_suffix("Cargo.toml")?;
            if !(name.ends_with("-core") || name.ends_with("-application")) {
                return None;
            }
            let library = format!("{directory}src/lib.rs");
            match files.iter().find(|file| file.path() == library) {
                Some(file) if declares_no_std(file) => None,
                Some(_) => Some(Violation::new(
                    "CNF-007",
                    &library,
                    0,
                    format!("`{name}` は中核クレートだが、先頭の属性に `{NO_STD_ATTRIBUTE}` が無い（ARC-003）"),
                )),
                None => Some(Violation::new(
                    "CNF-007",
                    manifest.path(),
                    0,
                    format!("`{name}` は中核クレートだが `{library}` が無い（ARC-003）"),
                )),
            }
        })
        .collect()
}

/// ビルド時にコードを生成する経路の名前。
const BUILD_SCRIPT: &str = "build.rs";

/// マニフェストからビルドスクリプトを指す書き方。
const BUILD_MANIFEST_KEY: &str = concat!("build", " = ");

/// CNF-008 — `build.rs` を置かない（RS-010）。例外は無い。要るなら ADR で 1 つだけ足し、ここで固定する。
pub(crate) fn no_build_script(file: &SourceFile) -> Vec<Violation> {
    if file.is_named(BUILD_SCRIPT) {
        return vec![Violation::new(
            "CNF-008",
            file.path(),
            0,
            "ビルドスクリプトである。ビルド時にコードを生成しない（RS-010）".to_owned(),
        )];
    }
    if !file.is_named("Cargo.toml") {
        return Vec::new();
    }
    production_lines(file)
        .filter(|&(_, line)| line.starts_with(BUILD_MANIFEST_KEY))
        .map(|(number, _)| {
            Violation::new(
                "CNF-008",
                file.path(),
                number,
                "マニフェストがビルドスクリプトを指している（RS-010）".to_owned(),
            )
        })
        .collect()
}

/// タスクの印。Issue 番号を伴わないものを拒む。
const TASK_MARKERS: [&str; 2] = [concat!("TO", "DO"), concat!("FIX", "ME")];

/// CNF-010 — タスクの印（`TASK_MARKERS`）は同じ行に Issue 番号を持つ（QLT-008）。
pub(crate) fn task_markers_cite_issue(file: &SourceFile) -> Vec<Violation> {
    let examined = file.has_extension("rs")
        || file.has_extension("toml")
        || file.has_extension("yml")
        || file.is_named("Makefile");
    if !examined {
        return Vec::new();
    }
    production_lines(file)
        .filter(|&(_, line)| TASK_MARKERS.iter().any(|marker| line.contains(marker)))
        .filter(|&(_, line)| !cites_issue(line))
        .map(|(number, _)| {
            Violation::new(
                "CNF-010",
                file.path(),
                number,
                "タスクの印には同じ行に Issue 番号（#N）が要る（QLT-008）".to_owned(),
            )
        })
        .collect()
}

/// `#N` の形の Issue 参照を含むか調べる。
fn cites_issue(line: &str) -> bool {
    line.split('#')
        .skip(1)
        .any(|rest| rest.starts_with(|c: char| c.is_ascii_digit() && c != '0'))
}

/// ゲートを無力化する設定の書き方。
const DISABLING_PATTERNS: [&str; 5] = [
    concat!("continue-on-error", ": true"),
    concat!("--cap-", "lints"),
    concat!("-A ", "warnings"),
    concat!("-A", "warnings"),
    concat!("= \"", "allow\""),
];

/// 置いてはいけない設定ファイル名の断片。
const FORBIDDEN_FILE_WORDS: [&str; 3] = ["baseline", "suppressions", "rustfmt.toml"];

/// CNF-011 — baseline とゲート無力化の禁止（QLT-003）。
pub(crate) fn no_gate_disabling(file: &SourceFile) -> Vec<Violation> {
    let mut found = Vec::new();
    let lower = Path::new(file.path())
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    if FORBIDDEN_FILE_WORDS.iter().any(|word| lower.contains(word)) {
        found.push(Violation::new(
            "CNF-011",
            file.path(),
            0,
            "baseline・抑制一覧・整形の別設定は置かない（QLT-003 / QLT-004）".to_owned(),
        ));
    }
    let examined =
        file.has_extension("toml") || file.has_extension("yml") || file.is_named("Makefile");
    if !examined {
        return found;
    }
    for (number, line) in file.numbered_lines() {
        if DISABLING_PATTERNS
            .iter()
            .any(|pattern| line.contains(pattern))
        {
            found.push(Violation::new(
                "CNF-011",
                file.path(),
                number,
                "ゲートを無力化する設定である（QLT-003）".to_owned(),
            ));
        }
    }
    found
}

/// 現在時刻・環境・ファイル・スレッド・乱数へ到達する名前（ARC-007）。
const NONDETERMINISTIC_NAMES: [&str; 9] = [
    "std::time",
    "SystemTime",
    "Instant",
    "std::env",
    "std::fs",
    "std::thread",
    "std::process::Command",
    "std::net",
    "rand::",
];

/// 決定性の禁止を適用しない製品モジュール（ARC-007）。OS の資源に触れるアダプタ 2 つと、原本を読む scan と、書き出しの store（ADR 0002 / 0006）。
const NONDETERMINISM_EXEMPT: [&str; 4] = [
    "crates/adapters-x11/",
    "crates/adapters-windows/",
    "crates/scan/",
    "crates/store/",
];

/// 合成ルート。コマンドライン引数だけは読める（ARC-006）。
const COMPOSITION_ROOT: &str = "crates/app/";

/// 合成ルートに許す唯一の入力。
const ARGUMENTS_INPUT: &str = "std::env::args";

/// CNF-012 — 製品クレートはアダプタ・scan・store の外で時刻・環境・ファイルへ到達しない（ARC-007）。
pub(crate) fn no_nondeterministic_input(file: &SourceFile) -> Vec<Violation> {
    let path = file.path();
    let product = path.starts_with("crates/") && file.has_extension("rs");
    if !product
        || NONDETERMINISM_EXEMPT
            .iter()
            .any(|exempt| path.starts_with(exempt))
    {
        return Vec::new();
    }
    let in_root = path.starts_with(COMPOSITION_ROOT);
    production_lines(file)
        .filter(|&(_, line)| !(in_root && line.contains(ARGUMENTS_INPUT)))
        .filter_map(|(number, line)| {
            let name = NONDETERMINISTIC_NAMES
                .iter()
                .find(|name| line.contains(*name))?;
            Some(Violation::new(
                "CNF-012",
                path,
                number,
                format!("`{name}` はアダプタ・scan・store の外で読まない（ARC-007）"),
            ))
        })
        .collect()
}

/// 書き込みで開く・作る API と、開かずに書く・消す・名前を変える `std::fs` の関数（ADR 0005）。
///
/// 🔴 `fs::write` は Issue #12 で足した。開き方だけを見ていたので、scan の本番のコードに書いても通った（gate-proofs 第 9 節）。
const WRITING_NAMES: [&str; 15] = [
    ".write(true",
    ".append(true",
    ".truncate(true",
    ".create(true",
    "File::create",
    concat!("Mmap", "Mut"),
    concat!("map_", "mut("),
    "fs::write(",
    "fs::remove_file(",
    "fs::remove_dir",
    "fs::rename(",
    "fs::copy(",
    "fs::create_dir",
    "fs::hard_link(",
    "fs::set_permissions(",
];

/// 原本を読む区画。
const READING_CRATE: &str = "crates/scan/";

/// CNF-015 — 読み取りの区画（scan）は書き込みで開かない。原本は保存の置き換えまで書かない（ARC-001 / ARC-005・ADR 0005）。
pub(crate) fn scan_does_not_write(file: &SourceFile) -> Vec<Violation> {
    let path = file.path();
    if !(path.starts_with(READING_CRATE) && file.has_extension("rs")) {
        return Vec::new();
    }
    production_lines(file)
        .filter_map(|(number, line)| {
            let name = WRITING_NAMES.iter().find(|name| line.contains(*name))?;
            Some(Violation::new(
                "CNF-015",
                path,
                number,
                format!("`{name}` は書き込みの経路である。scan は原本を読むだけ（ADR 0005）"),
            ))
        })
        .collect()
}

/// クレート間で許された依存（ARC-002）。鍵はディレクトリ名、値は依存してよいパッケージ名。
/// 🔴 外部クレートを足すのは ADR と deny.toml と、この表の仕事（ARC-013）。中核の外部クレートは scan と store の `encoding_rs`
/// （ADR 0015・Issue #18・ADR 0025・GitHub Issue #58）と、core の `regex-automata`（正規表現の照合・ADR 0031・GitHub Issue #171）。
const ALLOWED_DEPENDENCIES: [(&str, &[&str]); 10] = [
    ("crates/core/", &["regex-automata"]),
    ("crates/application/", &["nene-ledger-core"]),
    (
        "crates/scan/",
        &["nene-ledger-application", "nene-ledger-core", "encoding_rs"],
    ),
    (
        "crates/store/",
        &["nene-ledger-application", "nene-ledger-core", "encoding_rs"],
    ),
    (
        "crates/adapters-x11/",
        &["nene-ledger-application", "nene-ledger-core"],
    ),
    (
        "crates/adapters-windows/",
        &["nene-ledger-application", "nene-ledger-core"],
    ),
    (
        "crates/ui-x11/",
        &["nene-ledger-application", "nene-ledger-core"],
    ),
    (
        "crates/ui-win32/",
        &["nene-ledger-application", "nene-ledger-core"],
    ),
    (
        "crates/app/",
        &[
            "nene-ledger-scan",
            "nene-ledger-store",
            "nene-ledger-adapters-x11",
            "nene-ledger-adapters-windows",
            "nene-ledger-ui-x11",
            "nene-ledger-ui-win32",
            "nene-ledger-application",
            "nene-ledger-core",
        ],
    ),
    ("xtask/", &[]),
];

/// 外部クレートを表に書いたものだけに限るクレート（中核・scan・store・検査器）。他は ADR と deny.toml に任せる。
const LISTED_EXTERNAL_CRATES: [&str; 5] = [
    "crates/core/",
    "crates/application/",
    "crates/scan/",
    "crates/store/",
    "xtask/",
];

/// 内部クレートの接頭辞。
const INTERNAL_PREFIX: &str = "nene-ledger-";

/// マニフェストの `[dependencies]` 節と `[target.….dependencies]` 節にある依存名を列挙する。
/// 🔴 OS ごとの依存（ADR 0002）を素通りさせないため、target の節も読む。
fn dependency_names(file: &SourceFile) -> Vec<(usize, String)> {
    let mut inside = false;
    let mut found = Vec::new();
    for (number, line) in file.numbered_lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            inside = trimmed == "[dependencies]"
                || (trimmed.starts_with("[target.") && trimmed.ends_with(".dependencies]"));
            continue;
        }
        if !inside || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((name, _)) = trimmed.split_once('=') {
            found.push((number, name.trim().to_owned()));
        }
    }
    found
}

/// CNF-013 — クレート間の依存は許可された向きだけ。中核と検査器の外部クレートは表に書いたものだけ（ARC-002 / ARC-013）。
///
/// 内部クレートは表と照合する。外部クレートは中核（core / application）・scan・store・xtask では表に無ければ拒み、
/// 他では ADR と deny.toml に任せる。
pub(crate) fn dependencies_follow_graph(file: &SourceFile) -> Vec<Violation> {
    if !file.is_named("Cargo.toml") {
        return Vec::new();
    }
    let Some(&(directory, allowed)) = ALLOWED_DEPENDENCIES
        .iter()
        .find(|(directory, _)| file.path() == format!("{directory}Cargo.toml"))
    else {
        return Vec::new();
    };
    let listed = LISTED_EXTERNAL_CRATES.contains(&directory);
    dependency_names(file)
        .into_iter()
        .filter(|(_, name)| {
            let internal = name.starts_with(INTERNAL_PREFIX);
            let known = allowed.contains(&name.as_str());
            !known && (internal || listed)
        })
        .map(|(number, name)| {
            Violation::new(
                "CNF-013",
                file.path(),
                number,
                format!("`{name}` への依存は {directory} に許されていない（ARC-002 / ARC-013）"),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(path: &str, text: &str) -> SourceFile {
        SourceFile::new(path.to_owned(), text.to_owned())
    }

    fn rules(found: &[Violation]) -> Vec<&'static str> {
        found.iter().map(Violation::rule).collect()
    }

    // ── CNF-001 ────────────────────────────────────────────────────────────

    #[test]
    fn cnf001_catches_derive_default() {
        let file = fixture("a.rs", "#[derive(Debug, Default)]\nstruct A;\n");
        assert_eq!(no_default_construction(&file).len(), 1);
    }

    #[test]
    fn cnf001_catches_impl_default() {
        let file = fixture("a.rs", "impl Default for A {}\n");
        assert_eq!(no_default_construction(&file).len(), 1);
    }

    #[test]
    fn cnf001_catches_struct_update_syntax() {
        let file = fixture("a.rs", "let a = A { x: 1, ..Default::default() };\n");
        assert_eq!(no_default_construction(&file).len(), 1);
    }

    #[test]
    fn cnf001_ignores_clean_source() {
        let file = fixture("a.rs", "struct A;\nimpl A { fn new() -> Self { Self } }\n");
        assert!(no_default_construction(&file).is_empty());
    }

    #[test]
    fn cnf001_ignores_test_module() {
        let file = fixture(
            "a.rs",
            "struct A;\n#[cfg(test)]\nmod tests {\n    #[derive(Default)]\n    struct B;\n}\n",
        );
        assert!(no_default_construction(&file).is_empty());
    }

    #[test]
    fn cnf001_sees_past_a_single_item_cfg_test_but_not_the_test_module() {
        let file = fixture(
            "crates/core/src/lib.rs",
            "#![no_std]\n#[cfg(test)]\nextern crate std;\n#[derive(Default)]\nstruct A;\n#[cfg(test)]\nmod tests {\n    #[derive(Default)]\n    struct B;\n}\n",
        );
        assert_eq!(no_default_construction(&file).len(), 1);
    }

    #[test]
    fn cnf001_ignores_markdown() {
        let file = fixture("a.md", "#[derive(Default)]\n");
        assert!(no_default_construction(&file).is_empty());
    }

    // ── CNF-002 ────────────────────────────────────────────────────────────

    #[test]
    fn cnf002_catches_lazy_global() {
        let file = fixture("a.rs", "static X: OnceLock<u8> = OnceLock::new();\n");
        assert_eq!(no_forbidden_constructs(&file).len(), 1);
    }

    #[test]
    fn cnf002_catches_proc_macro_manifest() {
        let file = fixture("a/Cargo.toml", "[lib]\nproc-macro = true\n");
        assert_eq!(no_forbidden_constructs(&file).len(), 1);
    }

    #[test]
    fn cnf002_catches_x_surfaces() {
        let file = fixture(
            "crates/ui-x11/src/canvas.rs",
            "let s = cairo::XlibSurface::create(d, w, v, 1, 1);\nlet t = cairo::XCBSurface::create(c, d, v, 1, 1);\n",
        );
        assert_eq!(no_forbidden_constructs(&file).len(), 2);
    }

    #[test]
    fn cnf002_ignores_clean_source() {
        let file = fixture("a.rs", "fn f() -> u8 { 1_u8 }\n");
        assert!(no_forbidden_constructs(&file).is_empty());
    }

    // ── CNF-003 ────────────────────────────────────────────────────────────

    #[test]
    fn cnf003_catches_second_declaration() {
        let file = fixture("a.rs", "pub struct A;\nenum B { X }\n");
        assert_eq!(one_primary_declaration(&file).len(), 1);
    }

    #[test]
    fn cnf003_allows_single_declaration() {
        let file = fixture("a.rs", "pub struct A;\nimpl A {}\n");
        assert!(one_primary_declaration(&file).is_empty());
    }

    #[test]
    fn cnf003_ignores_nested_declaration() {
        let file = fixture("a.rs", "pub struct A;\nfn f() {\n    struct Local;\n}\n");
        assert!(one_primary_declaration(&file).is_empty());
    }

    // ── CNF-004 ────────────────────────────────────────────────────────────

    #[test]
    fn cnf004_catches_forbidden_suffix() {
        let file = fixture("a.rs", "pub struct RulerManager;\n");
        assert_eq!(role_bearing_names(&file).len(), 1);
    }

    #[test]
    fn cnf004_catches_forbidden_module_name() {
        let file = fixture("crates/core/src/utils.rs", "pub struct Length;\n");
        assert_eq!(role_bearing_names(&file).len(), 1);
    }

    #[test]
    fn cnf004_allows_context_dependent_name() {
        let file = fixture("a.rs", "pub struct TickProcessor;\n");
        assert!(role_bearing_names(&file).is_empty());
    }

    // ── CNF-006 ────────────────────────────────────────────────────────────

    fn known() -> BTreeSet<String> {
        ["RS-014".to_owned()].into_iter().collect()
    }

    #[test]
    fn cnf006_accepts_expect_citing_known_rule() {
        let file = fixture(
            "a.rs",
            "#[expect(clippy::print_stdout, reason = \"RS-014: 出力は 1 箇所\")]\nfn f() {}\n",
        );
        assert!(suppression_cites_rule(&file, &known()).is_empty());
    }

    #[test]
    fn cnf006_catches_unknown_rule_id() {
        let file = fixture(
            "a.rs",
            "#[expect(clippy::print_stdout, reason = \"RS-999: 無い規則\")]\nfn f() {}\n",
        );
        assert_eq!(suppression_cites_rule(&file, &known()).len(), 1);
    }

    #[test]
    fn cnf006_catches_reason_without_rule_id() {
        let file = fixture(
            "a.rs",
            "#[expect(clippy::print_stdout, reason = \"必要だから\")]\nfn f() {}\n",
        );
        assert_eq!(suppression_cites_rule(&file, &known()).len(), 1);
    }

    #[test]
    fn cnf006_reads_multiline_expect() {
        let file = fixture(
            "a.rs",
            "#[expect(\n    clippy::print_stdout,\n    reason = \"RS-999: 無い規則\"\n)]\nfn f() {}\n",
        );
        assert_eq!(suppression_cites_rule(&file, &known()).len(), 1);
    }

    // ── CNF-007 ────────────────────────────────────────────────────────────

    fn core_crate(library: &str) -> Vec<SourceFile> {
        vec![
            fixture(
                "crates/core/Cargo.toml",
                "[package]\nname = \"nene-ledger-core\"\n",
            ),
            fixture("crates/core/src/lib.rs", library),
        ]
    }

    #[test]
    fn cnf007_catches_core_crate_without_no_std() {
        let files = core_crate("//! 中核。\n\nextern crate alloc;\n");
        assert_eq!(core_crates_declare_no_std(&files).len(), 1);
    }

    #[test]
    fn cnf007_catches_no_std_after_real_code() {
        let files = core_crate("extern crate alloc;\n#![no_std]\n");
        assert_eq!(core_crates_declare_no_std(&files).len(), 1);
    }

    #[test]
    fn cnf007_allows_core_crate_with_no_std() {
        let files = core_crate("//! 中核。\n\n#![no_std]\n\nextern crate alloc;\n");
        assert!(core_crates_declare_no_std(&files).is_empty());
    }

    #[test]
    fn cnf007_requires_no_std_of_application_too() {
        let files = vec![
            fixture(
                "crates/application/Cargo.toml",
                "[package]\nname = \"nene-ledger-application\"\n",
            ),
            fixture("crates/application/src/lib.rs", "extern crate alloc;\n"),
        ];
        assert_eq!(core_crates_declare_no_std(&files).len(), 1);
    }

    #[test]
    fn cnf007_ignores_a_binary_crate() {
        let files = vec![
            fixture(
                "crates/app/Cargo.toml",
                "[package]\nname = \"nene-ledger\"\n",
            ),
            fixture("crates/app/src/main.rs", "fn main() {}\n"),
        ];
        assert!(core_crates_declare_no_std(&files).is_empty());
    }

    // ── CNF-008 ────────────────────────────────────────────────────────────

    #[test]
    fn cnf008_catches_a_build_script() {
        let file = fixture("crates/core/build.rs", "fn main() {}\n");
        assert_eq!(no_build_script(&file).len(), 1);
    }

    #[test]
    fn cnf008_catches_a_manifest_pointing_at_one() {
        let file = fixture(
            "a/Cargo.toml",
            "[package]\nname = \"a\"\nbuild = \"gen.rs\"\n",
        );
        assert_eq!(no_build_script(&file).len(), 1);
    }

    #[test]
    fn cnf008_catches_every_build_script() {
        // nene-ruler にあったアイコンの例外は持ち込まない。要るなら ADR で足す。
        for path in [
            "crates/app/build.rs",
            "crates/app/src/build.rs",
            "crates/ui-win32/build.rs",
            "build.rs",
        ] {
            let other = fixture(path, "fn main() {}\n");
            assert_eq!(no_build_script(&other).len(), 1, "{path}");
        }
    }

    #[test]
    fn cnf008_ignores_a_similar_file_name() {
        let file = fixture("a/src/rebuild.rs", "fn main() {}\n");
        assert!(no_build_script(&file).is_empty());
    }

    // ── CNF-010 ────────────────────────────────────────────────────────────

    #[test]
    fn cnf010_accepts_marker_with_issue() {
        let file = fixture("a.rs", "// TODO #12: 目盛の間隔\n");
        assert!(task_markers_cite_issue(&file).is_empty());
    }

    #[test]
    fn cnf010_catches_marker_without_issue() {
        let file = fixture("a.rs", "// FIXME: 目盛の間隔\n");
        assert_eq!(rules(&task_markers_cite_issue(&file)), vec!["CNF-010"]);
    }

    #[test]
    fn cnf010_ignores_markdown() {
        let file = fixture("docs/todo/current.md", "TODO を書く\n");
        assert!(task_markers_cite_issue(&file).is_empty());
    }

    // ── CNF-011 ────────────────────────────────────────────────────────────

    #[test]
    fn cnf011_catches_forbidden_filename() {
        let file = fixture("rustfmt.toml", "max_width = 100\n");
        assert_eq!(rules(&no_gate_disabling(&file)), vec!["CNF-011"]);
    }

    #[test]
    fn cnf011_catches_lint_set_to_allow() {
        let file = fixture(
            "Cargo.toml",
            "[workspace.lints.clippy]\nunwrap_used = \"allow\"\n",
        );
        assert_eq!(rules(&no_gate_disabling(&file)), vec!["CNF-011"]);
    }

    #[test]
    fn cnf011_catches_continue_on_error() {
        let file = fixture(
            ".github/workflows/check.yml",
            "    continue-on-error: true\n",
        );
        assert_eq!(rules(&no_gate_disabling(&file)), vec!["CNF-011"]);
    }

    #[test]
    fn cnf011_ignores_clean_configuration() {
        let file = fixture(
            "Cargo.toml",
            "[workspace.lints.clippy]\nunwrap_used = \"forbid\"\n",
        );
        assert!(no_gate_disabling(&file).is_empty());
    }

    // ── CNF-015 ────────────────────────────────────────────────────────────

    #[test]
    fn cnf015_catches_writing_in_scan() {
        for line in [
            "let f = OpenOptions::new().write(true).open(path)?;\n",
            "let f = File::create(path)?;\n",
            "let m = unsafe { MmapMut::map_mut(&file)? };\n",
            "std::fs::write(path, bytes)?;\n",
            "fs::remove_file(path)?;\n",
            "std::fs::remove_dir_all(path)?;\n",
            "fs::rename(from, to)?;\n",
            "std::fs::copy(from, to)?;\n",
            "fs::create_dir_all(path)?;\n",
            "fs::hard_link(from, to)?;\n",
            "fs::set_permissions(path, permissions)?;\n",
        ] {
            assert_eq!(
                rules(&scan_does_not_write(&fixture(
                    "crates/scan/src/source.rs",
                    line
                ))),
                vec!["CNF-015"],
                "{line}"
            );
        }
    }

    #[test]
    fn cnf015_allows_reading_in_scan_and_writing_in_store() {
        let reading = fixture(
            "crates/scan/src/source.rs",
            "let f = OpenOptions::new().read(true).open(path)?;\n",
        );
        assert!(scan_does_not_write(&reading).is_empty());
        let store = fixture("crates/store/src/save.rs", "let f = File::create(path)?;\n");
        assert!(scan_does_not_write(&store).is_empty());
        let replacing = fixture("crates/store/src/save.rs", "std::fs::rename(from, to)?;\n");
        assert!(scan_does_not_write(&replacing).is_empty());
        let metadata = fixture(
            "crates/scan/src/source.rs",
            "let len = std::fs::metadata(path)?.len();\n",
        );
        assert!(scan_does_not_write(&metadata).is_empty());
    }

    #[test]
    fn cnf015_ignores_tests_in_scan() {
        let file = fixture(
            "crates/scan/src/source.rs",
            "//! scan\n#[cfg(test)]\nmod tests {\n    let f = File::create(path);\n}\n",
        );
        assert!(scan_does_not_write(&file).is_empty());
    }

    // ── CNF-012 ────────────────────────────────────────────────────────────

    #[test]
    fn cnf012_catches_time_in_core() {
        let file = fixture(
            "crates/core/src/tick.rs",
            "let now = std::time::Instant::now();\n",
        );
        assert_eq!(rules(&no_nondeterministic_input(&file)), vec!["CNF-012"]);
    }

    #[test]
    fn cnf012_catches_env_in_ui() {
        let file = fixture(
            "crates/ui-x11/src/window.rs",
            "let home = std::env::var(\"HOME\");\n",
        );
        assert_eq!(no_nondeterministic_input(&file).len(), 1);
    }

    #[test]
    fn cnf012_allows_adapters() {
        let file = fixture(
            "crates/adapters-x11/src/clock.rs",
            "std::time::SystemTime::now();\n",
        );
        assert!(no_nondeterministic_input(&file).is_empty());
    }

    #[test]
    fn cnf012_allows_arguments_in_composition_root() {
        let file = fixture("crates/app/src/main.rs", "let args = std::env::args();\n");
        assert!(no_nondeterministic_input(&file).is_empty());
    }

    #[test]
    fn cnf012_catches_other_env_in_composition_root() {
        let file = fixture(
            "crates/app/src/main.rs",
            "let home = std::env::var(\"HOME\");\n",
        );
        assert_eq!(no_nondeterministic_input(&file).len(), 1);
    }

    #[test]
    fn cnf012_catches_files_and_commands_in_every_product_crate() {
        let line = "std::fs::write(out, bytes); std::process::Command::new(tool);\n";
        for path in [
            "crates/app/build.rs",
            "crates/app/src/main.rs",
            "crates/ui-win32/build.rs",
        ] {
            assert_eq!(
                no_nondeterministic_input(&fixture(path, line)).len(),
                1,
                "{path}"
            );
        }
    }

    #[test]
    fn cnf012_ignores_xtask() {
        let file = fixture("xtask/src/main.rs", "std::env::var(\"PR_TITLE\");\n");
        assert!(no_nondeterministic_input(&file).is_empty());
    }

    // ── CNF-013 ────────────────────────────────────────────────────────────

    #[test]
    fn cnf013_allows_the_documented_direction() {
        let file = fixture(
            "crates/ui-x11/Cargo.toml",
            "[dependencies]\nnene-ledger-core = { path = \"../core\" }\nx11rb = \"0.14\"\n",
        );
        assert!(dependencies_follow_graph(&file).is_empty());
    }

    #[test]
    fn cnf013_catches_ui_depending_on_adapters() {
        let file = fixture(
            "crates/ui-x11/Cargo.toml",
            "[dependencies]\nnene-ledger-adapters-x11 = { path = \"../adapters-x11\" }\n",
        );
        assert_eq!(rules(&dependencies_follow_graph(&file)), vec!["CNF-013"]);
    }

    #[test]
    fn cnf013_catches_external_crates_in_core_and_application_and_upward_dependencies() {
        let file = fixture("crates/core/Cargo.toml", "[dependencies]\nregex = \"1\"\n");
        assert_eq!(dependencies_follow_graph(&file).len(), 1);
        let listed = fixture(
            "crates/core/Cargo.toml",
            "[dependencies]\nnene-ledger-application = { path = \"../application\" }\n",
        );
        assert_eq!(rules(&dependencies_follow_graph(&listed)), vec!["CNF-013"]);
        let elsewhere = fixture(
            "crates/application/Cargo.toml",
            "[dependencies]\nregex = \"1\"\n",
        );
        assert_eq!(
            rules(&dependencies_follow_graph(&elsewhere)),
            vec!["CNF-013"]
        );
    }

    #[test]
    fn cnf013_allows_application_depending_on_core_only() {
        let file = fixture(
            "crates/application/Cargo.toml",
            "[package]\nname = \"nene-ledger-application\"\n\n[dependencies]\nnene-ledger-core = { path = \"../core\" }\n\n[lints]\nworkspace = true\n",
        );
        assert!(dependencies_follow_graph(&file).is_empty());
    }

    #[test]
    fn cnf013_catches_a_dependency_in_xtask() {
        let file = fixture("xtask/Cargo.toml", "[dependencies]\nserde = \"1\"\n");
        assert_eq!(dependencies_follow_graph(&file).len(), 1);
    }

    #[test]
    fn cnf012_allows_the_windows_adapters_the_scan_and_the_store() {
        for path in [
            "crates/adapters-windows/src/clipboard.rs",
            "crates/scan/src/source.rs",
            "crates/store/src/save.rs",
        ] {
            let file = fixture(path, "use std::env;\n");
            assert!(no_nondeterministic_input(&file).is_empty(), "{path}");
        }
    }

    #[test]
    fn cnf013_reads_target_specific_dependencies() {
        let file = fixture(
            "crates/ui-win32/Cargo.toml",
            "[target.'cfg(windows)'.dependencies]\nnene-ledger-adapters-windows = { path = \"../adapters-windows\" }\n",
        );
        assert_eq!(rules(&dependencies_follow_graph(&file)), vec!["CNF-013"]);
    }

    #[test]
    fn cnf013_allows_only_the_listed_external_crate_in_the_scan_and_the_store() {
        let listed = fixture(
            "crates/scan/Cargo.toml",
            "[dependencies]\nencoding_rs = { version = \"0.8.42\", default-features = false }\n",
        );
        assert!(dependencies_follow_graph(&listed).is_empty());
        // store も符号化に同じ crate を使う（ADR 0025 の決定 4）。ほかの区画では拒む。
        let store = fixture(
            "crates/store/Cargo.toml",
            "[dependencies]\nencoding_rs = \"0.8\"\n",
        );
        assert!(dependencies_follow_graph(&store).is_empty());
        let application = fixture(
            "crates/application/Cargo.toml",
            "[dependencies]\nencoding_rs = \"0.8\"\n",
        );
        assert_eq!(
            rules(&dependencies_follow_graph(&application)),
            vec!["CNF-013"]
        );
    }

    #[test]
    fn cnf013_keeps_the_scan_and_the_store_free_of_external_crates() {
        for path in ["crates/scan/Cargo.toml", "crates/store/Cargo.toml"] {
            let file = fixture(path, "[dependencies]\nserde = \"1\"\n");
            assert_eq!(
                rules(&dependencies_follow_graph(&file)),
                vec!["CNF-013"],
                "{path}"
            );
        }
    }

    #[test]
    fn cnf013_catches_scan_depending_on_store() {
        let file = fixture(
            "crates/scan/Cargo.toml",
            "[dependencies]\nnene-ledger-store = { path = \"../store\" }\n",
        );
        assert_eq!(rules(&dependencies_follow_graph(&file)), vec!["CNF-013"]);
    }
}
