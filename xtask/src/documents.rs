//! CNF-009 — 文書整合。規範文書と強制マトリクスと証明記録が互いに嘘をつかないことを見る。
//!
//! 「planned を active と書かない」（ADR 0001）を機械にする検査である。
//! 本文の `機械強制:` の状態、`docs/QUALITY_GATES.md` の強制マトリクス、
//! `docs/quality/gate-proofs.md` の証明行、規則 ID の参照、相対リンク、未置換の雛形値を照合する。

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::rule_id::{is_rule_id, rule_ids_in};
use crate::source_file::{self, SourceFile};
use crate::violation::Violation;

/// 規則の定義を持つ規範文書。
const NORMATIVE_FILES: [&str; 4] = [
    "docs/ARCHITECTURE_CONSTITUTION.md",
    "docs/CODING_RULES.md",
    "docs/COMMIT_CONVENTIONS.md",
    "docs/QUALITY_GATES.md",
];

/// 強制マトリクスを持つ文書。
const GATES_FILE: &str = "docs/QUALITY_GATES.md";

/// 証明記録。
const PROOFS_FILE: &str = "docs/quality/gate-proofs.md";

/// マトリクスの始まりと終わりの見出し。
const MATRIX_START: &str = "## 3. 強制マトリクス";
const MATRIX_END: &str = "## 4.";

/// 本文に書ける機械強制の状態。
const STATES: [&str; 4] = ["active", "planned", "不能", "不採用"];

/// 規範文書の `### ID` 見出しから、実在する規則 ID を集める。
pub(crate) fn known_rule_ids(files: &[SourceFile]) -> BTreeSet<String> {
    files
        .iter()
        .filter(|file| NORMATIVE_FILES.contains(&file.path()))
        .flat_map(SourceFile::numbered_lines)
        .filter_map(|(_, line)| heading_rule_id(line))
        .map(str::to_owned)
        .collect()
}

/// `### RS-001 — 題名` の見出しなら ID を返す。
fn heading_rule_id(line: &str) -> Option<&str> {
    let heading = line.strip_prefix("### ")?;
    let token = heading.split_whitespace().next()?;
    is_rule_id(token).then_some(token)
}

/// 本文の各規則について、最初に現れる `機械強制: **状態**` を集める。
fn defined_states(files: &[SourceFile]) -> (BTreeMap<String, String>, Vec<Violation>) {
    let mut states = BTreeMap::new();
    let mut found = Vec::new();
    for file in files.iter().filter(|f| NORMATIVE_FILES.contains(&f.path())) {
        found.extend(collect_states(file, &mut states));
    }
    for (id, state) in &states {
        if state.is_empty() {
            found.push(Violation::new(
                "CNF-009",
                "docs",
                0,
                format!("`{id}` に機械強制の状態が無い"),
            ));
        }
    }
    (states, found)
}

/// 1 つの規範文書から規則の見出しと状態を拾う。重複した定義は違反として返す。
fn collect_states(file: &SourceFile, states: &mut BTreeMap<String, String>) -> Vec<Violation> {
    let mut found = Vec::new();
    let mut current: Option<String> = None;
    for (number, line) in file.numbered_lines() {
        if let Some(id) = heading_rule_id(line) {
            if states.insert(id.to_owned(), String::new()).is_some() {
                found.push(Violation::new(
                    "CNF-009",
                    file.path(),
                    number,
                    format!("`{id}` の定義が重複している"),
                ));
            }
            current = Some(id.to_owned());
        } else if line.starts_with("## ") {
            current = None;
        } else {
            record_state(states, current.as_deref(), line);
        }
    }
    found
}

/// 現在の規則にまだ状態が無ければ、この行の状態を記録する。
fn record_state(states: &mut BTreeMap<String, String>, current: Option<&str>, line: &str) {
    let Some((id, state)) = current.zip(state_in(line)) else {
        return;
    };
    if states.get(id).is_some_and(String::is_empty) {
        states.insert(id.to_owned(), state.to_owned());
    }
}

/// `機械強制: **active**` の行から状態を取り出す。
fn state_in(line: &str) -> Option<&str> {
    let (_, rest) = line.split_once("機械強制:")?;
    let rest = rest.trim_start().strip_prefix("**")?;
    STATES.iter().copied().find(|state| rest.starts_with(state))
}

