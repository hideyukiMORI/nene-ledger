# CLAUDE.md — NeNe Ledger

Claude Code / AI エージェントがこのリポジトリで作業するための**中核ハンドブック**。
簡潔な英語版の入口は [AGENTS.md](AGENTS.md)。詳細の正本は `docs/` にあり、ここには複製しない。

---

## 0. まず読むもの（production コードに触れる前に必ず）

0. [SPECIFICATION.md](SPECIFICATION.md) — 何を作るか（FR-NNN）と、細目の [docs/FEATURES.md](docs/FEATURES.md)（213 項目・フェーズ・状態）。順番は [docs/ROADMAP.md](docs/ROADMAP.md)（M0〜M6）
1. [docs/ARCHITECTURE_CONSTITUTION.md](docs/ARCHITECTURE_CONSTITUTION.md) — 憲章（ARC-NNN）
2. [docs/PROJECT_LAYOUT.md](docs/PROJECT_LAYOUT.md) — クレートと依存方向
3. [docs/CODING_RULES.md](docs/CODING_RULES.md) — Rust 規約（RS-NNN）
4. [docs/QUALITY_GATES.md](docs/QUALITY_GATES.md) — **いま何が機械で守られているか**（QLT-NNN / CNF-NNN）
5. [docs/DEVELOPMENT_WORKFLOW.md](docs/DEVELOPMENT_WORKFLOW.md) — 手順
6. [docs/COMMIT_CONVENTIONS.md](docs/COMMIT_CONVENTIONS.md) — Issue・ブランチ・コミット・PR（GIT-NNN）
7. [docs/GLOSSARY.md](docs/GLOSSARY.md) — 用語
8. 該当する ADR（`docs/adr/`）。とくに [ADR 0005](docs/adr/0005-source-is-read-only-until-save.md)（原本は保存まで書かない）と [ADR 0006](docs/adr/0006-confined-unsafe-for-scan.md)（`unsafe` の区画）

---

## 1. このリポジトリの統治原則

> **一つのことを実現する方法を 1 つに固定し、そのことを人の記憶ではなく機械に守らせる。**

その帰結として、次の 3 つを常に守る。

1. **正典の経路を先に特定してから編集する。** 「ここで書いたほうが早いから」で第 2 の経路を作らない（ARC-001 / ARC-012）
2. **ゲートを弱めて通さない。** 検査が落ちたらコードを直す。閾値・`forbid` の降格・免除・`deny.toml` を触るのは ADR 相当の判断（QLT-010）
3. **`planned` を `active` と書かない。** 未実装の強制を実装済みに見せるのは、この規約体系で唯一「壊す」行為（[ADR 0001](docs/adr/0001-strictness-is-mechanically-enforced.md)）

🔴 機械（lint の表・`xtask`・`Makefile`・commit-msg フック・CI）を置き、このリポジトリで測れた規則だけを active にしてある（ADR 0010）。CI と ruleset が要る GIT-001 / 002 / 004 / QLT-005 は、実測を gate-proofs に記録するまで planned（ADR 0021）。

---

## 2. このプロジェクトで間違えやすい所

### 原本は保存まで書かない。画面の表は派生値

見えている表は「原本 ＋ 差分 ＋ 変換の規則 ＋ 表の見え」から作る。UI が持つ表示用の複製を正本にしない。
保存は一時ファイルに書いて置き換える 1 経路だけ。その場で書き換える経路を作らない（[ADR 0005](docs/adr/0005-source-is-read-only-until-save.md)）。

### セルは文字列。型を推測しない

先頭の `0`・日付に見える値・長い数字・指数表記を、表示でも保存でも変えない。**触っていない行は保存後もバイト列として元と同じ**（FR-003）。
数や日付として扱うのは、利用者が選んだ比較の方法と変換の規則の側だけ。「空のセル」と「セルが無い」を区別する（RS-004）。

### 速い経路には参照実装がある

