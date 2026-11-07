# 品質ゲート — NeNe Ledger

> Status: normative（規範）/ 2026-10-22 初版 / 2026-11-04 公開リポジトリへ移行（[ADR 0021](adr/0021-public-repository-on-github.md)）
> 本書は「いま何が機械で守られているか」の**正本**である。
> 規範の本文は [ARCHITECTURE_CONSTITUTION.md](ARCHITECTURE_CONSTITUTION.md) と
> [CODING_RULES.md](CODING_RULES.md)、機械側の実体はこの文書に対応する。出典は NeNeDocket `docs/QUALITY_GATES.md`。

🔴 Issue #2 で NeNeDocket の機械（lint の表・`clippy.toml`・`deny.toml`・`xtask`・`Makefile`・commit-msg フック）を移し、
`make prove`（26 件）と `cargo test -p xtask`（89 件）と実ファイルでの発火で**このリポジトリの道具で測った規則だけを active にした**（[gate-proofs](quality/gate-proofs.md)・[ADR 0010](adr/0010-rust-foundation-and-measured-limits.md)）。
Issue #9 で `crates/core` が生まれ、中核の実ファイルで CNF-007 / CNF-012 / CNF-013 を発火させて ARC-003 を、`make coverage` の下限で QLT-009 を active にした。2026-11-04 に公開リポジトリへ移った（[ADR 0021](adr/0021-public-repository-on-github.md)）。CI（`check.yml` → `make ci`）と ruleset を Issue #1 の PR #2 で実測し（gate-proofs 第 51 節）、GIT-001 / GIT-002 / GIT-004 / QLT-005 / QLT-012 を active にした。
🔴 記録に残る `Issue #N`（#1〜#68）は公開前のローカル履歴の番号で、GitHub の Issue とは別物（gate-proofs の節と対応する）。

完了条件は、差分の挙動と直接の依存・呼び出し元に必要な最小限の検証が成功し、その根拠・対象・結果・再利用条件が記録されていること。
既存の個別 target（`make lint` / `make test` / …）と対象指定を使う。全件（`make check REASON=…`）は既定ではない
（[ADR 0003](adr/0003-diff-scoped-verification-and-result-reuse.md)）。

---

## 1. ゲートの整合性規則（QLT-0xx）

### QLT-001 — 検証は差分と退行の根拠から選ぶ

変更した挙動と直接影響を受ける依存先・呼び出し元を確認し、その退行を検出できる最小限の検証を選ぶ。
「今回何が壊れる可能性を確認するか」を説明できない検証は実行しない。`make` の個別 target・`cargo test -p <crate> <name>` を優先し、大規模な自動選定基盤を作らない。
文書・コメント・ルール変更にアプリの動作テストは不要。フック・開発ツール変更はその道具の必要な正例・反例を短く確認する。

全件は共通基盤（lint の集合・`xtask`・workspace の設定）など、限定した検証では影響を覆えない具体的な理由がある場合のみ。
`make check REASON=<理由>` を使う。指定なし・空の理由は実行前に拒否する。「念のため」「習慣」は理由にならない。

- 機械強制: **planned**（`make check` は `REASON` が空なら実行前に拒む。対象と理由の妥当性はレビュー事項）

### QLT-002 — 警告は失敗する

`rustfmt` の差分・clippy の指摘・rustdoc の警告はすべて失敗にする。重大度の引き下げは禁止。

- 機械強制: **active**（clippy `-D warnings`・`cargo fmt --check`・`RUSTDOCFLAGS=-D warnings`）

### QLT-003 — baseline を作らない

既存の違反を「無かったこと」にする経路を作らない。クレート単位の `#![allow]` は `clippy::allow_attributes` が forbid なので**そもそも書けない**。
lint を `allow` に落とす設定・`--cap-lints`・`rustfmt.toml`・`baseline` と名の付くファイルは CNF-011 が拒む。
**例外機構は `#[expect]` だけで、それは理由と規則 ID を要求し、腐れば落ちる（RS-019）。**

- 機械強制: **active**（`allow_attributes` の forbid ＋ CNF-011）

### QLT-004 — 整形は検査であって修復ではない

