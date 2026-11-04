//! Git 規約（GIT-001〜GIT-005）の検証器。commit-msg フックと CI の必須 check が同じ判定を使う。
//!
//! 手元では `origin/main..HEAD` の件名と管理しているファイルを、CI では PR のメタデータ
//! （`PR_TITLE` / `PR_BODY` / `PR_HEAD_REF` / `PR_BASE_SHA` / `PR_DRAFT`）も見る（ADR 0021）。

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::git_range::GitRange;

/// Conventional Commits の `type`。`perf` は速さの変更（QLT-014）。
const TYPES: [&str; 9] = [
    "feat", "fix", "docs", "refactor", "test", "build", "ci", "chore", "perf",
];

/// 件名の上限。
const SUBJECT_LIMIT: usize = 100_usize;

/// 統合の先の枝。
const MAIN: &str = "main";

/// PR 本文に要る欄（GIT-004 / QLT-012）。行の始まりで見る。
const PR_FIELDS: [&str; 9] = [
    "目的:",
    "使った正典経路:",
    "規則 ID:",
    "振る舞い・スキーマの変更:",
    "確認する退行:",
    "検証",
    "再利用",
    "抑制",
    "残るリスク:",
];

/// 開発機に固有の絶対パスの始まり（GIT-005）。直後に名前の文字が続くときだけ違反にする。
const MACHINE_PATH_PREFIXES: [&str; 4] = [
    concat!("/ho", "me/"),
    concat!("/mnt/c/", "Users/"),
    concat!("C:\\", "Users\\"),
    concat!("/Us", "ers/"),
];

/// 件名の分解結果。
#[derive(Debug, PartialEq, Eq)]
struct Subject<'a> {
    breaking: bool,
    description: &'a str,
}

/// `type(scope)!: 説明 (#N)` を分解する。形が違えば `None`。
fn parse_subject(subject: &str) -> Option<Subject<'_>> {
    let (head, tail) = subject.split_once(": ")?;
    let (head, breaking) = head.strip_suffix('!').map_or((head, false), |h| (h, true));
    let (kind, scope) = head
        .split_once('(')
        .map_or((head, None), |(k, s)| (k, Some(s)));
    if !TYPES.contains(&kind) || !scope.is_none_or(valid_scope) {
        return None;
    }
    let (description, issue) = tail.rsplit_once(" (#")?;
    let numbered = issue_number(issue.strip_suffix(')')?);
    (numbered && !description.is_empty()).then_some(Subject {
        breaking,
        description,
    })
}

/// 先頭が 0 でない数字だけの並びか。
fn issue_number(text: &str) -> bool {
    text.starts_with(|c: char| c.is_ascii_digit() && c != '0')
        && text.chars().all(|c| c.is_ascii_digit())
}

/// `scope)` の形で、小文字英数字と `.` `-` だけからなるか。
fn valid_scope(scope: &str) -> bool {
    let Some(body) = scope.strip_suffix(')') else {
        return false;
    };
    let first_ok = body.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit());
    first_ok
        && body
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-')
}

/// 日本語（ひらがな・カタカナ・漢字）を含むか。
fn has_japanese(text: &str) -> bool {
    text.chars()
        .any(|c| ('\u{3040}'..='\u{30ff}').contains(&c) || ('\u{3400}'..='\u{9fff}').contains(&c))
}

/// GIT-003 / GIT-005 — コミットメッセージ（または PR タイトル）を検証し、違反の文言を返す。
pub(crate) fn validate(message: &str, title_only: bool) -> Vec<String> {
    let subject = message.lines().next().unwrap_or("");
    let Some(parsed) = parse_subject(subject).filter(|_| subject.chars().count() <= SUBJECT_LIMIT)
    else {
        return vec![
            "GIT-003: subject must be `<type>(<scope>): <日本語の説明> (#N)` within 100 characters"
                .to_owned(),
        ];
    };
    let mut errors = Vec::new();
    if !has_japanese(parsed.description) {
        errors.push("GIT-003: description must contain Japanese text".to_owned());
    }
    let footer = message
        .lines()
        .any(|line| line.starts_with("BREAKING CHANGE: "));
    if parsed.breaking && !title_only && !footer {
        errors.push("GIT-003: breaking commit needs a BREAKING CHANGE footer".to_owned());
    }
    errors.extend(
        machine_paths(message).map(|_| {
            "GIT-005: commit message contains a machine-specific absolute path".to_owned()
        }),
    );
    errors
}

/// GIT-002 — ブランチ名 `<type>/<N>-<kebab>`。`main` だけは形の外で許す。
pub(crate) fn valid_branch(branch: &str) -> bool {
    if branch == MAIN {
        return true;
    }
    let Some((kind, rest)) = branch.split_once('/') else {
        return false;
    };
    let Some((number, summary)) = rest.split_once('-') else {
        return false;
    };
    let kebab = !summary.is_empty()
        && !summary.starts_with('-')
        && !summary.ends_with('-')
        && !summary.contains("--")
        && summary
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    TYPES.contains(&kind) && issue_number(number) && kebab
}

