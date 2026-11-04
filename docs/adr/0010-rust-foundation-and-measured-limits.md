# ADR 0010 — Rust の検査基盤を NeNeDocket から移し、このリポジトリで測れた規則だけを active にする

- 状態: 受理
- 日付: 2026-10-22
- Issue: #2
- 影響する規則: RS-001〜RS-019 / ARC-002 / ARC-007 / ARC-013 / QLT-002 / QLT-003 / QLT-004 / QLT-011 / CNF-001〜CNF-015 / GIT-001〜GIT-005

## 文脈

[ADR 0001](0001-strictness-is-mechanically-enforced.md) は「文書を先に置き、機械を次に置く」と決め、Issue #1 ではすべての規則を planned と書いた。
Issue #2 で NeNeDocket の機械を移し、**このリポジトリの道具で**発火を測った（rustc 1.98.0・clippy・rustfmt・cargo-deny 0.20.2・WSL2 の Ubuntu 24.04）。

移したもの: `Cargo.toml` の `[workspace.lints]`・`clippy.toml`・`deny.toml`・`rust-toolchain.toml`・`xtask`・`Makefile`・`.githooks/commit-msg`。

NeNeDocket から変えたもの:

| 箇所 | 変えたこと | 理由 |
| --- | --- | --- |
| workspace | メンバーは `xtask` だけ。`repository` を書かない | 空の将来用クレートは作らない・リモートが無い（ADR 0004） |
| ライセンス | `LicenseRef-NeNe-Ledger-1.0`。`deny.toml` の `allow` と証明の戻し先も同じ | ADR 0009 |
| CNF-012 | 除外に `crates/scan/` を足した | ADR 0006 |
| CNF-013 | 許可表を PROJECT_LAYOUT 第 2 節にした（外部クレートは 0・scan と store は外部を表で限る・scan ⇄ store を禁止） | ADR 0006 / 0007 |
| CNF-014 | 対象に `crates/scan/Cargo.toml` を足した（3 クレート） | ADR 0006 |
| CNF-015 | NeNeDocket の「ファイルの時刻を使わない」を持ち込まず、「scan は書き込みの API を使わない」に替えた | ADR 0005。外部の変更の検出に時刻を使うかは FR-019 の ADR で決める |
| Git 規約 | PR と CI のメタデータを読む代わりに、Issue のファイルの統合記録・作業ブランチの `main..HEAD`・`main` の全履歴を見る。`perf` の type を足した。GIT-005（開発機に固有の絶対パス）を足した | ADR 0004・GIT-005 |
| `make ci` | `git diff --check main...HEAD` | リモートが無い |

## 決定

**このリポジトリで測れた規則を active とし、測れなかった規則は planned のまま残す。**

1. active: RS-001〜RS-019・ARC-002・ARC-007・ARC-013・QLT-002・QLT-003・QLT-004・QLT-011・CNF-001〜CNF-004・CNF-006〜CNF-015・GIT-001・GIT-003・GIT-005
2. planned のまま:
   - ARC-003（中核クレートの実ファイルで CNF-007 / CNF-013 を発火させてから）
   - QLT-009（中核が生まれてから `make coverage`）
   - QLT-001 / QLT-005 / QLT-012（`make check` の `REASON` と `make ci` は動くが、「統合の前に走らせたか」「理由の妥当性」は機械で見られない）
   - GIT-002 / GIT-004（名前と統合記録の欄は `make ci` が見るが、`main` への直接のコミットと squash 以外の統合をローカルでは拒めない）
   - CNF-005（判定方法が未定）・レビュー事項の ARC / QLT
3. 道具の置き場: `cargo install` で入れた道具（`cargo-deny` など）は `~/.cargo/bin` にある。`make` を走らせるシェルの `PATH` に入っていることを `make doctor` で確かめる

## 強制

- **active**: 上の 1。実測は `make prove`（26 件）・`cargo test -p xtask`（89 件）・実ファイルでの発火（[gate-proofs](../quality/gate-proofs.md) 第 3〜5 節）
- **planned**: 上の 2。それぞれ条件を書いた

## 結果

得られるもの:

- NeNeDocket と同じ 26 件の negative proof が、このリポジトリで再現した
- 違反ゼロの時点で機械が入った。baseline は要らない（ADR 0001）
- リモートも CI も無くても、`main` の全履歴を `make ci`（`validate-git`）で見直せる。フックを迂回したコミットは、次に `main` で `make ci` を走らせたときに見つかる

失うもの・残る穴（**正直に**）:

- 🔴 **製品クレートの実ファイルでは、まだ何も発火させていない。** CNF-007 / CNF-012 / CNF-013（製品の向き）/ CNF-014 / CNF-015 は正例・反例のテストでしか見ていない。最初の製品クレートの Issue で実ファイルで発火させて記録する
- 🔴 **`make ci` は手で走らせる。** 走らせずに squash すれば何も止まらない。`main` で走らせれば後から見つかる（GIT-001 / GIT-003 / GIT-005 の履歴の検査）
- 🔴 GIT-005 は字句の検査で、ホームディレクトリの形（`/home/名前/` など）しか見ない。社内の事情の書き込みそのものはレビュー事項
- `make lint-windows` は今は `xtask` しか見ない。Windows のコードが生まれるまで、Windows でだけ緩い書き方は検査されない
- 緑であることは規則が良いことの証明ではない。実装で邪魔になったら ADR で判断を残す

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| NeNeDocket の実測を引用して active と書く | workspace の設定を写し間違えれば発火しない。測ってから書く（QLT-007） |
| NeNeDocket の CNF-015（ファイルの時刻の禁止）をそのまま持ち込む | NeNe Ledger は外部の変更の検出（FR-019）でファイルの時刻を手がかりにする可能性がある。決めるのはその ADR。代わりに ADR 0005 を守る検査を CNF-015 にした |
| GIT-002 / GIT-004 を active と書く | 名前と欄は見られるが、`main` への直接のコミットと squash 以外の統合を拒む層がローカルに無い |
| `make ci` を git のフック（pre-push など）で強制する | push 先が無い。pre-commit で全履歴を見ると毎回のコミットが重くなる |