整形は rustfmt の既定から決定的に決まる。検査がソースを書き換えて成功させることはしない。`rustfmt.toml` を置かない。

- 機械強制: **active**（`cargo fmt --all --check` ＋ CNF-011）

### QLT-005 — 判定は make と xtask にだけ置く

CI（`.github/workflows/check.yml`）は `make ci` を 1 行呼ぶだけ。判定は `make` の target と `xtask` にあり、手元でも同じ 1 行で同じ結果が出る。CI 側にだけ存在する検査を作らない（[ADR 0021](adr/0021-public-repository-on-github.md)）。

- 機械強制: **active**（`check.yml` は `make ci` だけを呼ぶ。PR #2 の実行 41832653816 で手元と同じ `Git conventions passed.` が出た・[gate-proofs](quality/gate-proofs.md) 第 51 節）

### QLT-006 — アーキテクチャは実行可能

モジュール境界・依存の向き・状態の経路は機械が検査する。散文だけのアーキテクチャ規則は、**「不能」と明記されない限り未完成**として扱う。

- 機械強制: **planned**（向きは CNF-013、循環は cargo。状態の所有者と隔離区画はレビュー事項のまま）

### QLT-007 — カスタムゲートには negative proof が要る

自作の検査は、「わざと違反させた入力で、意図した規則によって落ちること」と「正しい入力では通ること」の両方を示すまで信用しない。
`xtask` の各 CNF は正例・反例の単体テストを持ち `make test` で常に走る。lint・コンパイラ・cargo-deny・rustfmt は `make prove` が本物の道具に違反を食わせる。
証拠は [quality/gate-proofs.md](quality/gate-proofs.md) に記録する。ゲートを変えたら証拠も同じ変更で更新する。

🔴 **設定が在ることは、検査が効いていることの証明ではない。** xi-tools では `cognitive_complexity` が閾値だけ設定されて lint が無効という死んだ状態だった。
nene-ruler では `indexing_slicing` が固定長配列の定数添字を「安全」と判断して発火せず、slice に直して初めて落ちた。

- 機械強制: **planned**

### QLT-008 — 振る舞いの変更はテストを伴う

振る舞いを変えたら、いちばん狭い安定した境界にテストを足す。不具合の修正は、可能なら**先に落ちる回帰テスト**を書いてから直す。
NeNe Ledger では次の 3 つを、触れた変更で必ず回す。

1. **速い経路と参照実装の一致**: scan の SIMD・並列の経路は、core の素朴な参照実装と同じセル・同じ行の境界を出す（ADR 0006）
2. **切り方の不変**: 並列の読み取り・並べ替え・集計は、スレッドの数と区切りの位置を変えても同じ結果を出す（ARC-007）
3. **触っていない行のバイト一致**: 開いて、ある行だけを編集して保存したとき、他の行は元のバイト列と同じ（FR-003・ADR 0005）

- 機械強制: **planned**（レビュー事項。カバレッジ下限＝QLT-009 が部分的に代替する）

### QLT-009 — カバレッジは下げられない

中核（core / application）の region カバレッジ下限を 90% とする。閾値は上げてよいが下げてはならない。下げるには ADR が要る。
**置いていない層は「置いていない」と書く。** 分岐は stable の rustc では測れないので、単位は region（nene-ruler ADR 0010 の継承）。

- 機械強制: **active**（`make coverage` の `--fail-under-regions 90`。Issue #9 の実測は core の region 99.79%、閾値を実測より上に置くと終了 1・[gate-proofs](quality/gate-proofs.md) 第 8 節。測る前に毎回 `cargo llvm-cov clean --workspace` を走らせ、判定がそれまでに何を測ったかに依らないようにした（Issue #16・第 10 節）。application は Issue #19 で `COVERAGE_PACKAGES` に足した）

### QLT-010 — ゲートの弱体化はアーキテクチャ変更

閾値・`forbid` からの降格・lint の削除・`clippy.toml` の免除追加・モジュール境界・`make ci` の中身・`deny.toml` の緩和を変えるには明示的な根拠が要る。
MUST 規則を弱める変更には ADR が要る。**無関係な作業を通すために検査を切ることは禁止。**

- 機械強制: **不能**（判断そのものが対象。PR とレビューの手続きで担保する）

