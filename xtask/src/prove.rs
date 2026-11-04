//! `make prove` — ゲートが意図した違反で発火することの実測（QLT-007）。
//!
//! 🔴 検査の最大の失敗は、見逃したまま常に緑を返すことである。本物のコードを見ている限り
//! それは発覚しない。ここでは `target/proofs/` に最小の workspace を作り、本物の
//! `cargo clippy` / `cargo fmt` / `cargo run --release` / `cargo deny` に違反を食わせ、
//! 落ちること（negative）と、元に戻せば通ること（restoration）の両方を確かめる。
//! 結果は `target/proofs/results.json` に残し、`docs/quality/gate-proofs.md` に転記する。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::proof_case::ProofCase;

/// 証明 1 件の結果。
#[derive(Debug, Clone)]
pub(crate) struct Proof {
    id: &'static str,
    rule: &'static str,
    command: String,
    exit_code: i32,
    excerpt: String,
}

impl Proof {
    /// 証明の番号（`P-01` 等）。
    pub(crate) fn id(&self) -> &'static str {
        self.id
    }

    /// 守る規則。
    pub(crate) fn rule(&self) -> &'static str {
        self.rule
    }

    /// 実行した命令。
    pub(crate) fn command(&self) -> &str {
        &self.command
    }

    /// 終了コード。
    pub(crate) fn exit_code(&self) -> i32 {
        self.exit_code
    }

    /// 実出力のうち、期待した診断を含む 1 行。
    pub(crate) fn excerpt(&self) -> &str {
        &self.excerpt
    }
}

/// 正しい `main.rs`。復帰の確認に使う。
const VALID_MAIN: &str = "//! probe\nfn main() {}\n";

/// 正しい `lib.rs`。
const VALID_LIB: &str = "//! probe\n";

/// 命令を走らせ、終了コードと結合した出力を返す。
fn run(directory: &Path, program: &str, arguments: &[&str]) -> Result<(i32, String), String> {
    let output = Command::new(program)
        .args(arguments)
        .current_dir(directory)
        .env_remove("RUSTFLAGS")
        .output()
        .map_err(|error| format!("{program} could not start: {error}"))?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok((output.status.code().unwrap_or(-1_i32), text))
}

/// 出力から、期待した診断を含む最初の 1 行を取り出す。
///
/// 🔴 絶対パスは相対に直す。実行した環境ごとに記録が変わると差分が取れない（RS-016 と同じ理由）。
fn excerpt_of(output: &str, diagnostic: &str, root: &Path) -> Option<String> {
    let prefix = format!("{}/", root.display());
    output
        .lines()
        .find(|line| line.contains(diagnostic))
        .map(|line| {
            line.trim()
                .replace(&prefix, "")
                .chars()
                .take(160_usize)
                .collect()
        })
}

/// 根の `Cargo.toml` を、メンバーを `probe` だけに替えて写す。lint の集合は同じ物を使う。
fn workspace_manifest(root: &Path) -> Result<String, String> {
    let text = fs::read_to_string(root.join("Cargo.toml")).map_err(|e| e.to_string())?;
    Ok(probe_members(&text))
}

/// `members = [` から閉じ括弧の行までを `members = ["probe"]` の 1 行に替える。1 行の形も複数行の形も（NeNeDocket Issue #69）。
fn probe_members(text: &str) -> String {
    let mut replaced = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        if line.starts_with("members = [") {
            replaced.push("members = [\"probe\"]");
            inside = !line.trim_end().ends_with(']');
        } else if inside {
            inside = line.trim() != "]";
        } else {
            replaced.push(line);
        }
    }
    replaced.join("\n") + "\n"
}

