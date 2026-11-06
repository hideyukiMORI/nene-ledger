//! `NeNe Ledger` の道具。規約検査（CNF-0xx）・Git 規約（GIT-0xx）・ゲート発火の証明を 1 つの
//! バイナリで持つ。規則の正本は `docs/`。lint が見ないもの、つまり**このリポジトリとして
//! 守るべきこと**だけをここで見る。
//!
//! 🔴 依存を足さずに書くこと（ARC-013）。CNF-0xx は全て構文で判定できる。
//!
//! ```console
//! $ cargo run --quiet -p xtask -- conformance
//! $ cargo run --quiet -p xtask -- validate-git
//! $ cargo run --quiet -p xtask -- commit-msg .git/COMMIT_EDITMSG
//! $ cargo run --quiet -p xtask -- prove
//! $ cargo run --quiet --release -p xtask -- bench-pollock <Pollock の置き場>
//! $ cargo run --quiet -p xtask -- notices          # THIRD-PARTY-NOTICES.txt を作り直す
//! $ cargo run --quiet -p xtask -- notices-check    # Cargo.lock と食い違えば落とす
//! ```

mod bench_pollock;
mod check;
mod documents;
mod git_conventions;
mod git_range;
mod lint_tables;
mod proof_case;
mod prove;
mod rule_id;
mod source_file;
mod third_party;
mod violation;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::source_file::SourceFile;
use crate::violation::Violation;

/// リポジトリのルート。`xtask/` の 1 つ上。
fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

/// 全ファイルに全検査をかけ、違反をパス順に返す。
fn inspect(files: &[SourceFile], root: &Path) -> Vec<Violation> {
    let known = documents::known_rule_ids(files);
    let mut violations: Vec<Violation> = files
        .iter()
        .flat_map(|file| {
            let mut found = check::no_default_construction(file);
            found.extend(check::no_forbidden_constructs(file));
            found.extend(check::one_primary_declaration(file));
            found.extend(check::role_bearing_names(file));
            found.extend(check::suppression_cites_rule(file, &known));
            found.extend(check::no_build_script(file));
            found.extend(check::task_markers_cite_issue(file));
            found.extend(check::no_gate_disabling(file));
            found.extend(check::no_nondeterministic_input(file));
            found.extend(check::scan_does_not_write(file));
            found.extend(check::dependencies_follow_graph(file));
            found
        })
        .collect();
    violations.extend(check::core_crates_declare_no_std(files));
    violations.extend(lint_tables::unsafe_crates_mirror_workspace(files));
    violations.extend(documents::inspect(files, root));
    violations
}

/// 副命令。閉じた集合で、増えたら `match` が落ちる（RS-002）。
enum Task {
    Conformance,
    ValidateGit,
    CommitMessage(PathBuf),
    Prove,
    BenchPollock(PathBuf),
    Notices,
    NoticesCheck,
}

/// 引数から副命令を決める。
fn task_from(arguments: &[String]) -> Result<Task, String> {
    let name = arguments.get(1_usize).map_or("conformance", String::as_str);
    match name {
        "conformance" => Ok(Task::Conformance),
        "validate-git" => Ok(Task::ValidateGit),
        "commit-msg" => arguments
            .get(2_usize)
            .map(|file| Task::CommitMessage(PathBuf::from(file)))
            .ok_or_else(|| "commit-msg needs the message file".to_owned()),
        "prove" => Ok(Task::Prove),
        "bench-pollock" => arguments
            .get(2_usize)
            .map(|directory| Task::BenchPollock(PathBuf::from(directory)))
            .ok_or_else(|| "bench-pollock needs the Pollock directory".to_owned()),
        "notices" => Ok(Task::Notices),
        "notices-check" => Ok(Task::NoticesCheck),
        "conformance-help" | "help" | "--help" | "-h" => Err(
            "usage: xtask [conformance|validate-git|commit-msg <file>|prove|bench-pollock <dir>|notices|notices-check]"
                .to_owned(),
        ),
        other => Err(format!("unknown task `{other}`")),
    }
}

/// 規約検査を走らせ、報告の行を返す。
fn conformance(root: &Path) -> Result<Vec<String>, String> {
    let files =
        source_file::collect(root).map_err(|error| format!("検査対象を読めなかった: {error}"))?;
    let violations = inspect(&files, root);
    if violations.is_empty() {
        return Ok(vec![format!(
            "xtask: 規約違反なし（検査 {} ファイル）",
            files.len()
        )]);
    }
    let rules: std::collections::BTreeSet<&str> = violations.iter().map(Violation::rule).collect();
    let mut lines = vec![format!(
        "xtask: 規約違反 {} 件（{}）",
        violations.len(),
        rules.into_iter().collect::<Vec<_>>().join(", ")
    )];
    lines.extend(violations.iter().map(|violation| format!("  {violation}")));
    Err(lines.join("\n"))
}

/// 証明の結果を Markdown の表にする。`docs/quality/gate-proofs.md` へそのまま転記できる。
fn proof_table(proofs: &[prove::Proof]) -> Vec<String> {
    let mut lines = vec![
        "| # | 規則 | 命令 | 終了 | 実出力 |".to_owned(),
        "| --- | --- | --- | --- | --- |".to_owned(),
    ];
    lines.extend(proofs.iter().map(|p| {
        format!(
            "| {} | {} | `{}` | {} | `{}` |",
            p.id(),
            p.rule(),
            p.command(),
            p.exit_code(),
            p.excerpt().replace('`', "'").replace('|', "\\|")
        )
    }));
    lines.push(format!(
        "Gate proofs passed: {} real-tool proofs（target/proofs/results.json）",
        proofs.len()
    ));
    lines
}

/// 副命令を実行し、標準出力へ出す行を返す。失敗は `Err` の文言。
fn execute(task: Task, root: &Path) -> Result<Vec<String>, String> {
    match task {
        Task::Conformance => conformance(root),
        Task::ValidateGit => {
            let errors = git_conventions::validate_repository(root)?;
            errors
                .is_empty()
                .then(|| vec!["Git conventions passed.".to_owned()])
                .ok_or_else(|| errors.join("\n"))
        }
        Task::CommitMessage(file) => {
            let message = fs::read_to_string(&file)
                .map_err(|error| format!("{}: {error}", file.display()))?;
            let errors = git_conventions::validate(message.trim_start_matches('\u{feff}'), false);
            errors
                .is_empty()
                .then(Vec::new)
                .ok_or_else(|| errors.join("\n"))
        }
        Task::Prove => prove::run_all(root).map(|proofs| proof_table(&proofs)),
        Task::BenchPollock(pollock) => {
            let binary = root.join("target").join("release").join("nene-ledger");
            if !binary.exists() {
                return Err(format!(
                    "{} が無い。make bench-pollock で release を組む",
                    binary.display()
                ));
            }
            bench_pollock::run(&pollock, &binary)
        }
        Task::Notices => third_party::write(root),
        Task::NoticesCheck => third_party::check(root),
    }
}

// 🔴 出力を行ってよい唯一の場所（RS-014）。
#[expect(
    clippy::print_stdout,
    reason = "RS-014: 検査結果の報告点は main だけである"
)]
#[expect(
    clippy::print_stderr,
    reason = "RS-014: 失敗の報告点は main だけである"
)]
fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().collect();
    let root = repository_root();
    match task_from(&arguments).and_then(|task| execute(task, &root)) {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