### QLT-011 — 依存は再現可能

道具の版は `rust-toolchain.toml` にだけ書く（`Makefile` にも書かない）。依存は `Cargo.lock` で固定し、ゲートは `--locked` で走る。
lock が更新される状態でのゲート通過を拒む。

- 機械強制: **active**（`--locked`・cargo-deny の取得元の制限＝ARC-013）

### QLT-012 — 検証の頻度は変更に従う

検証済みの実装・テスト・関連依存と必要な環境条件が変わらなければ、成功済み結果を統合で再利用する。
担当・工程・文書追記・コミット ID の変更だけでは再実行しない。関連する変更・失敗・具体的な未確認事項がある場合だけ、その範囲を再検証する。
PR に「確認する退行」「対象・依存」「検証結果（コマンド・結果・記録の所在）」「再利用（元の記録と不変性の根拠、または初回検証）」を残す。実装中とレビュー中は draft で、必須 check は draft では走らない。

- 機械強制: **active**（`make ci` が PR 本文の欄の存在を検査し（GIT-004）、`check.yml` は draft では走らない。CI の実行は gate-proofs 第 51 節。結果の真偽・再利用条件・失敗との因果はレビュー事項）

### QLT-013 — 環境依存の主張は正直に名付ける

単体テストは X サーバ・Windows・実機・実 OS 資源を必要としない。単体テストが通ったことを「実機で動く」の証拠として扱わない。
Xvfb の上での表示確認は「仮想の X サーバでの確認」であり、WSLg や Windows の実機での確認とは別に記録する。
**Windows で動かすまで「Windows で動く」と書かない。** ビルドが通ることは動くことの証拠ではない（NeNeDocket の Windows 版は組み立てただけで、長く Windows で動いていなかった）。
Windows の exe は `make build-windows` で組み立て、施主の実機で確かめて gate-proofs に記録する（2026-10-27 から・第 42 節。2026-10-22 の検疫（公開前の Issue #7）はこの exe では起きていない）。配る exe は ADR 0032 のとおりタグから GitHub Actions で作る。画面の確かめ方は [DEVELOPMENT_WORKFLOW.md](DEVELOPMENT_WORKFLOW.md) 第 9 節。

**Windows のターゲットの単体テストは `make test-windows` で、WSL から Windows の OS の上で走らせる**（GitHub Issue #211・2026-11-07 から）。`make test` は Linux のターゲットだけを走らせ、`make lint-windows` は組み立てるだけなので、ui-win32・adapters-windows のテストと、scan・store・app の Windows での振る舞いは、これまでどこでも走っていなかった（#200 で `key_intent` に足した Alt+Z・Alt+K を窓の `ALT_KEYS` に足し忘れたのを、テストでは見つけられなかった）。WSL の相互運用が無い機械では走らない（終了 2）。これは単体テストを Windows で走らせるだけで、窓・IME・倍率・クリップボードの施主の実機での確かめの代わりではない。CI では走らせていない（GitHub Actions の Windows の走り手は ADR 0032 の配る exe だけに使う）。

- 機械強制: **planned**

### QLT-014 — 速さの主張は実測で書く

「速い」「巨大なファイルでも」と書くときは、ファイルの大きさ・行数・列数・機械・道具の版と、比較相手（DuckDB・qsv・xsv・Modern CSV）の同じファイルでの数字を添える（[ADR 0007](adr/0007-own-engine-and-external-benchmarks.md)）。
測っていない大きさについて「動く」「速い」と書かない。目標（SPECIFICATION 第 2 節）と実測を混ぜない。

- 機械強制: **planned**（主張が実測を名指ししているかはレビュー事項。測る道具は `make bench-pollock`（正しく読めた率・Issue #24）と `make bench-corpus`（実データの突き合わせ・ADR 0023）、記録の形は [quality/benchmarks.md](quality/benchmarks.md)。速さの比較相手（DuckDB・qsv・xsv）の測り方は次の性能の Issue）

---

## 2. 規約検査の規則（CNF-0xx）— `xtask`