/// 1 行に開発機に固有の絶対パスがあれば、その始まりを返す（GIT-005）。
fn machine_path_in(line: &str) -> Option<&'static str> {
    MACHINE_PATH_PREFIXES.iter().copied().find(|prefix| {
        line.match_indices(prefix).any(|(index, _)| {
            line.get(index.saturating_add(prefix.len())..)
                .and_then(|rest| rest.chars().next())
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        })
    })
}

/// 本文の中の、開発機に固有の絶対パスの行番号（1 始まり）。
fn machine_paths(text: &str) -> impl Iterator<Item = usize> + '_ {
    text.lines()
        .enumerate()
        .filter(|(_, line)| machine_path_in(line).is_some())
        .map(|(index, _)| index.saturating_add(1))
}

/// GIT-004 — PR 本文の欄のうち、行の始まりに無いものを返す。
fn missing_pr_fields(body: &str) -> Vec<&'static str> {
    PR_FIELDS
        .into_iter()
        .filter(|field| {
            !body
                .lines()
                .any(|line| line.trim_start().starts_with(field))
        })
        .collect()
}

/// PR のメタデータ（CI が環境変数で渡す）を検証する（GIT-001 / GIT-004 / GIT-005 / QLT-012）。
pub(crate) fn validate_pull_request(title: &str, body: &str, draft: &str) -> Vec<String> {
    let mut errors = Vec::new();
    if draft == "true" {
        errors.push("QLT-012: the required check must not run for draft pull requests".to_owned());
    }
    if !closes_issue(body) {
        errors.push("GIT-001: PR body must contain `Closes #N`".to_owned());
    }
    errors.extend(
        missing_pr_fields(body)
            .into_iter()
            .map(|field| format!("GIT-004: missing PR field `{field}`")),
    );
    errors.extend(machine_paths(body).map(|number| {
        format!("GIT-005: PR body line {number} contains a machine-specific absolute path")
    }));
    errors.extend(
        validate(title, true)
            .into_iter()
            .map(|e| format!("{e} (PR title)")),
    );
    errors
}

/// `Closes #N` を含むか。
fn closes_issue(body: &str) -> bool {
    body.lines().any(|line| {
        line.split("Closes #")
            .skip(1)
            .any(|rest| rest.starts_with(|c: char| c.is_ascii_digit() && c != '0'))
    })
}

/// `git` を走らせ、標準出力を返す。失敗は文言にする。
fn git(root: &Path, arguments: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(root)
        .output()
        .map_err(|error| format!("git could not start: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git {} failed: {}",
            arguments.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// 環境から検証範囲を決める。CI（`PR_HEAD_REF` がある）では PR も検証する。
fn range_from_environment(root: &Path) -> Result<(GitRange, Vec<String>), String> {
    if let Ok(branch) = env::var("PR_HEAD_REF") {
        let base = env::var("PR_BASE_SHA")
            .map_err(|error| format!("GIT-004: PR_BASE_SHA is missing ({error})"))?;
        let title = env::var("PR_TITLE").unwrap_or_default();
        let body = env::var("PR_BODY").unwrap_or_default();
        let draft = env::var("PR_DRAFT").unwrap_or_default();
        return Ok((
            GitRange::new(branch, base),
            validate_pull_request(&title, &body, &draft),
        ));
    }
    let branch = git(root, &["branch", "--show-current"])?.trim().to_owned();
    Ok((GitRange::new(branch, format!("origin/{MAIN}")), Vec::new()))
}

/// 範囲のコミットのメッセージを検証する（GIT-003 / GIT-005）。
fn validate_commits(root: &Path, range: &GitRange) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    let listed = git(root, &["rev-list", &format!("{}..HEAD", range.base())])?;
    for commit in listed
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let message = git(root, &["show", "-s", "--format=%B", commit])?;
        errors.extend(
            validate(&message, false)
                .into_iter()
                .map(|e| format!("{e} (commit {commit})")),
        );
    }
    Ok(errors)
}

/// GIT-005 — 管理しているファイルに、開発機に固有の絶対パスが無いことを見る。
fn validate_tracked_files(root: &Path) -> Result<Vec<String>, String> {
    let listed = git(root, &["ls-files"])?;
    let mut errors = Vec::new();
    for path in listed
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let Ok(text) = fs::read_to_string(root.join(path)) else {
            continue;
        };
        errors.extend(machine_paths(&text).map(|number| {
            format!("GIT-005: {path}:{number} に開発機に固有の絶対パスがある（CLAUDE.local.md へ）")
        }));
    }
    Ok(errors)
}