/// 証明用の最小 workspace を `target/proofs/workspace` に作り直す。
fn prepare_workspace(root: &Path) -> Result<PathBuf, String> {
    let fixture = root.join("target/proofs/workspace");
    if fixture.exists() {
        fs::remove_dir_all(&fixture).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(fixture.join("probe/src")).map_err(|e| e.to_string())?;
    fs::write(fixture.join("Cargo.toml"), workspace_manifest(root)?).map_err(|e| e.to_string())?;
    for name in ["clippy.toml", "rust-toolchain.toml", "deny.toml"] {
        fs::copy(root.join(name), fixture.join(name)).map_err(|e| format!("{name}: {e}"))?;
    }
    let manifest = "[package]\nname = \"probe\"\ndescription = \"gate proof\"\nversion.workspace = true\nedition.workspace = true\nlicense.workspace = true\nauthors.workspace = true\npublish = false\n\n[lints]\nworkspace = true\n";
    fs::write(fixture.join("probe/Cargo.toml"), manifest).map_err(|e| e.to_string())?;
    restore(&fixture)?;
    Ok(fixture)
}

/// `probe/src` を正しい 2 ファイルだけに戻す。
fn restore(fixture: &Path) -> Result<(), String> {
    let src = fixture.join("probe/src");
    if src.exists() {
        fs::remove_dir_all(&src).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&src).map_err(|e| e.to_string())?;
    fs::write(src.join("main.rs"), VALID_MAIN).map_err(|e| e.to_string())?;
    fs::write(src.join("lib.rs"), VALID_LIB).map_err(|e| e.to_string())?;
    Ok(())
}

/// `main` の本体だけを差し替えた `main.rs`。
fn main_with(body: &str) -> String {
    format!("//! probe\nfn main() {{\n{body}\n}}\n")
}

/// 単一の `main.rs` を持つ違反。
fn main_case(
    id: &'static str,
    rule: &'static str,
    source: String,
    diagnostic: &'static str,
) -> ProofCase {
    ProofCase::new(
        id,
        rule,
        vec![("probe/src/main.rs".to_owned(), source)],
        diagnostic,
    )
}

/// 認知的複雑度が上限を超える関数。
fn complex_main() -> String {
    let mut body = String::from("    let x = std::hint::black_box(3_u8);\n    let mut s = 0_u8;\n");
    for value in 0_u8..12_u8 {
        let branch =
            format!("    if x == {value}_u8 {{\n        s = s.saturating_add(1_u8);\n    }}\n");
        body.push_str(&branch);
    }
    body.push_str("    std::hint::black_box(s);");
    main_with(&body)
}

/// 60 行を超える関数。
fn long_main() -> String {
    let lines: Vec<&str> =
        std::iter::repeat_n("    std::hint::black_box(1_u8);", 62_usize).collect();
    main_with(&lines.join("\n"))
}

/// lint と言語の違反の一覧。
fn lint_cases() -> Vec<ProofCase> {
    vec![
        main_case("P-01", "RS-019", "//! probe\n#[expect(clippy::unwrap_used, reason = \"RS-005: 試し\")]\nfn main() {\n    let v: Option<u8> = Some(1_u8);\n    std::hint::black_box(v.unwrap());\n}\n".to_owned(), "incompatible with previous forbid"),
        main_case("P-02", "RS-019", "//! probe\n#[expect(unused_variables, reason = \"RS-019: 発火しない抑制\")]\nfn main() {\n    let used = 1_u8;\n    std::hint::black_box(used);\n}\n".to_owned(), "this lint expectation is unfulfilled"),
        main_case("P-03", "QLT-003", "//! probe\n#[allow(unused_variables)]\nfn main() {}\n".to_owned(), "attribute"),
        main_case("P-04", "RS-002", "//! probe\nenum Mode { Px, Mm }\nfn main() {\n    let m = std::hint::black_box(Mode::Px);\n    let n: u8 = match m {\n        Mode::Px => 1_u8,\n        _ => 2_u8,\n    };\n    std::hint::black_box((n, Mode::Mm));\n}\n".to_owned(), "wildcard match"),
        main_case("P-05", "RS-002", "//! probe\nenum Mode { Px, Mm }\nfn main() {\n    let m = std::hint::black_box(Mode::Px);\n    let n: u8 = match m {\n        Mode::Px => 1_u8,\n    };\n    std::hint::black_box((n, Mode::Mm));\n}\n".to_owned(), "E0004"),
        main_case("P-06", "RS-007", main_with("    let n = std::hint::black_box(300_u16);\n    let m: u8 = n as u8;\n    std::hint::black_box(m);"), "`as` conversion"),
        main_case("P-07", "RS-016", "//! probe\nuse std::collections::HashMap;\nfn main() {\n    let m: HashMap<u8, u8> = HashMap::new();\n    std::hint::black_box(m);\n}\n".to_owned(), "disallowed type"),
        main_case("P-08", "RS-018", main_with("    let n = 1;\n    std::hint::black_box(n);"), "default numeric fallback"),
        ProofCase::new("P-09", "RS-008", vec![("probe/src/lib.rs".to_owned(), "//! probe\npub fn undocumented() {}\n".to_owned())], "missing documentation"),
        ProofCase::new("P-10", "RS-012", vec![("probe/src/main.rs".to_owned(), "//! probe\nmod probe10;\nfn main() {\n    probe10::touch();\n}\n".to_owned()), ("probe/src/probe10/mod.rs".to_owned(), "pub(crate) fn touch() {}\n".to_owned())], "mod.rs"),
        main_case("P-11", "RS-011", complex_main(), "cognitive complexity"),
        main_case("P-12", "RS-011", long_main(), "too many lines"),
        main_case("P-13", "RS-005", main_with("    let v: Option<u8> = Some(1_u8);\n    std::hint::black_box(v.unwrap());"), "unwrap"),
        main_case("P-14", "RS-005", main_with("    let v: &[u8] = &std::hint::black_box([1_u8]);\n    std::hint::black_box(v[0]);"), "indexing may panic"),
        main_case("P-15", "RS-001", "//! probe\nmod m {\n    pub(crate) struct Id(u8);\n    impl Id {\n        pub(crate) fn value(&self) -> u8 {\n            self.0\n        }\n    }\n}\nfn main() {\n    std::hint::black_box(m::Id(1_u8).value());\n}\n".to_owned(), "E0603"),
        main_case("P-16", "RS-003", main_with("    let z: u8;\n    std::hint::black_box(z);"), "E0381"),
        main_case("P-17", "RS-003", "//! probe\nstruct Config { a: u8, b: u8 }\nfn main() {\n    let c = Config { a: 1_u8 };\n    std::hint::black_box((c.a, c.b));\n}\n".to_owned(), "E0063"),
        main_case("P-18", "RS-010", main_with("    let p: *const u8 = &1_u8;\n    let v = unsafe { *p };\n    std::hint::black_box(v);"), "unsafe"),
        main_case("P-19", "RS-014", main_with("    println!(\"probe\");"), "println"),
        main_case(
            "P-26",
            "RS-013",
            main_with("    let data = std::hint::black_box(1_u8);\n    std::hint::black_box(data);"),
            "disallowed/placeholder name",
        ),
    ]
}

/// 1 件の違反を書き込み、clippy が落ちることと、戻せば通ることを確かめる。
fn prove_lint(root: &Path, fixture: &Path, case: &ProofCase) -> Result<Proof, String> {
    for (path, content) in case.files() {
        let target = fixture.join(path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(&target, content).map_err(|e| e.to_string())?;
    }
    let arguments = ["clippy", "--all-targets", "--quiet", "--", "-D", "warnings"];
    let (code, output) = run(fixture, "cargo", &arguments)?;
    let excerpt = excerpt_of(&output, case.diagnostic(), root).ok_or_else(|| {
        format!(
            "{}: expected `{}` but got:\n{output}",
            case.id(),
            case.diagnostic()
        )
    })?;
    if code == 0_i32 {
        return Err(format!("{}: the gate did not fail", case.id()));
    }
    restore(fixture)?;
    let (restored, restored_output) = run(fixture, "cargo", &arguments)?;
    if restored != 0_i32 {
        return Err(format!(
            "{}: restoration failed:\n{restored_output}",
            case.id()
        ));
    }
    Ok(Proof {
        id: case.id(),
        rule: case.rule(),
        command: format!("cargo {}", arguments.join(" ")),
        exit_code: code,
        excerpt,
    })
}

/// 整形差分が失敗すること（QLT-004）。
fn prove_format(root: &Path, fixture: &Path) -> Result<Proof, String> {
    fs::write(
        fixture.join("probe/src/main.rs"),
        "//! probe\nfn main(){let x=1_u8;std::hint::black_box(x);}\n",
    )
    .map_err(|e| e.to_string())?;
    let arguments = ["fmt", "--all", "--check"];
    let (code, output) = run(fixture, "cargo", &arguments)?;
    let excerpt = excerpt_of(&output, "Diff in", root)
        .ok_or_else(|| format!("P-20: rustfmt did not report a diff:\n{output}"))?;
    restore(fixture)?;
    let (restored, _) = run(fixture, "cargo", &arguments)?;
    if code == 0_i32 || restored != 0_i32 {
        return Err("P-20: format proof did not behave as expected".to_owned());
    }
    Ok(Proof {
        id: "P-20",
        rule: "QLT-004",
        command: "cargo fmt --all --check".to_owned(),
        exit_code: code,
        excerpt,
    })
}

/// リリースでも整数のオーバーフローが落ちること（RS-017）。
fn prove_overflow(root: &Path, fixture: &Path) -> Result<Proof, String> {
    let source = main_with(
        "    let a = std::hint::black_box(255_u8);\n    let b = std::hint::black_box(1_u8);\n    let c = a + b;\n    std::hint::black_box(c);",
    );
    fs::write(fixture.join("probe/src/main.rs"), source).map_err(|e| e.to_string())?;
    let arguments = ["run", "--release", "--quiet", "-p", "probe"];
    let (code, output) = run(fixture, "cargo", &arguments)?;
    let excerpt = excerpt_of(&output, "attempt to add with overflow", root)
        .ok_or_else(|| format!("P-21: release build wrapped silently:\n{output}"))?;
    restore(fixture)?;
    if code == 0_i32 {
        return Err("P-21: overflow did not fail".to_owned());
    }
    Ok(Proof {
        id: "P-21",
        rule: "RS-017",
        command: "cargo run --release -p probe".to_owned(),
        exit_code: code,
        excerpt,
    })
}

/// `--locked` が lock の更新を拒むこと（QLT-011）。
fn prove_locked(root: &Path, fixture: &Path) -> Result<Proof, String> {
    let lock = fixture.join("Cargo.lock");
    if lock.exists() {
        fs::remove_file(&lock).map_err(|e| e.to_string())?;
    }
    let arguments = ["check", "--locked", "--quiet"];
    let (code, output) = run(fixture, "cargo", &arguments)?;
    let excerpt = excerpt_of(&output, "--locked", root)
        .ok_or_else(|| format!("P-22: --locked did not refuse:\n{output}"))?;
    let (restored, _) = run(fixture, "cargo", &["check", "--quiet"])?;
    if code == 0_i32 || restored != 0_i32 {
        return Err("P-22: lock proof did not behave as expected".to_owned());
    }
    Ok(Proof {
        id: "P-22",
        rule: "QLT-011",
        command: "cargo check --locked（Cargo.lock を消した状態）".to_owned(),
        exit_code: code,
        excerpt,
    })
}

/// クレート間の循環依存を cargo が拒むこと（ARC-002）。
fn prove_cycle(root: &Path) -> Result<Proof, String> {
    let fixture = root.join("target/proofs/cycle");
    if fixture.exists() {
        fs::remove_dir_all(&fixture).map_err(|e| e.to_string())?;
    }
    for (name, other) in [("a", "b"), ("b", "a")] {
        fs::create_dir_all(fixture.join(name).join("src")).map_err(|e| e.to_string())?;
        let manifest = format!(
            "[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[dependencies]\n{other} = {{ path = \"../{other}\" }}\n"
        );
        fs::write(fixture.join(name).join("Cargo.toml"), manifest).map_err(|e| e.to_string())?;
        fs::write(fixture.join(name).join("src/lib.rs"), "//! cycle\n")
            .map_err(|e| e.to_string())?;
    }
    fs::write(
        fixture.join("Cargo.toml"),
        "[workspace]\nresolver = \"3\"\nmembers = [\"a\", \"b\"]\n",
    )
    .map_err(|e| e.to_string())?;
    fs::copy(
        root.join("rust-toolchain.toml"),
        fixture.join("rust-toolchain.toml"),
    )
    .map_err(|e| e.to_string())?;
    let (code, output) = run(&fixture, "cargo", &["check", "--quiet"])?;
    let excerpt = excerpt_of(&output, "cyclic package dependency", root)
        .ok_or_else(|| format!("P-23: cargo accepted a cycle:\n{output}"))?;
    if code == 0_i32 {
        return Err("P-23: cycle did not fail".to_owned());
    }
    Ok(Proof {
        id: "P-23",
        rule: "ARC-002",
        command: "cargo check（a ⇄ b の循環）".to_owned(),
        exit_code: code,
        excerpt,
    })
}

/// 証明の workspace で実際に使うライセンス（probe の独自のライセンスだけ・ADR 0009）。
const PROBE_LICENSES: &str = "allow = [\"LicenseRef-NeNe-Ledger-1.0\"]";

/// `deny.toml` のライセンスの許可の行（`allow = [`）を `allow` に替える。並びの中身に依らない（NeNeDocket Issue #69）。
fn license_allow(text: &str, allow: &str) -> String {
    let replaced: Vec<&str> = text
        .lines()
        .map(|line| {
            if line.starts_with("allow = [") {
                allow
            } else {
                line
            }
        })
        .collect();
    replaced.join("\n") + "\n"
}

/// ライセンスの許可制が落ちること（ARC-013）。
fn prove_deny(root: &Path, fixture: &Path) -> Result<Vec<Proof>, String> {
    let deny = fixture.join("deny.toml");
    let original = fs::read_to_string(&deny).map_err(|e| e.to_string())?;
    let arguments = [
        "deny",
        "--locked",
        "check",
        "licenses",
        "--deny",
        "license-not-encountered",
    ];
    let mut proofs = Vec::new();
    for (id, allow, diagnostic) in [
        ("P-24", "allow = []", "not explicitly allowed"),
        (
            "P-25",
            "allow = [\"MIT\", \"Apache-2.0\"]",
            "license-not-encountered",
        ),
    ] {
        fs::write(&deny, license_allow(&original, allow)).map_err(|e| e.to_string())?;
        let (code, output) = run(fixture, "cargo", &arguments)?;
        let excerpt = excerpt_of(&output, diagnostic, root)
            .ok_or_else(|| format!("{id}: cargo-deny did not report `{diagnostic}`:\n{output}"))?;
        if code == 0_i32 {
            return Err(format!("{id}: cargo-deny did not fail"));
        }
        proofs.push(Proof {
            id,
            rule: "ARC-013",
            command: format!("cargo {}（{allow}）", arguments.join(" ")),
            exit_code: code,
            excerpt,
        });
    }
    // 証明の workspace は独自のライセンスの probe だけなので、戻すのはその許可だけ（NeNeDocket Issue #69 の継承）。
    fs::write(&deny, license_allow(&original, PROBE_LICENSES)).map_err(|e| e.to_string())?;
    let (restored, output) = run(fixture, "cargo", &arguments)?;
    if restored != 0_i32 {
        return Err(format!("cargo-deny restoration failed:\n{output}"));
    }
    Ok(proofs)
}

/// 結果を JSON として書く（依存ゼロなので手で組む）。
fn write_results(root: &Path, proofs: &[Proof]) -> Result<(), String> {
    let escape = |text: &str| text.replace('\\', "\\\\").replace('"', "\\\"");
    let entries: Vec<String> = proofs
        .iter()
        .map(|p| format!("  {{\"id\": \"{}\", \"rule\": \"{}\", \"command\": \"{}\", \"exitCode\": {}, \"excerpt\": \"{}\"}}", p.id, p.rule, escape(&p.command), p.exit_code, escape(&p.excerpt)))
        .collect();
    let text = format!("[\n{}\n]\n", entries.join(",\n"));
    fs::write(root.join("target/proofs/results.json"), text).map_err(|e| e.to_string())
}

/// 全ての証明を走らせる。1 つでも期待どおりに落ちなければ `Err`。
pub(crate) fn run_all(root: &Path) -> Result<Vec<Proof>, String> {
    let fixture = prepare_workspace(root)?;
    let (baseline, output) = run(
        &fixture,
        "cargo",
        &["clippy", "--all-targets", "--quiet", "--", "-D", "warnings"],
    )?;
    if baseline != 0_i32 {
        return Err(format!("the valid probe does not pass clippy:\n{output}"));
    }
    let mut proofs = Vec::new();
    for case in lint_cases() {
        proofs.push(prove_lint(root, &fixture, &case)?);
    }
    proofs.push(prove_format(root, &fixture)?);
    proofs.push(prove_overflow(root, &fixture)?);
    proofs.push(prove_locked(root, &fixture)?);
    proofs.push(prove_cycle(root)?);
    proofs.extend(prove_deny(root, &fixture)?);
    write_results(root, &proofs)?;
    Ok(proofs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_members_replaces_a_single_line_list() {
        let text = "[workspace]\nmembers = [\"a\", \"b\"]\n\n[workspace.lints.rust]\n";
        assert_eq!(
            probe_members(text),
            "[workspace]\nmembers = [\"probe\"]\n\n[workspace.lints.rust]\n"
        );
    }

    #[test]
    fn license_allow_replaces_only_the_license_list() {
        let text = "[licenses]\nallow = [\"MIT\", \"Apache-2.0\"]\n\n[sources]\nallow-registry = [\"x\"]\n";
        assert_eq!(
            license_allow(text, PROBE_LICENSES),
            "[licenses]\nallow = [\"LicenseRef-NeNe-Ledger-1.0\"]\n\n[sources]\nallow-registry = [\"x\"]\n"
        );
    }

    #[test]
    fn probe_members_replaces_a_multi_line_list() {
        let text = "[workspace]\nmembers = [\n    \"a\",\n    \"b\",\n]\n\n[workspace.package]\n";
        assert_eq!(
            probe_members(text),
            "[workspace]\nmembers = [\"probe\"]\n\n[workspace.package]\n"
        );
    }
}