lint が見ないもの、つまり**このリポジトリ固有の規約**を検査する。依存ゼロで書き、`make conformance` から常に呼ぶ。
各規則には正例・反例の単体テストを付け、`make test` に結線する（QLT-007）。Issue #2 で NeNeDocket の `xtask` を写し、クレート名・CNF-012 の除外・CNF-013 の許可表・CNF-014 の対象だけを変える予定。

🔴 **`.rs` は、桁 0 の `#[cfg(test)]` の次の行が `mod` で始まる所より後を見ない**（テストは意図的な違反を書く場所）。
🔴 **検出語は `concat!` で分割して書く**（検査器が自分自身を違反として報告しないため）。

### CNF-001 — `Default` の禁止

`#[derive(Default)]`・`impl Default for`・`..Default::default()` を拒む。生成経路は唯一のファクトリに限る。

- 対応する規則: RS-003
- 機械強制: **active**（`xtask`）

### CNF-002 — 禁止構文

`dyn Any`・`OnceLock`・`LazyLock`・`lazy_static`・`proc-macro = true`・`XlibSurface`・`XCBSurface` を拒む。

- 対応する規則: RS-006 / RS-009 / RS-010 / ADR 0002
- 機械強制: **active**（`xtask`）

### CNF-003 — 1 ファイル 1 主要宣言

桁 0 の `struct` / `enum` / `trait` / `union` を数える。2 つ目を拒む。関数内のローカル型は数えない。

- 対応する規則: RS-012
- 機械強制: **active**（`xtask`）

### CNF-004 — 役割を語らない名前

型名の語尾 `Manager` / `Helper` / `Util` / `Utils` / `Common` と、モジュール名 `utils` / `helpers` / `managers` / `misc` / `common` を拒む。
判断が要る語（`Processor` 等）は拒まない。

- 対応する規則: RS-013
- 機械強制: **active**（`xtask`）

### CNF-005 — 不変条件を持つ型は単独のモジュールに置く

- 対応する規則: RS-003
- 機械強制: **planned**（「不変条件を持つ型」を構文で判定する方法が未定。NeNeDocket でも planned のまま）

### CNF-006 — `#[expect]` の `reason` は実在する規則 ID を引く

`reason = "<規則 ID>: <理由>"` の形で、ID が規範文書の `### ID` 見出しに実在すること。複数行の `#[expect]` も 1 件として読む。

- 対応する規則: RS-019
- 機械強制: **active**（`xtask`）

### CNF-007 — 中核クレートは `#![no_std]` を宣言する

名前が `-core` / `-application` で終わるクレートの `src/lib.rs` の先頭の属性行群に `#![no_std]` があること。

- 対応する規則: ARC-003 / RS-015
- 機械強制: **active**（`xtask`）

### CNF-008 — `build.rs` を置かない

`build.rs` そのものと、マニフェストの `build = ` を拒む。例外は無い。アイコンを埋める必要が出たら ADR で 1 つだけ足し、検査で固定する。

- 対応する規則: RS-010
- 機械強制: **active**（`xtask`）

### CNF-009 — 文書整合

規則 ID の重複定義・機械強制の状態の欠落・本文とマトリクスの不一致・active の証明行の欠落・未定義 ID の参照・切れた相対リンク・未置換の雛形値を拒む。

- 対応する規則: QLT-007 / ADR 0001
- 機械強制: **active**（`xtask`。実ファイルでの発火は [gate-proofs](quality/gate-proofs.md) 第 5 節）

### CNF-010 — `TODO` / `FIXME` に Issue 番号

`.rs` / `.toml` / `Makefile` のタスクの印は同じ行に `#N` を持つ。

- 対応する規則: QLT-008
- 機械強制: **active**（`xtask`）

### CNF-011 — baseline とゲート無力化の禁止

`baseline` / `suppressions` / `rustfmt.toml` の名を持つファイル、`= "allow"`・`--cap-lints`・`-A warnings` を拒む。

- 対応する規則: QLT-003 / QLT-004
- 機械強制: **active**（`xtask`）

### CNF-012 — 決定性の入力 API

`crates/` の `.rs` で `std::time` / `SystemTime` / `Instant` / `std::env` / `std::fs` / `std::thread` / `std::process::Command` / `std::net` / `rand::` を拒む。
`crates/adapters-x11/`・`crates/adapters-windows/`・`crates/scan/`・`crates/store/` は除外。`crates/app/` は `std::env::args` だけを許す。テストにも適用する。