/// GIT-001〜005 を作業木と履歴（と CI では PR）に対して検証する。戻り値は違反の文言。
pub(crate) fn validate_repository(root: &Path) -> Result<Vec<String>, String> {
    let (range, mut errors) = range_from_environment(root)?;
    if !valid_branch(range.branch()) {
        errors.push(format!("GIT-002: invalid branch name `{}`", range.branch()));
    }
    errors.extend(validate_commits(root, &range)?);
    errors.extend(validate_tracked_files(root)?);
    Ok(errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_documented_form() {
        assert!(validate("build: 検査を固定する (#1)", false).is_empty());
        assert!(validate("feat(ui): 枠なしの窓を出す (#2)", false).is_empty());
        assert!(validate("perf(scan): 索引を並列で作る (#12)", false).is_empty());
    }

    #[test]
    fn rejects_missing_issue() {
        assert!(!validate("build: 検査を固定する", false).is_empty());
    }

    #[test]
    fn rejects_english_description() {
        assert!(!validate("build: enforce checks (#1)", false).is_empty());
    }

    #[test]
    fn rejects_unknown_type_and_bad_scope() {
        assert!(!validate("wip: 途中 (#1)", false).is_empty());
        assert!(!validate("feat(UI): 窓 (#1)", false).is_empty());
    }

    #[test]
    fn rejects_zero_issue_and_long_subject() {
        assert!(!validate("fix: 直す (#0)", false).is_empty());
        let long = format!("fix: {} (#1)", "長".repeat(100));
        assert!(!validate(&long, false).is_empty());
    }

    #[test]
    fn breaking_commit_needs_footer_but_title_does_not() {
        assert!(!validate("build!: 契約を変える (#1)", false).is_empty());
        assert!(
            validate(
                "build!: 契約を変える (#1)\n\nBREAKING CHANGE: 契約を更新",
                false
            )
            .is_empty()
        );
        assert!(validate("build!: 契約を変える (#1)", true).is_empty());
    }

    #[test]
    fn branch_names() {
        assert!(valid_branch("main"));
        assert!(valid_branch("build/2-port-machinery"));
        assert!(!valid_branch("feature/2-window"));
        assert!(!valid_branch("feat/0-window"));
        assert!(!valid_branch("feat/2-Window"));
        assert!(!valid_branch("feat/2-window-"));
        assert!(!valid_branch("claude/inspiring-einstein"));
    }

    fn body() -> String {
        "Closes #2\n\n目的: a\n\n使った正典経路: b\n\n規則 ID: c\n\n振る舞い・スキーマの変更: d\n\n確認する退行: e\n\n検証（実行したコマンドと結果・記録の所在）: f\n\n再利用（元の記録と不変性の根拠、または初回検証）: g\n\n抑制（`#[expect]`）: none\n\n残るリスク: h\n".to_owned()
    }

    #[test]
    fn pull_request_passes_with_all_fields() {
        assert!(validate_pull_request("feat(ui): 窓を出す (#2)", &body(), "false").is_empty());
    }

    #[test]
    fn pull_request_rejects_draft_missing_close_and_bad_title() {
        let errors = validate_pull_request("窓を出す", "目的:", "true");
        assert!(errors.iter().any(|e| e.starts_with("QLT-012")));
        assert!(errors.iter().any(|e| e.starts_with("GIT-001")));
        assert!(errors.iter().any(|e| e.starts_with("GIT-004")));
        assert!(errors.iter().any(|e| e.starts_with("GIT-003")));
    }

    #[test]
    fn pull_request_field_must_start_a_line() {
        let missing =
            missing_pr_fields("本文の途中に 目的: と書いても欄ではない\n残るリスク: 無い\n");
        assert!(missing.contains(&"目的:"));
        assert!(!missing.contains(&"残るリスク:"));
    }

    #[test]
    fn pull_request_body_with_a_machine_path_is_rejected() {
        let text = format!("{}\n{}", body(), concat!("/ho", "me/alice/x に置いた"));
        let errors = validate_pull_request("feat(ui): 窓を出す (#2)", &text, "false");
        assert!(errors.iter().any(|e| e.starts_with("GIT-005")));
    }

    #[test]
    fn finds_machine_specific_paths() {
        for line in [
            concat!("cd /ho", "me/alice/work"),
            concat!("out: /mnt/c/", "Users/alice/WORKS"),
            concat!("C:\\", "Users\\alice\\Desktop"),
            concat!("/Us", "ers/alice/src"),
        ] {
            assert!(machine_path_in(line).is_some(), "{line}");
        }
    }

    #[test]
    fn ignores_patterns_without_a_name() {
        for line in [
            concat!(
                "（`/ho",
                "me/<名前>/`・`/mnt/c/",
                "Users/`・`C:\\",
                "Users\\`）"
            ),
            "XDG_CONFIG_HOME と $HOME の話",
            "users/ の下",
        ] {
            assert!(machine_path_in(line).is_none(), "{line}");
        }
    }

    #[test]
    fn commit_message_with_a_machine_path_is_rejected() {
        let message = concat!("docs: 置き場を書く (#1)\n\n/ho", "me/alice/x に置いた\n");
        assert!(
            validate(message, false)
                .iter()
                .any(|e| e.starts_with("GIT-005"))
        );
    }
}