/// 強制マトリクスの行を `ID → 状態` に集める。重複行は違反にする。
fn matrix_rows(files: &[SourceFile]) -> (BTreeMap<String, String>, Vec<Violation>) {
    let mut rows = BTreeMap::new();
    let mut found = Vec::new();
    let Some(gates) = files.iter().find(|file| file.path() == GATES_FILE) else {
        return (
            rows,
            vec![Violation::new(
                "CNF-009",
                GATES_FILE,
                0,
                "強制マトリクスの文書が無い".to_owned(),
            )],
        );
    };
    let mut inside = false;
    for (number, line) in gates.numbered_lines() {
        if line.starts_with(MATRIX_START) {
            inside = true;
            continue;
        }
        if inside && line.starts_with(MATRIX_END) {
            break;
        }
        let Some((id, state)) = (inside).then(|| matrix_cells(line)).flatten() else {
            continue;
        };
        if rows.insert(id.to_owned(), state.to_owned()).is_some() {
            found.push(Violation::new(
                "CNF-009",
                GATES_FILE,
                number,
                format!("マトリクスの行 `{id}` が重複している"),
            ));
        }
    }
    (rows, found)
}

/// `| RS-001 | active | … |` の行から ID と状態を取り出す。
fn matrix_cells(line: &str) -> Option<(&str, &str)> {
    let mut cells = line.strip_prefix('|')?.split('|').map(str::trim);
    let id = cells.next()?;
    let state = cells.next()?;
    (is_rule_id(id) && STATES.contains(&state)).then_some((id, state))
}

/// 本文とマトリクスと証明記録を照合する。
fn consistency(files: &[SourceFile]) -> Vec<Violation> {
    let (states, mut found) = defined_states(files);
    let (rows, row_errors) = matrix_rows(files);
    found.extend(row_errors);
    let ids: BTreeSet<&String> = states.keys().chain(rows.keys()).collect();
    for id in ids {
        if states.get(id) != rows.get(id) {
            found.push(Violation::new(
                "CNF-009",
                GATES_FILE,
                0,
                format!("`{id}` の本文の状態とマトリクスの行が一致しない"),
            ));
        }
    }
    let proofs = files.iter().find(|file| file.path() == PROOFS_FILE);
    for (id, state) in &rows {
        if state != "active" {
            continue;
        }
        let proven = proofs.is_some_and(|file| {
            file.numbered_lines()
                .any(|(_, line)| line.starts_with(&format!("| {id} |")))
        });
        if !proven {
            found.push(Violation::new(
                "CNF-009",
                PROOFS_FILE,
                0,
                format!("`{id}` は active だが証明行が無い"),
            ));
        }
    }
    found
}

/// 全ての Markdown について、未定義の規則 ID・雛形値・切れたリンクを見る。
fn markdown_checks(files: &[SourceFile], root: &Path, known: &BTreeSet<String>) -> Vec<Violation> {
    files
        .iter()
        .filter(|file| file.has_extension("md"))
        .flat_map(|file| {
            file.numbered_lines()
                .flat_map(move |numbered| line_checks(file, numbered, root, known))
        })
        .collect()
}

/// Markdown の 1 行に対する検査。
fn line_checks(
    file: &SourceFile,
    (number, line): (usize, &str),
    root: &Path,
    known: &BTreeSet<String>,
) -> Vec<Violation> {
    let mut found = Vec::new();
    if line.contains("{{") && line.contains("}}") {
        found.push(Violation::new(
            "CNF-009",
            file.path(),
            number,
            "未置換の雛形値がある".to_owned(),
        ));
    }
    for id in rule_ids_in(line)
        .into_iter()
        .filter(|id| !known.contains(id))
    {
        found.push(Violation::new(
            "CNF-009",
            file.path(),
            number,
            format!("`{id}` はどの規範文書にも定義が無い"),
        ));
    }
    found.extend(broken_links(file, number, line, root));
    found
}

/// 1 行の相対リンクのうち、実在しないものを違反にする。
fn broken_links(file: &SourceFile, number: usize, line: &str, root: &Path) -> Vec<Violation> {
    markdown_links(line)
        .filter(|link| !(link.contains(':') || link.starts_with('#')))
        .map(|link| link.split('#').next().unwrap_or(link))
        .filter(|target| {
            !target.is_empty()
                && !root
                    .join(source_file::resolve_sibling(file.path(), target))
                    .exists()
        })
        .map(|target| {
            Violation::new(
                "CNF-009",
                file.path(),
                number,
                format!("リンク先 `{target}` が存在しない"),
            )
        })
        .collect()
}

/// 1 行から Markdown のリンク先を取り出す。
fn markdown_links(line: &str) -> impl Iterator<Item = &str> {
    line.split("](")
        .skip(1)
        .filter_map(|rest| rest.split_once(')'))
        .map(|(link, _)| link)
}