- 対応する規則: ARC-007 / RS-015
- 機械強制: **active**（`xtask`。マクロ・別名・間接呼び出しは字句では見えない）

### CNF-013 — クレート間の依存は許可された向きだけ

`crates/*/Cargo.toml` と `xtask/Cargo.toml` の `[dependencies]` と `[target.….dependencies]` を [PROJECT_LAYOUT.md](PROJECT_LAYOUT.md) 第 2 節の許可表と照合する。内部クレートは向き、外部クレートは表に書いたものだけ。

- 対応する規則: ARC-002 / ARC-013
- 機械強制: **active**（`xtask`）

### CNF-014 — `unsafe` を許す 3 クレートの lint の表は workspace と同じ

`crates/adapters-windows/`・`crates/ui-win32/`・`crates/scan/` の `Cargo.toml` の `[lints.rust]` / `[lints.clippy]` が、workspace の表と同じであることを見る。違ってよいのは `unsafe_code = "deny"` だけ（Cargo は継承した表の 1 項目だけを変えられないので、3 クレートは表を写す・ADR 0002 / ADR 0006）。

- 対応する規則: RS-010 / QLT-003 / QLT-010
- 機械強制: **active**（`xtask`。NeNeDocket の 2 クレートの検査に `scan` を足した）

### CNF-015 — 読み取りの区画は書かない

`crates/scan/` の `.rs` の production コードで、書き込みで開く API（`OpenOptions` の `.write(` / `.append(` / `.truncate(` / `.create(`・`File::create`・書き込み可能な写像）と、開かずに書く・消す・名前を変える `std::fs` の関数（`fs::write` / `fs::remove_file` / `fs::remove_dir*` / `fs::rename` / `fs::copy` / `fs::create_dir*` / `fs::hard_link` / `fs::set_permissions`・Issue #12）を拒む。原本は保存の置き換えまで書かない（ADR 0005）。

- 対応する規則: ARC-001 / ARC-005
- 機械強制: **active**（`xtask`。字句検査なので別名・間接呼び出しは見えない）

---

## 3. 強制マトリクス

規範の各規則が、いまどの層で守られているか。**この表が実装と食い違ったら統合を止める。**
CNF-009（予定）が「本文に定義があるのにここに行が無い」「状態が本文と違う」「active に証明行が無い」を拒否する。