セルの切り出しの規則の正本は core の素朴な純関数。scan の SIMD と並列の経路は、それと同じ結果を出すことをテストで固定する。
並列の結果はスレッドの数と区切りの位置に依らない（QLT-008・ARC-007）。

### `unsafe` は 3 クレートだけ

`adapters-windows` / `ui-win32`（ADR 0002）と `scan`（ADR 0006）。小さな関数ごとに `#[expect(unsafe_code, reason = "RS-010: …")]` と `// SAFETY:`。
写像（mmap）は、SIGBUS とロックへの備えを決めるまで製品の経路に入れない。scan は書き込みの API を使わない（CNF-015）。

### 抑制は二段構え。`#[allow]` は書けない

`forbid` の lint（`unwrap` / `as` / `_` / `unsafe` / `#[allow]` そのもの）は抑制できない（E0453）。抑制したくなった時点で設計の問題。
`deny` の lint は `#[expect(lint, reason = "<規則 ID>: <理由>")]` だけで抑制でき、発火しなくなれば落ちる（RS-019）。

### 現在時刻・環境・ファイル・スレッドを持てる場所は決まっている

読んでよいのは **`crates/adapters-x11`・`crates/adapters-windows`・`crates/scan`・`crates/store`** だけ（ARC-007）。合成ルートは `std::env::args` だけ。中核は `#![no_std]`。
**テストが実時刻を読むことも決定性の破壊である。**

### 網羅性検査を殺す分岐を書かない・`Default` を書かない・`as` を書かない

閉じた `enum` の `match` に `_` を書かない（RS-002）。`Default` は唯一のファクトリを迂回する門（RS-003）。数値は `400_u32` と書き、変換は `From` / `TryFrom`（RS-007 / RS-018）。

### 依存は許可制。いまは scan の `encoding_rs` と ui-x11 の `x11rb`・`cairo-rs` と Windows の 2 クレートの `windows`・`windows-numerics` だけ

足すときは ADR ＋ `deny.toml` の `allow` を実測で更新 ＋ 規約 第 5 節の表 ＋ CNF-013 の表（ARC-013）。分析エンジン（DuckDB 等）は組み込まない（ADR 0007）。

### 部品の配置と当たり判定は core。UI は描くだけ

UI に業務の判断（並び・選択肢・状態遷移・並べ替え・集計）を置かない（ARC-011）。

### Windows を後回しにしない

利用者の大半は Windows。IME・画面ごとの倍率・Excel とのクリップボード・Shift_JIS・ファイルのロックは早い Issue で扱う。**Windows で動かすまで「動く」と書かない**（QLT-013）。

### 速さは測ってから書く

「速い」には、ファイルの大きさ・機械・比較相手（DuckDB・qsv・xsv・Modern CSV）の同じファイルでの数字を添える（QLT-014）。

---

## 3. 検証コマンド

`cargo install` で入れた道具は `~/.cargo/bin` にある。シェルの `PATH` に無ければ `export PATH="$HOME/.cargo/bin:$PATH"`。

```bash
make lint          # clippy（全 target・--locked）
make lint-windows  # Windows のクレート
make test          # cargo test（xtask の正例・反例を含む）
make conformance   # CNF-0xx
make deny          # 依存の許可制
make doc-check     # rustdoc
make prove         # ゲート発火の証明（ゲートを触ったときだけ）
make ci            # CI の必須 check と同じ 1 行（origin/main が要る・ADR 0021）
```

差分の挙動・直接の依存先と呼び出し元から、起こり得る退行を検出する最小限の検証を選ぶ。全件は `make check REASON='…'` だけ。
実行していない結果を書かない。正本は QLT-001 / QLT-012 と [ADR 0003](docs/adr/0003-diff-scoped-verification-and-result-reuse.md)。

---

## 4. 変更の進め方

[docs/DEVELOPMENT_WORKFLOW.md](docs/DEVELOPMENT_WORKFLOW.md) が正本。要約すると:

Issue → 正典経路の特定 → ブランチ（`<type>/<N>-<kebab>`）→（設計を変えるなら先に ADR）→ 最小の実装 →
必要な検証（成功結果を再利用）→ 規則 ID ごとの自己レビュー → PR（draft）→ 検証記録を整えて Ready → 必須 check（`make ci`）→ squash merge（`gh pr merge --squash --subject '<GIT-003 の件名>'`）→ 作業木とローカル枝を消す。

コミットは Conventional Commits（`type` と `scope` は英語、説明は日本語、末尾に `(#N)`）。形の正本は GIT-003。
`main` へ直接 push しない（ruleset が拒む）。

---

## 5. 完了報告の形

作業を終えたら必ず次を報告する。

```text
Issue / 規則 ID:
変更したファイルと振る舞い:
実行した検証コマンドと結果:
ドキュメント・スキーマの変更:
抑制（#[expect]）: none | 一覧
残るリスク:
```

調査だけを頼まれたときは、編集・コミット・push・PR 作成・外部状態の変更を行わない。

---

## 6. いまの状況

現在のタスクは [docs/todo/current.md](docs/todo/current.md)。GitHub Issue と PR が正で、そこは要約。

2026-10-22〜11-02 にリモートの無いローカルの git で 68 の Issue を進め、M0（仕様・規範文書・機械・機能の一覧・ロードマップ）と M1 の大半を置いた:
8 クレート（core / scan / application / ui-x11 / adapters-x11 / ui-win32 / adapters-windows / app）、X11 と Windows の窓（自前のタイトルバー・125 配色・6 言語の土台・倍率）、
方言と文字コードの判別（Pollock 2,290 ファイルで重み付き 9.66）、まばらな索引と裏の処理、選択・コピー・集計・パレット・読み方の選び直し。Windows の実機で M1 の `実装済み` を確かめた。
2026-11-04 に公開リポジトリへ移った（[ADR 0021](docs/adr/0021-public-repository-on-github.md)）。過程の記録（Issue のファイル）は持ち込まず、
残した記録（gate-proofs・benchmarks・FEATURES）の `Issue #N` は公開前の番号である。**残りの M1**（IME・ドラッグ＆ドロップ・判別の候補の提示・改行の混在・壊れた行の隔離・左の列の固定など）は current.md の表。

### 機能の状態を正直に書く

FEATURES.md の状態を `確認済み` にしてよいのは、テストか実測の記録を名指しできるときだけ。「全部カバーした」はフェーズ 1〜3 の全行が `確認済み` のときだけ言う（ADR 0008）。

---

## 7. 開発環境

Windows 11 ＋ WSL2 の Ubuntu。道具の入れ方は DEVELOPMENT_WORKFLOW 第 9 節、有無は `make doctor`。クラウドのコンテナでは `.claude/hooks/session-start.sh` が揃える。
表示は `make screenshot`（Xvfb）、操作は `xdotool`。人が触る手応え・Windows の見え・IME は施主の実機で確かめる。
🔴 **開発機に固有の情報は git 管理外の `CLAUDE.local.md` に置く（あれば作業の前に読む）。** リポジトリに置くのは対外的に要るものだけで、その中身を管理するファイル・コミット・PR 本文に写さない（GIT-005）。

---

## 8. 上位のポリシーと雛形の所在

このリポジトリの規約は **AYANE 厳格規約ポリシー**の 6 回目の適用例である（C++ / C で 3 回、Rust で xi-tools・nene-ruler・NeNeDocket に続く 4 回目）。
文書と機械の直接の出典は NeNeDocket。ポリシー本文・初期化手順・雛形は施主の private ワークスペースにあり、このリポジトリには置かない。
雛形で足りなかったものを見つけたら、このリポを直すと同時に施主へ還流点として報告する（還流点: リモートの無いリポジトリで GIT 規則をどこに置くか・ADR 0004 → 公開に戻した ADR 0021）。