/// CNF-009 の全検査。
pub(crate) fn inspect(files: &[SourceFile], root: &Path) -> Vec<Violation> {
    let known = known_rule_ids(files);
    let mut found = consistency(files);
    found.extend(markdown_checks(files, root, &known));
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(path: &str, text: &str) -> SourceFile {
        SourceFile::new(path.to_owned(), text.to_owned())
    }

    fn missing_root() -> &'static Path {
        Path::new("/xtask-test-root-that-does-not-exist")
    }

    fn gates(body: &str, matrix: &str) -> SourceFile {
        fixture(
            GATES_FILE,
            &format!(
                "{body}\n\n## 3. 強制マトリクス\n\n| 規則 | 状態 | 実体 |\n| --- | --- | --- |\n{matrix}\n\n## 4. 層\n"
            ),
        )
    }

    fn consistent() -> Vec<SourceFile> {
        vec![
            gates(
                "### CNF-009 — 文書整合\n\n- 機械強制: **active**（xtask）\n\n### QLT-001 — 一つ\n\n- 機械強制: **planned**",
                "| CNF-009 | active | xtask |\n| QLT-001 | planned | |",
            ),
            fixture(PROOFS_FILE, "| CNF-009 | 正例・反例 | 通る／落ちる |\n"),
        ]
    }

    #[test]
    fn consistent_documents_pass() {
        assert!(inspect(&consistent(), missing_root()).is_empty());
    }

    #[test]
    fn catches_state_mismatch() {
        let files = vec![
            gates(
                "### CNF-009 — 文書整合\n\n- 機械強制: **planned**",
                "| CNF-009 | active | xtask |",
            ),
            fixture(PROOFS_FILE, "| CNF-009 | x | y |\n"),
        ];
        assert_eq!(inspect(&files, missing_root()).len(), 1);
    }

    #[test]
    fn catches_missing_matrix_row() {
        let files = vec![gates(
            "### CNF-009 — 文書整合\n\n- 機械強制: **planned**",
            "",
        )];
        assert_eq!(inspect(&files, missing_root()).len(), 1);
    }

    #[test]
    fn catches_active_without_proof() {
        let files = vec![gates(
            "### CNF-009 — 文書整合\n\n- 機械強制: **active**",
            "| CNF-009 | active | xtask |",
        )];
        assert_eq!(inspect(&files, missing_root()).len(), 1);
    }

    #[test]
    fn catches_duplicate_definition() {
        let mut files = consistent();
        files.push(fixture(
            "docs/CODING_RULES.md",
            "### CNF-009 — 二重\n\n- 機械強制: **active**\n",
        ));
        assert!(
            inspect(&files, missing_root())
                .iter()
                .any(|v| v.to_string().contains("重複"))
        );
    }

    #[test]
    fn catches_missing_state() {
        let files = vec![gates(
            "### CNF-009 — 文書整合\n\n本文だけ",
            "| CNF-009 | active | xtask |",
        )];
        assert!(
            inspect(&files, missing_root())
                .iter()
                .any(|v| v.to_string().contains("状態が無い"))
        );
    }

    #[test]
    fn catches_undefined_rule_reference() {
        let mut files = consistent();
        files.push(fixture("README.md", "RS-999 を守る\n"));
        assert!(
            inspect(&files, missing_root())
                .iter()
                .any(|v| v.to_string().contains("RS-999"))
        );
    }

    #[test]
    fn catches_template_placeholder() {
        let mut files = consistent();
        files.push(fixture("README.md", "{{TOOL}} を使う\n"));
        assert!(
            inspect(&files, missing_root())
                .iter()
                .any(|v| v.to_string().contains("雛形"))
        );
    }

    #[test]
    fn catches_broken_relative_link() {
        let mut files = consistent();
        files.push(fixture("docs/a.md", "[規約](CODING_RULES.md) を読む\n"));
        assert!(
            inspect(&files, missing_root())
                .iter()
                .any(|v| v.to_string().contains("リンク先"))
        );
    }

    #[test]
    fn ignores_external_and_anchor_links() {
        let mut files = consistent();
        files.push(fixture(
            "docs/a.md",
            "[rust](https://www.rust-lang.org/) [節](#section) [mail](mailto:a@b)\n",
        ));
        assert!(inspect(&files, missing_root()).is_empty());
    }

    #[test]
    fn first_state_line_wins_over_later_ones() {
        let files = vec![gates(
            "### CNF-009 — 文書整合\n\n- 機械強制: **planned**（CNF）\n- 機械強制: **不能**（判断）",
            "| CNF-009 | planned | |",
        )];
        assert!(inspect(&files, missing_root()).is_empty());
    }
}