| 規則 | 状態 | 機械強制の実体 |
| --- | --- | --- |
| ARC-001 | planned | レビュー事項（CNF-004・`pub_use`・CNF-015 が一部を守る予定） |
| ARC-002 | active | cargo の循環拒否 ＋ CNF-013（[gate-proofs](quality/gate-proofs.md)） |
| ARC-003 | active | CNF-007（`#![no_std]`）＋ CNF-013（外部クレートの禁止）＋ E0433。実ファイルでの発火は [gate-proofs](quality/gate-proofs.md) 第 8 節 |
| ARC-004 | planned | レビュー事項 |
| ARC-005 | planned | レビュー事項（CNF-015 が scan の一部を守る予定） |
| ARC-006 | planned | 予定: CNF-002 と RS-014 が一部を守る |
| ARC-007 | active | CNF-012（[gate-proofs](quality/gate-proofs.md)） |
| ARC-008 | planned | レビュー事項 |
| ARC-009 | planned | |
| ARC-010 | planned | 予定: RS-005 が panic を塞ぐ |
| ARC-011 | planned | |
| ARC-012 | 不能 | QLT-010 の手続き |
| ARC-013 | active | `wildcard_dependencies`・`--locked`・cargo-deny・CNF-013（[gate-proofs](quality/gate-proofs.md)） |
| RS-001 | active | E0603（[gate-proofs](quality/gate-proofs.md)） |
| RS-002 | active | E0004 ＋ `wildcard_enum_match_arm`（[gate-proofs](quality/gate-proofs.md)） |
| RS-003 | active | E0381・E0063・CNF-001（[gate-proofs](quality/gate-proofs.md)） |
| RS-004 | active | `unwrap_used` / `expect_used` の forbid（[gate-proofs](quality/gate-proofs.md)） |
| RS-005 | active | forbid 群 ＋ `panic_in_result_fn` / `unwrap_in_result`（[gate-proofs](quality/gate-proofs.md)） |
| RS-006 | active | CNF-002（[gate-proofs](quality/gate-proofs.md)） |
| RS-007 | active | `as_conversions`（[gate-proofs](quality/gate-proofs.md)） |
| RS-008 | active | `missing_docs` 等（[gate-proofs](quality/gate-proofs.md)） |
| RS-009 | active | `unsafe_code` の forbid ＋ CNF-002（[gate-proofs](quality/gate-proofs.md)） |
| RS-010 | active | `unsafe_code`・CNF-002・CNF-008。unsafe を許す 3 クレートは `deny` と `#[expect]`・`undocumented_unsafe_blocks`・CNF-014（[gate-proofs](quality/gate-proofs.md)） |
| RS-011 | active | `cognitive_complexity`・`too_many_lines`（[gate-proofs](quality/gate-proofs.md)） |
| RS-012 | active | `mod_module_files` ＋ CNF-003（[gate-proofs](quality/gate-proofs.md)） |
| RS-013 | active | `disallowed-names` ＋ CNF-004（[gate-proofs](quality/gate-proofs.md)） |
| RS-014 | active | `print_stdout` / `print_stderr`（[gate-proofs](quality/gate-proofs.md)） |
| RS-015 | active | CNF-012（[gate-proofs](quality/gate-proofs.md)） |
| RS-016 | active | `disallowed-types` ＋ `iter_over_hash_type`（[gate-proofs](quality/gate-proofs.md)） |
| RS-017 | active | `overflow-checks`（[gate-proofs](quality/gate-proofs.md)） |
| RS-018 | active | `default_numeric_fallback`（[gate-proofs](quality/gate-proofs.md)） |
| RS-019 | active | forbid の抑制不能・腐った抑制・`#[allow]` の禁止・CNF-006（[gate-proofs](quality/gate-proofs.md)） |
| GIT-001 | active | PR テンプレート＋`make ci`（`Closes #N`）。CI の実行は gate-proofs 第 51 節 |
| GIT-002 | active | `make ci`（PR の head のブランチ名）＋ ruleset 26530802（PR 必須・force push と削除の禁止・gate-proofs 第 51 節） |
| GIT-003 | active | `.githooks/commit-msg` ＋ `make ci`（[gate-proofs](quality/gate-proofs.md) 第 6 節） |
| GIT-004 | active | PR テンプレート＋`make ci`（欄）＋ ruleset 26530802（squash のみ・`check` 必須・gate-proofs 第 51 節） |
| GIT-005 | active | `make ci`（管理するファイル・コミット・PR 本文の絶対パスの検出）。内容の判断はレビュー事項（[gate-proofs](quality/gate-proofs.md) 第 6 節） |
| QLT-001 | planned | `make check` の `REASON` 要求。理由の妥当性はレビュー事項 |
| QLT-002 | active | clippy `-D warnings`・rustfmt・rustdoc（[gate-proofs](quality/gate-proofs.md)） |
| QLT-003 | active | `allow_attributes` の forbid ＋ CNF-011（[gate-proofs](quality/gate-proofs.md)） |
| QLT-004 | active | `cargo fmt --check` ＋ CNF-011（[gate-proofs](quality/gate-proofs.md)） |
| QLT-005 | active | `check.yml` → `make ci` だけ。CI と手元が同じ出力（gate-proofs 第 51 節） |
| QLT-006 | planned | |
| QLT-007 | planned | 予定: `make prove` と `xtask` のテスト |
| QLT-008 | planned | レビュー事項 |
| QLT-009 | active | `make coverage`（region 90%）。実測と発火は [gate-proofs](quality/gate-proofs.md) 第 8 節 |
| QLT-010 | 不能 | PR とレビューの手続き |
| QLT-011 | active | `rust-toolchain.toml` ＋ `--locked`（[gate-proofs](quality/gate-proofs.md)） |
| QLT-012 | active | `make ci` が PR 本文の欄を見る・draft では走らない（gate-proofs 第 51 節）。中身はレビュー事項 |
| QLT-013 | planned | |
| QLT-014 | planned | |
| CNF-001 | active | xtask / `cargo test -p xtask`（[gate-proofs](quality/gate-proofs.md)） |
| CNF-002 | active | xtask / `cargo test -p xtask`（[gate-proofs](quality/gate-proofs.md)） |
| CNF-003 | active | xtask / `cargo test -p xtask`（[gate-proofs](quality/gate-proofs.md)） |
| CNF-004 | active | xtask / `cargo test -p xtask`（[gate-proofs](quality/gate-proofs.md)） |
| CNF-005 | planned | |
| CNF-006 | active | xtask / `cargo test -p xtask`（[gate-proofs](quality/gate-proofs.md)） |
| CNF-007 | active | xtask / `cargo test -p xtask`（[gate-proofs](quality/gate-proofs.md)） |
| CNF-008 | active | xtask / `cargo test -p xtask`（[gate-proofs](quality/gate-proofs.md)） |
| CNF-009 | active | xtask / `cargo test -p xtask`（[gate-proofs](quality/gate-proofs.md)） |
| CNF-010 | active | xtask / `cargo test -p xtask`（[gate-proofs](quality/gate-proofs.md)） |
| CNF-011 | active | xtask / `cargo test -p xtask`（[gate-proofs](quality/gate-proofs.md)） |
| CNF-012 | active | xtask / `cargo test -p xtask`（[gate-proofs](quality/gate-proofs.md)） |
| CNF-013 | active | xtask / `cargo test -p xtask`（[gate-proofs](quality/gate-proofs.md)） |
| CNF-014 | active | xtask / `cargo test -p xtask`（[gate-proofs](quality/gate-proofs.md)） |
| CNF-015 | active | xtask / `cargo test -p xtask`（[gate-proofs](quality/gate-proofs.md)） |

---

## 4. `make` の target に入っている層

`coverage`・`build-windows`・`screenshot` は対象の製品クレートが生まれてから使う。

| target | 目的 | 実体 |
| --- | --- | --- |
| `fmt-check` | 文字列としての正本 | rustfmt の既定 |
| `lint` | 型安全・網羅性・警告ゼロ・二段構えの抑制 | clippy ＋ `[workspace.lints]` ＋ `clippy.toml` |
| `lint-windows` | Windows のクレートを Windows のターゲットで検査する | clippy `--target x86_64-pc-windows-gnu` |
| `test` | 振る舞い・検査器の正例と反例 | `cargo test --workspace` |
| `test-windows` | Windows のターゲットの単体テストを Windows で走らせる（WSL の上でだけ・Issue #211） | `tools/windows-tests.sh`（`cargo test --no-run --target x86_64-pc-windows-gnu` ＋ WSL の相互運用 ＋ `subst`） |
| `conformance` | NeNe Ledger 固有（CNF-0xx） | `xtask conformance` |
| `deny` | ライセンス・脆弱性・重複版・取得元 | cargo-deny / `deny.toml` |
| `doc-check` | rustdoc の警告 | `RUSTDOCFLAGS=-D warnings` |
| `build` / `build-windows` | release の 1 バイナリ / Windows の exe | `cargo build --release --locked` |
| `prove` | ゲート発火の証明 | `xtask prove` |
| `ci` | CI の必須 check: Git 規約・PR の記録の欄・空白・絶対パス | `xtask validate-git` ＋ `git diff --check origin/main...HEAD` |
| `coverage` | 中核の region カバレッジ 90%（QLT-009） | cargo-llvm-cov |

---

## 5. マージゲート

`main` へのマージは次を必須とする（[ADR 0021](adr/0021-public-repository-on-github.md)）。

- PR であること（`main` への直接の push・force push・削除は ruleset が拒む）
- `check`（`make ci`）が成功していること（CI の必須 check）
- squash merge であること（ruleset の `allowed_merge_methods`）
- PR 本文の欄が埋まっていること（`make ci` が見る）

🔴 **リポジトリ設定（ruleset）を適用したら、その事実を [quality/gate-proofs.md](quality/gate-proofs.md) に記録する。**
**設定していないものを「必須になっている」と書かない。** ruleset 26530802 と CI の実行は gate-proofs 第 51 節に記録してある（2026-11-04）。
