# ゲート発火の証明 — NeNe Ledger

> 記録: 2026-10-22 / Issue #1（文書だけ・第 2 節）、Issue #2（機械の移植・第 3〜7 節）、Issue #9（中核の最初のクレート・第 8 節）、Issue #11 / #12（scan・第 9 節・2026-10-23）、Issue #16（カバレッジの作業領域・第 10 節）、Issue #18（最初の外部の依存・第 11 節）、Issue #19（application・第 12 節）、Issue #20（最初の窓・第 13 節）、Issue #21（裏の索引と状態の行・第 14 節）、Issue #22（キーでの移動・第 15 節）、Issue #23（閉じない引用符と判別・第 16 節）。公開後は GitHub の Issue #1（CI と ruleset・第 51 節）。以下は実行した結果のみ。
> 関連: QLT-007 / QLT-011 / QLT-013 / ADR 0001 / ADR 0010
> 🔴 2026-11-04 に公開リポジトリへ移った（ADR 0021）。本書の `Issue #N`（#1〜#68）は**公開前のローカル履歴の番号**で、GitHub の Issue とは別物。節の番号と記録はそのまま残す。

## 1. 環境と範囲

WSL2 の Ubuntu 24.04.5 LTS（x86_64）。Issue #1 の時点では Rust の道具が無かった（`cargo: command not found`）。
Issue #2（2026-10-22）: rustc 1.98.0（88d9e12ae 2026-08-18）/ clippy・rustfmt（同じツールチェーン）/ cargo-deny 0.20.2。rustup は apt の 1.26。
これらは実測時の版であり、今後使う版の正本は `rust-toolchain.toml`。Issue #2 の時点で製品コードは 0。workspace のメンバーは `xtask` だけ。

## 2. Issue #1 の文書整合（手で確かめたもの）

CNF-009（文書整合）は planned である。代わりに Issue #1 で、統合の前に Python の使い捨てのスクリプトで次を確かめた（スクリプトはリポジトリに置かない。検査器は Issue #2 で `xtask` に置く・ADR 0001 の決定 8）。

| 確かめたこと | 結果 |
| --- | --- |
| 4 つの規範文書の `### ID` の定義の数と、強制マトリクスの行の数 | 定義 66・行 66。漏れ・余り・重複なし |
| 定義ごとの「機械強制」の状態とマトリクスの状態の一致 | 不一致なし |
| active の行 | 0 件 |
| 全 `.md` の規則 ID（ARC / RS / QLT / CNF / GIT）の参照が定義に在るか | 未定義の参照なし |
| 全 `.md` の FR-NNN の参照が SPECIFICATION の表に在るか | 未定義の参照なし |
| 全 `.md` の相対リンク | 切れたリンクなし（本書を置いた後に再確認） |
| ADR の番号の参照 | 0001〜0007 は在る。0008 以降は NeNeDocket・nene-ruler・NeNe Nib の ADR を名指ししたものだけ |

🔴 これは手で 1 回確かめた記録。Issue #2 で CNF-009 を機械に置いた（第 5 節）。

## 3. `make prove` — 本物の道具に違反を食わせる（Issue #2）

`make prove` は `target/proofs/workspace` に `probe` だけの workspace を作り、根の `Cargo.toml` の lint の表をそのまま写して、違反を 1 つずつ書いては戻す。結果は `target/proofs/results.json`。実行 8.3 秒。

| # | 規則 | 命令 | 終了 | 実出力 |
| --- | --- | --- | --- | --- |
| P-01 | RS-019 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error[E0453]: expect(clippy::unwrap_used) incompatible with previous forbid` |
| P-02 | RS-019 | 同上 | 101 | `error: this lint expectation is unfulfilled` |
| P-03 | QLT-003 | 同上 | 101 | `error: #[allow] attribute found` |
| P-04 | RS-002 | 同上 | 101 | `error: wildcard matches only a single variant and will also match any future added variants` |
| P-05 | RS-002 | 同上 | 101 | `error[E0004]: non-exhaustive patterns: 'Mode::Mm' not covered` |
| P-06 | RS-007 | 同上 | 101 | `error: using a potentially dangerous silent 'as' conversion` |
| P-07 | RS-016 | 同上 | 101 | `error: use of a disallowed type 'std::collections::HashMap'` |
| P-08 | RS-018 | 同上 | 101 | `error: default numeric fallback might occur` |
| P-09 | RS-008 | 同上 | 101 | `error: missing documentation for a function` |
| P-10 | RS-012 | 同上 | 101 | `error: 'mod.rs' files are not allowed, found 'probe/src/probe10/mod.rs'` |
| P-11 | RS-011 | 同上 | 101 | `error: the function has a cognitive complexity of (13/10)` |
| P-12 | RS-011 | 同上 | 101 | `error: this function has too many lines (62/60)` |
| P-13 | RS-005 | 同上 | 101 | `error: used 'unwrap()' on 'Some' value` |
| P-14 | RS-005 | 同上 | 101 | `error: indexing may panic` |
| P-15 | RS-001 | 同上 | 101 | `error[E0603]: tuple struct constructor 'Id' is private` |
| P-16 | RS-003 | 同上 | 101 | `error[E0381]: used binding 'z' isn't initialized` |
| P-17 | RS-003 | 同上 | 101 | `error[E0063]: missing field 'b' in initializer of 'Config'` |
| P-18 | RS-010 | 同上 | 101 | `error: usage of an 'unsafe' block` |
| P-19 | RS-014 | 同上 | 101 | `error: use of 'println!'` |
| P-26 | RS-013 | 同上 | 101 | `error: use of a disallowed/placeholder name 'data'` |
| P-20 | QLT-004 | `cargo fmt --all --check` | 1 | `Diff in target/proofs/workspace/probe/src/main.rs:1:` |
| P-21 | RS-017 | `cargo run --release -p probe` | 101 | `attempt to add with overflow` |
| P-22 | QLT-011 | `cargo check --locked`（Cargo.lock を消した状態） | 101 | `error: cannot create the lock file ... because --locked was passed to prevent this` |
| P-23 | ARC-002 | `cargo check`（a ⇄ b の循環） | 101 | `error: cyclic package dependency: package 'a v0.0.0 (target/proofs/cycle/a)' depends on itself.` |
| P-24 | ARC-013 | `cargo deny --locked check licenses --deny license-not-encountered`（`allow = []`） | 4 | `rejected: license is not explicitly allowed` |
| P-25 | ARC-013 | 同上（`allow = ["MIT", "Apache-2.0"]`） | 4 | `error[license-not-encountered]: license was not encountered` |

`Gate proofs passed: 26 real-tool proofs`。P-24 / P-25 は自分のクレートのライセンス（`LicenseRef-NeNe-Ledger-1.0`・ADR 0009）で測った。

## 4. `cargo test -p xtask` — 自作の検査の正例と反例（Issue #2）

`make test`: **89 件すべて成功**（NeNeDocket の 80 件から、CNF-015 の差し替え・CNF-012 / CNF-013 の scan・GIT-004 の統合記録・GIT-005 の絶対パスのテストを足し引きした）。
`make lint`（clippy・全 target）・`make fmt-check`・`make doc-check`・`make deny`（`advisories ok, bans ok, licenses ok, sources ok`）・`make conformance`（`規約違反なし（検査 55 ファイル）`）も通った。

## 5. 実ファイルでの規約検査の発火（Issue #2）

実際のファイルに違反を 1 つずつ入れて `xtask conformance` を走らせ、すぐ元に戻した。戻した後は `規約違反なし`。

| 規則 | 入れた違反 | 実出力 |
| --- | --- | --- |
| CNF-009 | QUALITY_GATES のマトリクスで `QLT-006` を `active` に書き換えた | `QLT-006 の本文の状態とマトリクスの行が一致しない` と `QLT-006 は active だが証明行が無い` の 2 件 |
| CNF-010 | `Makefile` の末尾に Issue 番号の無いタスクの印 | `Makefile:116 — タスクの印には同じ行に Issue 番号（#N）が要る` |
| CNF-011 | `Cargo.toml` の `dbg_macro` を `"allow"` に | `Cargo.toml:77 — ゲートを無力化する設定である` |
| CNF-013 | `xtask/Cargo.toml` に `serde = "1"` | `xtask/Cargo.toml:17 — serde への依存は xtask/ に許されていない`（`cargo run --locked` では、その前に cargo が lock の更新を拒んだ。QLT-011 が先に効く） |

## 6. Git 規約の発火（Issue #2）

| 規則 | 入れた違反 | 実出力 |
| --- | --- | --- |
| GIT-003 | `.githooks/commit-msg` に件名 `update docs` | `GIT-003: subject must be ...`・終了 1。正しい件名 `build: 検査を移す (#2)` は終了 0 |
| GIT-004 | 統合記録が空のまま作業ブランチで `make validate-git` | 9 つの欄それぞれに `GIT-004: docs/issues/0002-port-machinery.md の統合記録の ... が空か、無い`（公開前の検証器。PR 本文を見る今の検証器の実測は第 51 節） |
| GIT-002 | 名前が形に合わない枝 `wip-branch` で `xtask validate-git` | `GIT-002: invalid branch name 'wip-branch'` |
| GIT-005 | 管理している `README.md` にホームディレクトリの絶対パスの行を足した | `GIT-005: README.md:32 に開発機に固有の絶対パスがある` |
| GIT-001 / GIT-003 / GIT-005 | `main` で `xtask validate-git`（全履歴 9 コミットと管理する全ファイル） | `Git conventions passed.`。存在しない Issue を引くコミットは単体テストでだけ見た（履歴に違反を作らないため） |

## 8. 中核の最初のクレート（Issue #9）

`crates/core`（`nene-ledger-core`・`#![no_std]`・依存 0）を置いた。`cargo test -p nene-ledger-core` 25 件成功（乱数で作った 5,000 の入力 × 6 方言で往復のバイト一致を含む・種は固定）。

| 規則 | 入れた違反（実ファイル） | 実出力 |
| --- | --- | --- |
| CNF-007 | `crates/core/src/lib.rs` の `#![no_std]` を消した | `nene-ledger-core は中核クレートだが、先頭の属性に #![no_std] が無い（ARC-003）` |
| CNF-012 / E0433 | `crates/core/src/byte_range.rs` の本番のコードに `std::time::SystemTime::now()` | xtask: `byte_range.rs:2 — std::time はアダプタ・scan・store の外で読まない`。`cargo check`: `error[E0433]: cannot find module or crate std in this scope` |
| CNF-013 | `crates/core/Cargo.toml` に `regex = "1"` | `regex への依存は crates/core/ に許されていない` |
| QLT-009 | `cargo llvm-cov -p nene-ledger-core --fail-under-regions 100`（実測 99.79% より上） | 終了 1 |

`make coverage`（下限 90%）: core の region **99.79%**（969 中 2 を逃す）・関数 100%・行 99.82%。

🔴 記録の手違い: 最初に CNF-012 を試したとき、違反の行をファイルの末尾（テストのモジュールの後ろ）に足したので発火しなかった。CNF-012 は本番のコード（桁 0 の `#[cfg(test)]` の次が `mod` の所より前）だけを見る仕様どおり。本番のコードに入れ直して発火を確かめた。

## 9. 読み取りの区画（Issue #11）

`crates/scan`（`nene-ledger-scan`・依存は core だけ・`unsafe` は 0）を置いた。`cargo test -p nene-ledger-scan` はユニット 13 件・結合 8 件が成功（結合は `CARGO_TARGET_TMPDIR` の一時ファイル）。違反を一時的に入れ、確かめた後に元へ戻した（差分の無いことを `diff -q` で確認）。

| 規則 | 入れた違反（実ファイル） | 実出力 |
| --- | --- | --- |
| CNF-014 | `crates/scan/Cargo.toml` の `mem_forget = "forbid"` を `"deny"` に | `crates/scan/Cargo.toml:1 — [lints.clippy] が workspace の表と違う。違ってよいのは unsafe_code = "deny" だけ` |
| CNF-015 | `crates/scan/src/source.rs` の本番のコードに `File::create(path)` | `source.rs:27 — File::create は書き込みの経路である。scan は原本を読むだけ（ADR 0005）` |
| CNF-015 | 同じファイルの `OpenOptions` に `.write(true)` | `source.rs:90 — .write(true は書き込みの経路である` |

🔴 見つかった穴（Issue #11）: 同じ場所に `std::fs::write(path, b"")` を書くと、CNF-015 は**発火しなかった**（`規約違反なし（検査 110 ファイル）`）。字句の表に無かったため。
Issue #12 で表に `std::fs` の書き込み・消去・名前の変更の関数を足した。同じ違反で `source.rs:27 — fs::write( は書き込みの経路である。scan は原本を読むだけ（ADR 0005）`（`make conformance` は失敗）を確かめた。

`unsafe` の 2 段（Issue #11 では未実施・Issue #12 で実施）。`crates/scan/src/source.rs` に、固定長の配列の 0 番目を `get_unchecked` で読む公開関数を一時的に置いた:

| 規則 | 入れた形 | 実出力 |
| --- | --- | --- |
| RS-010（`unsafe_code` の deny） | `// SAFETY:` はあるが `#[expect]` が無い | `error: usage of an unsafe block`（`-D unsafe-code`） |
| RS-010（`undocumented_unsafe_blocks`） | `#[expect(unsafe_code, reason = "RS-010: …")]` はあるが `// SAFETY:` が無い | `error: unsafe block missing a safety comment` |
| RS-010（正しい形） | `#[expect]` と `// SAFETY:` の両方 | 終了 0 |

参考: `cargo llvm-cov -p nene-ledger-scan` の region **96.44%**（scan は QLT-009 の下限の対象外。逃しているのは読み出しの失敗・`Interrupted`・型変換の失敗の経路）。

## 10. カバレッジの判定が履歴に依らないこと（Issue #16）

Issue #15 で `make coverage` が誤って落ちた。原因は、残った作業領域（`target/llvm-cov-target`）が集計に混ざったこと。同じ手順を直す前と後で回した:

| 手順 | 直す前 | 直した後（測る前に `cargo llvm-cov clean --workspace`） |
| --- | --- | --- |
| `cargo llvm-cov clean --workspace` → 6ac44e6（Issue #13）で `cargo llvm-cov -p nene-ledger-scan` → 今の版に戻して `make coverage` | region 2,106 中 243 を逃す・**88.46%**・終了 2 | region 1,867 中 4 を逃す・**99.79%**・終了 0 |

直す前でも、scan を測った後にコミットを跨がず core の 1 関数を書き換えただけでは再現しなかった（99.79%）。混ざる条件は特定していない。直し方は条件に依らない（毎回消す）。

## 11. 最初の外部の依存（Issue #18）

scan に `encoding_rs` 0.8.42 を足すとき、許可を書く前と後で回した（実ファイル・本物の依存）:

| 規則 | 許可を書く前 | 書いた後 |
| --- | --- | --- |
| ARC-013（cargo-deny） | `deny.toml` の `allow` が自前のライセンスだけ → `make deny`: `licenses FAILED`・終了 4 | `cargo deny list` の実測（Apache-2.0・BSD-3-Clause・MIT）を足す → `licenses ok` |
| CNF-013 | `crates/scan/Cargo.toml` に `encoding_rs` → `crates/scan/Cargo.toml:14 — encoding_rs への依存は crates/scan/ に許されていない`・終了 1 | xtask の許可表に足す → `規約違反なし` |

`cargo tree -p nene-ledger-scan -e normal`: `encoding_rs` が `cfg-if`・`core_detect`・`multiversion_no_op`（手続きマクロ）・`scopeguard`・`simdutf8` を連れてくる。0.8.35 なら `cfg-if` だけだが、0.8.40 の正しさの直し（2 バイトの文字コードの逐次の復号の境目・panic のときの初期化されていない記憶）を取るため 0.8.42 にした（CODING_RULES 第 5 節）。

## 12. application の最初の実ファイル（Issue #19）

`crates/application`（`#![no_std]`・依存は core だけ）を置いた。違反を一時的に入れ、確かめた後に元へ戻した（`diff -q`）:

| 規則 | 入れた違反 | 実出力 |
| --- | --- | --- |
| CNF-007 | `crates/application/src/lib.rs` の `#![no_std]` を消した | `nene-ledger-application は中核クレートだが、先頭の属性に #![no_std] が無い（ARC-003）` |
| CNF-012 | `table_view.rs` の本番のコードに `use std::time::Instant;` | `table_view.rs:4 — std::time はアダプタ・scan・store の外で読まない（ARC-007）` |
| CNF-013 | `crates/application/Cargo.toml` に `nene-ledger-scan` | `make conformance` は `--locked` でロックファイルの更新を拒んで終了 101（xtask に届く前に止まる）。xtask を `--offline` で直接走らせると `Cargo.toml:16 — nene-ledger-scan への依存は crates/application/ に許されていない` |

`make coverage` に application を足した（QLT-009）。core と application をあわせて region **99.24%**。

## 13. 最初の窓（Issue #20）— 仮想の X サーバでの確認（QLT-013）

`make screenshot`（Xvfb の 1024 × 640 × 24 に `crates/app/fixtures/screenshot.csv` を開いて `import` で撮る）の画像を目で見た（2026-10-23）。

- 1 回目: 表は出たが、(1) 改行の印 `↵`（U+21B5）が Noto Sans CJK JP に無く、字の無い印になった (2) 行の帯を窓の幅いっぱいに罫線の色で塗ったので、表の右に灰色の縦の帯が出た。印を `¶` に替え、帯を表の幅までにした（どちらも単体テストを足した）
- 2 回目: BOM の後ろから切れて見出しの引用符が外れる・日本語が描ける・先頭の 0（00123）と 1.20E+03 がそのまま・引用の中の区切り（Smith, John）と `""`・改行は `¶`・タブは `→`・欠けたセル（6 行目）は灰色で空のセルと区別・長い備考は列の右端で切り取られ、右は白。崩れは見つからなかった
- 🔴 この開発機（WSLg）では `/tmp/.X11-unix` が読み取り専用で、Xvfb が Unix の接続口を作れずに起動しなかった（`_XSERVTransSocketCreateListener: failed to bind listener`）。`make screenshot` は Xvfb を TCP（`-nolisten unix -listen tcp`）で待たせ、`DISPLAY=localhost:99` でつなぐ形にした
- 🔴 これは仮想の X サーバでの確認で、実機（WSLg・Xorg）のウィンドウマネージャーの下での移動・大きさの変更・題名は見ていない。Windows の窓は無い

ui-x11 の実ファイルで、違反を一時的に入れて戻した（`diff -q`）:

| 規則 | 入れた違反 | 実出力 |
| --- | --- | --- |
| CNF-002 | `frame_surface.rs` に `use cairo::XlibSurface;` | `frame_surface.rs:4 — XlibSurface を書いている。描画は ImageSurface に完成させてから転送する。X の surface は unsafe を要する（ADR 0002）` |
| CNF-012 | `running.rs` に `use std::env::var;` | `running.rs:7 — std::env はアダプタ・scan・store の外で読まない（ARC-007）` |

依存の許可（ARC-013）: `x11rb` と `cairo-rs` を足した直後の `make deny` は `licenses FAILED`（`target-lexicon` の `Apache-2.0 WITH LLVM-exception`）。実測どおり `deny.toml` に足して `licenses ok`。版の重複は無い（`cargo tree -d`）。

## 14. 裏の索引と状態の行（Issue #21）— 仮想の X サーバでの確認（QLT-013）

`make screenshot` の画像を目で見た（2026-10-23・debug の組み立て）:

| 開いたファイル | 撮った時点の状態の行 | 確かめたこと |
| --- | --- | --- |
| `crates/app/fixtures/screenshot.csv` | `7 行` | 索引が終わり、作業のスレッドの合図で窓が起き、結果を取り込んで描き直した |
| 約 300 MB・2 行目から閉じない引用符（`target/` に作って撮った後に消した） | `索引を作っています 33%` | 作っている途中の進み具合。🔴 閉じない引用符が判別の標本（64 KiB）の中にあるので、判別は引用を使わない方言を選び、引用符は文字として読まれた（下の「見つけたこと」） |
| 約 300 MB・閉じない引用符を 175,026 バイト目（標本の外）に置いた | `索引を作っています 42% · 175,026 バイト目の引用符が閉じていないかもしれません` | ADR 0018 の決定 1 の途中の知らせ。位置は手で数えて一致（見出し 17 ＋ 35 バイト × 5,000 ＋ 9） |

見つけたこと: 方言の判別（Issue #10）は、閉じない引用符の残る候補を 0 点にする。閉じない引用符が標本の中にあると、ADR 0018 の流れ（厳格に読み、見つけたら尋ねる）に乗らず、引用を使わない方言が選ばれる。ほかの行の正しい引用（`"Smith, John"`）も文字として読まれうる。扱いは別の Issue で決める。

## 15. キーでの縦の移動（Issue #22）— 仮想の X サーバでの操作（QLT-013）

`target/` に作った 5,000 行（＋ 見出し 1 行）のファイルを開き、索引を作り終えた後に xdotool でキーを送って撮った（2026-10-23）:

| 送ったキー | 画面 |
| --- | --- |
| PageDown | 先頭の行が 1 → 20（1 画面に全部入る 19 行ぶん） |
| End | 4,983〜5,001 行。最後の行が全部入る所で止まり、下は空き。状態の行は `5,001 行` |
| Home → ↓ × 3 | 先頭の行が 4 |

作り替え（CNF-003 で `Reader` を別のファイルへ）の後に End をもう一度送り、5,001 行目まで動くことを確かめた。

- 窓は `xwininfo -root -children` で探した。`xdotool search --name` はこの窓を見つけなかった（題名は `_NET_WM_NAME` だけに付けている）
- 窓の管理者が居ないので、xdotool は `XGetInputFocus returned the focused window of 1` と警告するが、`--window` で送ったキーは届いた
- 🔴 列の幅を見えている行の中身から決めるので、動くと列の幅が変わる（末尾は数字が長いぶん広い）。幅を固定するのは VIEW-02

## 16. 閉じない引用符が判別の標本にあるとき（Issue #23）— 仮想の X サーバでの確認（QLT-013）

`target/` に作った約 300 MB のファイル（見出し・`1,"Smith, John",正しい引用`・`2,山田,"閉じない引用符`・普通の行の繰り返し）を開いて撮った（2026-10-23・debug の組み立て。撮った後にファイルは消した）:

| 時点 | 画面 |
| --- | --- |
| 開いて 2 秒 | 最初の画面が出ている（範囲の取り出しが閉じない引用符のレコードの終わりを探しに行かない）。判別は `"` の引用を選び、`Smith, John` は 1 つのセル。3 行目は厳格に読まれ、閉じない引用符から先が 1 つのセル（`閉じない引用符¶3,鈴木,ふつうの行¶…`・1 MiB で切り詰め）。下の行は `索引を作っています 34% · 58 バイト目の引用符が閉じていないかもしれません` |
| 索引の後 | `3 行 · 58 バイト目の引用符が最後まで閉じていません` |

位置 58 は手で数えて一致（見出し 17 ＋ 1 行目 32 ＋ `2,山田,` の 9）。Issue #21 の同じ形のファイル（gate-proofs 第 14 節）では、判別が引用を使わない方言を選び、引用符は文字として読まれていた。

## 17. 自前のタイトルバーと縁（Issue #27）— WSLg の上での施主の目視（QLT-013）

WSLg のウィンドウマネージャーの `_NET_SUPPORTED` は `_NET_WM_MOVERESIZE`・`_NET_WM_STATE`・`_NET_WM_STATE_MAXIMIZED_VERT` / `_HORZ`・`_NET_FRAME_EXTENTS` などを含む（`xprop -root`・2026-10-26）。
Pollock の `results/source.csv` を開き、施主が WSLg の上で見て触った（2026-10-26）:

| 確かめたこと | 結果 |
| --- | --- |
| OS の枠と題名の帯が消え、自前のタイトルバー（題名・最小化・最大化・閉じる）だけになる | 良し |
| タイトルバーのドラッグで移動・ダブルクリックで最大化と元に戻す | 良し |
| ボタンに指を乗せた色（閉じるは赤）・最小化・最大化・閉じる | 良し |
| 縁と角でカーソルが変わり、ドラッグでリサイズ | 良し |

- 窓の `_MOTIF_WM_HINTS` は `0x2, 0x0, 0x0, 0x0, 0x0`（`xprop -id`）。ウィンドウマネージャーは影の分の外枠の窓（四方 32 画素）を残し、`_NET_FRAME_EXTENTS` は `38, 38, 59, 38` と飾りのある値のまま
- xdotool の `mousemove --window … click` は WSLg では窓に届かなかった（ポインターがルートの上と扱われる）。操作の確認は施主の手による
- 当たり判定と描画命令は core と application の純関数のテストで固定した（`window_chrome`・`document`）。ボタンの押し・離し・ドラッグの閾値・ダブルクリックは ui-x11 の `pointer` のテスト

## 18. 見出しの行の固定と列の幅の固定（Issue #26）— 仮想の X サーバでの操作（QLT-013）

`target/` に作った 5,000 行（＋ 見出し `id,name,city,amount`）のファイルを開き、索引を作り終えた後に xdotool でキーを送って撮った（2026-10-26。撮った後にファイルは消した）:

| 送ったキー | 画面 |
| --- | --- |
| （開いた直後） | タイトルバーの下に見出しの行（行番号 1）、その下に本体の行番号 2〜 |
| PageDown | 見出しの行は上に残り、本体の先頭は行番号 19（1 画面に全部入る本体の 17 行ぶん）。列の幅は開いたときと同じ |
| End | 見出しの行は上に残り、本体は 4,985〜5,001 行で止まる。列の幅は同じ（行番号の列だけがレコードの数の 4 桁に広がる） |

- 🔴 列の幅は先頭の 200 行で決めるので、`id` の列は 3 字のまま。4 桁の値はセルいっぱいに切り取られて見える

## 19. セルのカーソルと選択・横の移動（Issue #28）— 仮想の X サーバでの操作（QLT-013）

`target/` に作った 300 行 × 20 列（見出し `column_01`〜`column_20`）のファイルを開き、xdotool でキーを送って撮った（2026-10-26。撮った後にファイルは消した）:

| 送ったキー | 画面 |
| --- | --- |
| ↓ ↓ → → | カーソルの枠が本体の 3 番目の行（行番号 4）の 3 列目 |
| Shift+↓ ↓ → | 行番号 4〜6 × 3〜4 列目の矩形が選択の色、カーソルの枠は行番号 6 の 4 列目 |
| End | 表示の最初の列が 13 列目になり、20 列目の行番号 6 のセルにカーソルの枠（選択は畳む）。行番号の列は左に残り、見出しは列と一緒に動く |

- WSLg の上で施主がキー・Shift・クリック・横のホイールを触り、見出しの固定（Issue #26）と状態の行（Issue #29）とあわせて良しとした（2026-10-26）
- 🔴 最初に撮った画像は、組み立て直す前の古い実行ファイルのものだった（End で末尾の行へ飛んだ）。`cargo build -p nene-ledger` の後に撮り直した

## 20. 状態の行の読み方とカーソル（Issue #29）— 仮想の X サーバでの確認（QLT-013）

Pollock の `results/source.csv`（84 レコード・9 列）を開き、xdotool で ↓ ↓ → を送って撮った（2026-10-26）。状態の行は `84 行 · UTF-8 · カンマ区切り · LF · 9 列 · 4 行目 · 2 列目`。カーソルの枠は行番号 4 の 2 列目（`00:30`）にあり、状態の行の位置と一致した。

## 21. 選択のコピー（Issue #30）— 仮想の X サーバでの読み出し（QLT-013）

`target/` に作った 4 行のファイル（見出し `id,name,note`・改行を含むセル・`007`・タブを含むセル）を開き、xdotool で Shift+↓ ↓ Shift+End Ctrl+C を送った後、別のクライアント（使い捨ての python-xlib のスクリプト・作業フォルダの仮想環境）から `CLIPBOARD` を読んだ（2026-10-26。読んだ後にファイルとスクリプトは消した）:

| 求めた形 | 返った中身 |
| --- | --- |
| `TARGETS` | `TARGETS`・`UTF8_STRING`・`text/plain;charset=utf-8` |
| `UTF8_STRING` | `1\tAlice\t"line1\nline2"\r\n2\tBob\t007\r\n3\tCarol\t"tab\there"\r\n` |

見出しは入らず、`007` は変わらず、改行とタブを含むセルは `"` で囲まれた。状態の行は `… · 3 行 × 3 列をコピーしました`。

- WSLg の上で施主が Pollock の `results/source.csv` の範囲を選んで Ctrl+C を押し、Windows のアプリに貼って良しとした（2026-10-26。WSLg がクリップボードを Windows と共有する）

## 22. 詳細ペイン（Issue #31）— 仮想の X サーバでの確認（QLT-013）

`target/` に作った 40 行のファイル（長い英文・改行を含む和文のセル）を開き、xdotool で End と ↓ を送って撮った（2026-10-26。撮った後にファイルは消した）:

| 時点 | ペイン |
| --- | --- |
| End（セルの無い 4 列目） | 見出しの行に `4 列目`・`2 行目`・右端に `Ctrl+Shift+U で閉じる`。本文は `〈この行にはこの列のセルが無い〉` |
| ↓（311 字の英文） | 見出しの行に `3 行目　311 文字　UTF-8 で 311 バイト`。本文は 4 行に折り返し、左に青い帯 |

- WSLg の上で施主が Pollock の `results/source.csv` でペインと Ctrl+Shift+U の開閉を見て良しとした（2026-10-26）
- 確認用のファイルは、`Smith, John` を引用符で囲み忘れていたので、その行は列が 1 つ多い。`--dump` で読み方は正しいと確かめた（改行を含む引用のセルは 1 つ）

## 23. 選択の見えやすさと面の区切り（Issue #32）— 仮想の X サーバでの確認（QLT-013）

Pollock の `results/source.csv` を開き、xdotool で ↓ ↓ → Shift+↓ ↓ Shift+→ → を送って撮った（2026-10-26）。行番号 4〜6 × TIME・Qty・PRODUCTID の矩形の外周に青い枠、行番号 4〜6 と見出しの TIME・Qty・PRODUCTID が選択の色、カーソルの枠は行番号 6 の PRODUCTID。タイトルバーの下と状態の行の上に区切りの線。詳細ペインの見出しは `PRODUCTID　6 行目　4 列目　7 文字　UTF-8 で 7 バイト`（直す前は名前の最後の字が欠けて `PRODUCTII` に見えた）。

- WSLg の上で施主が見て良しとした（2026-10-26）。あわせて「選んだセルと同じ行と列に薄い背景色」の要望があり、Issue #33 にした

## 24. 選んだ範囲の行と列の帯（Issue #33）— 仮想の X サーバでの確認（QLT-013）

Pollock の `results/source.csv` を開き、xdotool で ↓ ↓ ↓ → → → を送って撮った（2026-10-26）。カーソルは行番号 5 の PRODUCTID。行番号 5 の行の他のセルと、PRODUCTID の列の他のセルが薄い帯（#F0F6FF）。行番号 5 と見出しの PRODUCTID は選択の色（Issue #32）。

- WSLg の上で施主が見て良しとした（2026-10-26）

## 25. 組み込みの配色（Issue #34）— 仮想の X サーバでの確認（QLT-013）

Pollock の `results/source.csv` を `--theme` で 4 つの配色（ライトの `neutral-light`・`gruvbox-light`、ダークの `ubuntu-aubergine`・`solarized-dark`）で開き、xdotool で ↓ ↓ → Shift+↓ Shift+→ を送って撮った（2026-10-26）。どの配色でも、表・見出しの行・行番号・選択の矩形と外周の枠・カーソルの枠・帯・詳細ペイン（帯は強調の色）・状態の行・タイトルバーが配色の役割から描かれ、字が読めた。ライトで強調が黄色の `gruvbox-light` は選択が淡い（百分率は仮）。

- WSLg の上で施主が Ctrl+Shift+T で配色を巡り、「どれもすごくいい」とした（2026-10-26）。既定は `neutral-light` のまま

## 26. Shift_JIS の 2 バイト目と重なる区切り（Issue #35）— 仮想の X サーバでの確認（QLT-013）

Issue #18 で断っていた `crates/app/fixtures/shift-jis-pipe.csv`（Shift_JIS・`|` 区切り・「表」は `95 5C`・「ソ」は `83 5C`）を開いて撮った（2026-10-26）。表は 4 行 × 3 列で、`表計算ソフト`・`能力表`・`ソース` がそれぞれ 1 つのセル。状態の行は `4 行 · Shift_JIS（CP932） · 縦棒区切り · LF · 3 列`。`--dump` でも同じ 4 行を UTF-8 の RFC 4180 で書き出した。

## 27. UTF-16 のファイル（Issue #36）— 仮想の X サーバでの確認（QLT-013）

`crates/app/fixtures/utf16le-tab.txt`（BOM 付きの UTF-16LE・タブ区切り・CRLF・値の中のタブ・空のセル・サロゲートの組の `𠮷`・`0012`・`1.20E+03`）を開いて撮った（2026-10-26）。表は 4 行 × 4 列で、値はそのまま（`0012`・`Smith, John`・`1.20E+03`）、値の中のタブは `→` で見え、空のセルは詳細ペインで `〈空のセル〉`。状態の行は `4 行 · UTF-16LE（BOM） · タブ区切り · CRLF · 4 列`。`𠮷` はこの字の大きさでは `吉` と見分けにくいが、`--dump` の書き出しで `𠮷` が正しいことを確かめた。

- WSLg の上で施主が見て良しとした（2026-10-26）

## 28. 文字の大きさの変更（Issue #37）— 仮想の X サーバでの確認（QLT-013）

Pollock の `results/source.csv` を開き、xdotool で ↓ と Ctrl+`=` を 4 回送って撮り（175%）、Ctrl+`0` と Ctrl+`-` を 2 回送って撮った（80%）（2026-10-26）。どちらも、表（見出しの行と行番号を含む）と詳細ペインの字だけが大きさを変え、タイトルバーと状態の行は同じ大きさ。状態の行の後ろに `文字の大きさ 175%` / `文字の大きさ 80%`。175% では詳細ペインも高くなり、表は 4 行ほど見える。

- Issue #38（施主が選んだ案）の後、175% で URL の列のセルを撮り直した。詳細ペインの高さは 100% のときと同じくらいで本文は 2 行ぶん、表は 7 行ほど見える
- Issue #39（施主の指摘）の後、300% で ProductDescription の列のセルを撮った。本文は 2 行で、値の 1 行目と `〈あと 5 行〉`。見出しの行の補助の情報は入りきらず見えない
- WSLg の上で施主が拡大と詳細ペインを触って良しとした（2026-10-26・Issue #37〜#39）

## 29. 設定を残すことと OS の明暗（Issue #40）— 仮想の X サーバでの開き直し（QLT-013）

設定の置き場を `XDG_CONFIG_HOME=target/xdg-check`（リポジトリの中・確かめた後に消した）にして、Pollock の `results/source.csv` を開いた（2026-10-26）:

| 時点 | 結果 |
| --- | --- |
| 1 回目（設定が無い・`GTK_THEME=Adwaita:dark`） | 既定のダークの配色 `ubuntu-aubergine`・100%・詳細ペインは開く |
| Ctrl+Shift+T・Ctrl+`=` × 2・Ctrl+Shift+U の後に終える | `settings.v1.txt` は `version = 1` / `theme = neutral-light` / `zoom = 125` / `detail = closed` |
| 2 回目（`GTK_THEME` なし） | `neutral-light`・125%・詳細ペインは閉じたまま |

- WSLg の上で施主が配色と文字の大きさを変えて閉じ、開き直して残っていることを確かめた（2026-10-26。設定の置き場は `XDG_CONFIG_HOME` でリポジトリの `target/xdg-wslg` に向けた。作業の中身をプロジェクトの外に出さないため）

## 30. コマンドパレット（Issue #41）— 仮想の X サーバでの確認（QLT-013）

設定の置き場を `target/xdg-check`（確かめた後に消した）にして Pollock の `results/source.csv` を開き、xdotool で送って撮った（2026-10-26）:

| 送ったキー | 画面 |
| --- | --- |
| Ctrl+Shift+P | 窓の上の中央にパレット。空の検索欄に案内文、右端に `8 / 8`。8 つの操作が固定の順で、キーの案内は右寄せ。先頭の候補に選択の色と左の帯。状態の行の右端に `Ctrl+Shift+P でコマンド` |
| `haishoku` | `配色を選ぶ…` の 1 件（`1 / 8`） |
| Enter・↓ × 3 | 配色の段。表だけが `monokai` に替わり、パレットは開いたときの配色のまま。各行に 3 色の見本と `ライト` / `ダーク`、上に `試し中: monokai　Enter で決める・Esc で戻す`、右端に `125 / 125` |
| Esc | パレットが閉じ、元の配色に戻る。設定のファイルは書かれない（試しは残さない） |

- WSLg の上で施主が触って良しとした（2026-10-26。パレットで配色 `monalisa` を選び、設定に残った）

## 31. 選んだ範囲の集計（Issue #42）— 仮想の X サーバでの確認（QLT-013）

設定の置き場を `target/xdg-shot` にして、数の見本（`out/sum.csv`・見出し 1 行 ＋ 5 行・`1,200`・全角の `１２`・空のセル・負の数を含む）を開き、
xdotool で → と Shift+↓ × 4・Shift+→ × 2 を送って撮った（2026-10-26）:

- 状態の行の後ろに `件数 14 · 合計 3,802.00 · 平均 316.833333 · 数でない 2`。手で数えた値と同じ（空でないセル 15 − 1 = 14・数 12・`1,200` と `１２` は数でない・見出しは数えない）
- 状態の行の右端の `Ctrl+Shift+P でコマンド` は残り、左の文言と重ならない
- WSLg の上で施主が触って良しとした（2026-10-26）

## 32. 裏の集計（Issue #43）— 仮想の X サーバでの確認（QLT-013）

100 万行の見本（`out/million.csv`・28 MB・見出し 1 行 ＋ 1,000,000 行 × 4 列・乱数の単価と数量と文字の備考）をリリースの組み立てで開き、
索引を作り終えてから → と Ctrl+Shift+End で 2〜4 列目の 1,000,001 行を選んで撮った（2026-10-26・設定の置き場は `target/xdg-shot`）:

| 時点 | 状態の行の右端 |
| --- | --- |
| 選んで 0.3 秒 | `合計を計算しています 19%（Esc で止める）`（同じ操作の別の回で 22%） |
| 選んで 3 秒 | `件数 3,000,000 · 合計 525,112,271.06 · 平均 262.556136 · 数でない 1,000,000`（備考の列は数でない） |
| もう一度選んで 0.2 秒で Esc | `合計を止めました` |

- 0.5 秒ごとに状態の行を撮ると、選んでから 1.3 秒の画像で結果になり、その後は変わらなかった（3 列 × 100 万行の集計が約 1.3 秒以内・この開発機の WSL2）
- 集計は、パレットの案内の代わりに状態の行の右端に出す（Excel と同じ場所）。左の文言はその全角 1 字手前で切る。1 つのセルに戻すと案内が戻る
- WSLg の上で施主が 100 万行の見本を触って良しとした（2026-10-26・「めっちゃ早い」）

## 33. スクロールバー（Issue #44）— 仮想の X サーバでの確認（QLT-013）

30 列 × 5,000 行の見本（`out/wide.csv`・数字だけ）を開き、xdotool で送って撮った（2026-10-27・設定の置き場は `target/xdg-shot`）:

| 操作 | 画面 |
| --- | --- |
| ↓↓→ と Shift+↓ × 12・Shift+→ × 2 | 縦（右端）と横（下端）の両方が出る。縦の溝は見出しの行の下から、横の溝は行番号の列の右から。溝は面の色、表との間に罫線。つまみは細く、選んだ範囲の強調の色の棒が溝の表側の端に、カーソルの線が溝の幅いっぱいに出る（横では、つまみに重なる所は背景の色） |
| 縦のつまみを押して下へドラッグ | 表が 2,839 行目へ動き、つまみが太く補助の字の色になり、左に「2,839 行目」の札 |
| `--theme monokai` | 暗い配色でも溝とつまみと印が見分けられる |

- WSLg の上で施主が触って良しとした（2026-10-27）

## 34. ドラッグでの範囲選択（Issue #45）— 仮想の X サーバでの確認（QLT-013）

30 列 × 5,000 行の見本（`out/wide.csv`）で、xdotool で 2 列目の本体のセルを押し、押したまま表の下の端より下（状態の行の上）へ動かして 1 秒待って撮った（2026-10-27）:

- 指を止めていても表が下へ流れ続け（62〜78 行目が見えている）、選択は見えている最後の行まで広がり、状態の行の右端に `76 行 × 4 列`
- 離すと右端が `件数 328 · 合計 1,646,473 · 平均 5,019.734756 · 数でない 0` に変わる（82 行 × 4 列 = 328。離すまでに少し流れた）
- WSLg の上で施主が触って良しとした（2026-10-27）

## 35. 離れた範囲を足す選択（Issue #46）— 仮想の X サーバでの確認（QLT-013）

30 列 × 5,000 行の見本（`out/wide.csv`）で、xdotool で送って撮った（2026-10-27）:

| 操作 | 画面 |
| --- | --- |
| ドラッグで 1〜3 列 × 2〜5 行、Ctrl+ドラッグで 6〜8 列 × 7〜10 行、Ctrl+クリックで 2 列 9 行 | 3 つの範囲がそれぞれ選択の色と外周の枠。カーソルは最後に足したセル。行番号と見出しの印と帯は全部の範囲に。横の溝に範囲ごとの棒。右端に `件数 25 · 合計 112,176 · 平均 4,487.04 · 数でない 0`（12 ＋ 12 ＋ 1） |
| 上の 2 つの範囲で Ctrl+C | 右端に `離れた範囲は、行か列が揃っていないとコピーできません`（コピーしない） |
| 1 列目の 2 行と 8 行を Ctrl+クリックで選んで Ctrl+C | 右端に `2 行 × 1 列をコピーしました`（同じ列なので上から詰める） |

- 一度だけの知らせは、状態の行の右端に集計の代わりに出すようにした（左に続けると集計に隠れて読めなかった）
- WSLg の上で施主が触って良しとした（2026-10-27）

## 36. 行全体・列全体の選択（Issue #47）— 仮想の X サーバでの確認（QLT-013）

100 万行の見本（`out/million.csv`）をリリースの組み立てで開き、索引を作り終えてから xdotool で送って撮った（2026-10-27）:

| 操作 | 画面 |
| --- | --- |
| 「単価」の見出しを押す | 列全体が選ばれ、カーソルは最初の行。縦の溝の範囲の棒が溝の全体に。右端は裏の計算を経て `件数 1,000,000 · 合計 499,610,315.06 · 平均 499.610315 · 数でない 0` |
| 続けて「数量」の見出しを Ctrl+クリック | 2 列の選択。`件数 2,000,000 · 合計 525,112,271.06 · 平均 262.556136 · 数でない 0`（Issue #43 で数えた単価と数量の合計と同じ） |

- 最初の撮影で、索引を作り終えた後の知らせのたびにファイルの終わりまでの選択を数え直し、裏の計算が 0% から進まなかった。行の数が決まったときの 1 度だけ数え直すように直し、テストを足した（`later_reports_after_the_index_finished_do_not_recount`）

## 37. 列の番号の行（Issue #48）— 仮想の X サーバでの確認（QLT-013）

見出しの行が無い見本（`out/noheader.csv`・4 行 × 4 列）を開き、xdotool で 4 列目の番号を押して撮った（2026-10-27）: 表の上に `1 2 3 4` の列の番号の行（見出しの行と同じ面の色・行番号の列は空き）。4 列目の全部の行が選ばれ、右端に `件数 4 · 合計 3,350 · 平均 837.5 · 数でない 0`（番号の行は数えない）。

## 38. 列の入れ替え（Issue #49）— 仮想の X サーバでの確認（QLT-013）

30 列 × 5,000 行の見本（`out/wide.csv`）で、xdotool で「列5」の見出しを押して選び、もう一度押して「列1」と「列2」の間へドラッグして撮った（2026-10-27）:

- ドラッグ中は「列1」の右端に強調の色の縦の線（落とす位置）。右端に `離した所へ列を移します（Esc でやめる）`
- 離すと「列5」が 2 番目に移り、列全体で選び直される。状態の行は `5 列目`（元の列の番号）。右端は移した列の集計 `件数 5,000 · 合計 25,057,868 · …`
- WSLg の上で施主が触って良しとした（2026-10-27）

## 39. 読み方を選んで開き直す（Issue #52）— 仮想の X サーバでの確認（QLT-013）

Shift_JIS・縦棒区切りの見本（`crates/app/fixtures/shift-jis-pipe.csv`）を開き、xdotool で送って撮った（2026-10-27）:

| 操作 | 画面 |
| --- | --- |
| Ctrl+Shift+P → `encoding` → Enter → ↑↑ | 2 段目に文字コードの 9 つ。`Shift_JIS（CP932）` に `今の読み方`。`UTF-8` を選ぶと表がその場で UTF-8 で読み直されて文字化けし、上に `試し中: UTF-8　Enter で開き直す・Esc で戻す`、状態の行は `UTF-8 · 縦棒区切り` |
| Esc | 元の画面に戻る（`4 行 · Shift_JIS（CP932） · 縦棒区切り`。索引も元のまま） |
| Ctrl+Shift+P → `delimiter` → Enter → ↓ → Enter | 空白区切りで開き直され、表は 1 列（`コード\|品名\|単価`）。索引が作り直されて `4 行 · Shift_JIS（CP932） · 空白区切り` |

## 40. 見えない文字の印（Issue #53）— 仮想の X サーバでの確認（QLT-013）

全角空白・NBSP・ゼロ幅の空白を含む見本（`out/invisible.csv`）を開いて撮った（2026-10-27）: `A001□`（末尾の全角空白）・`山田□太郎`・`10␣000`（NBSP）・`ゼロ幅¦あり`（U+200B）。普通の空白（`鈴木 花子`）はそのまま。詳細ペインも同じ印で、文字数とバイト数は元の値（`山田　太郎` は 5 文字・UTF-8 で 15 バイト）。最初は NBSP を `⍽` にしたが、本文の字体（Noto Sans CJK JP）に無い字で豆腐になったので `␣` に替えた。

- WSLg の上で施主が見本を開いて印を見て良しとした（2026-10-27）

## 41. 行へ飛ぶ（Issue #54）— 仮想の X サーバでの確認（QLT-013）

100 万行の見本（`out/million.csv`）をリリースの組み立てで開き、xdotool で Ctrl+G と `500000,3` を送って撮った（2026-10-27）: 入力欄の下に `Enter で 500,000 行目 · 3 列目 へ`。Enter で 500,000 行目の「数量」のセルが選ばれ、表がそこまで動き、状態の行と詳細ペインは `500,000 行目 · 3 列目`。

## 42. Windows の実機での最初の起動（Issue #55）— 施主の機械（QLT-013）

`make build-windows` で組み立てた `nene-ledger.exe`（2,754,977 バイト・x86_64-pc-windows-gnu）と見本 4 つを Windows 側の置き場に置いた（2026-10-27 11:49）。

- 20 秒後もファイルは残り、検疫されなかった（Issue #7 の試験用の exe とは違う結果）
- コマンドプロンプトから `nene-ledger.exe wide.csv` で起動して窓が出た。エクスプローラーで CSV を exe の上に落としても起動した（施主）。exe だけを開くとファイルが無いので使い方を出して終わる（窓は出ない）
- 施主が触った結果（2026-10-27）: 自前のタイトルバーと 3 つのボタンが出る・最小化と閉じるが効く・タイトルバーのドラッグで移動・縁でリサイズ・ダブルクリックで最大化・画面の端へのスナップ・字体と大きさと切れ方に違和感なし・矢印と Ctrl+C と Ctrl+Shift+P と Ctrl+G・クリックとドラッグとホイール・Ctrl+C で Excel に貼れる。「すごく性能いい」
- 🔴 見つけた不具合: 最大化の**ボタン**で最大化すると重くなり反応しなくなった（ダブルクリックの最大化は問題なし）。ボタンの動作を状態を借りたまま `ShowWindow` で行い、同期で届く `WM_SIZE` を状態が受け取れず窓の大きさを古いまま覚えていた。`WM_SYSCOMMAND` を積む形に直し、作り直した exe（2,754,714 バイト・12:01）で施主が「直ってる。サクサク動く」と確かめた

## 43. Windows の起動の入口（Issue #56）— 施主の機械（QLT-013）

作り直した exe（12:13）を施主が触った（2026-10-27）: exe をダブルクリックすると「開くファイルを選ぶ」画面が出て、CSV を選ぶと開き、キャンセルで何も出ずに終わる。ドラッグ＆ドロップでもコンソールが出ない。開けないときはメッセージの窓が出る。exe そのものを選ぶと、バイナリがセルに入った表として開いてしまう（文字のファイルかどうかの判別は無い・残るリスク）。

## 44. 文字のファイルでないものの確かめ（Issue #57）

- X11（Xvfb・2026-10-27）: 組み立てた X11 版の先頭 4 KiB（`out/binary-sample.bin`・ELF）を開くと、状態の行の右端に `文字のファイルではないようです（NUL のバイトがある）`
- Windows（施主の機械・14:12 の exe）: exe 自身を選ぶと尋ねる窓が出て、「いいえ」で終わり、「はい」で開いて知らせが出る。CSV は尋ねられずに開く（施主「想定通り」）

## 45. 画面の倍率（Issue #58）— 施主の機械（QLT-013）

作り直した exe（16:32）を施主が触った（2026-10-27）: 125% と 150% の画面で、字・タイトルバー・ボタン・スクロールバー・罫線・カーソルの枠がほかのアプリと同じくらいの大きさ。窓を別の倍率の画面へ動かすと大きさが切り替わり、見かけの大きさが保たれる。画面をまたいだ後も縁のリサイズ・ボタン・スクロールバーのドラッグ・セルのクリックの位置がずれない。150% の画面での最大化ではみ出しも隙間も無い（施主「全部大丈夫そう」）。倍率の計算と 150% の配置は core と application のテスト（`display_scale`・`the_border_and_the_edges_grow_with_the_display_scale`・`the_grid_line_grows_with_the_display_scale`・`the_bars_grow_with_the_display_scale`）

## 46. M1 の行の Windows の実機での確認（Issue #59）— 施主の機械（QLT-013）

試しのファイル 13 個（`out/m1-check/make_samples.py`・Linux の `--dump` で期待を確かめた）と 16:32 の exe で、施主が Issue #59 の手順 1〜15 を通した（2026-10-27）。全部が期待どおり: 題名・値をそのまま見せる・引用の中の改行と区切りと `""`・欠けたセルと増える列・Shift_JIS と UTF-16LE と `\` のエスケープと `;` と `^`・開き直しの試しと Esc・見えない文字の印（Yu Gothic UI と代わりの字体）・100 万行の最初の画面と進み具合と Ctrl+End / Ctrl+G / 選択・横の移動・約 300 MB の閉じない引用符（固まらない）・拡大・配色の切り替えと開き直し・アプリ モードがダークならダークの配色。

- 🔴 見つけたこと: 詳細ペインのバイト数がファイルの文字コードに依らず `UTF-8 で N バイト` だった（施主の画面の写し・UTF-16LE のファイル）。ファイルの文字コードで数えるようにし（core の `TextEncoding::encoded_length`・`encoding_rs` で復号できる全部の 1 字との総当たりのテスト）、作り直した exe（17:42）で施主が `UTF-16LE で 8 バイト`・`Shift_JIS で 12 バイト`・`𠮷田 花子` が `5 文字　UTF-16LE で 12 バイト` を確かめた

## 47. 行全体・列全体・列を移すキーと、キーの一覧（Issue #62・#63）

- X11（Xvfb・2026-11-02・xdotool で Ctrl+Shift+P → `keys` → Enter）: パレットの 2 段目に `25 / 25` の一覧。キーのある操作（`表の先頭に移る  Ctrl+Home`・`行全体を選ぶ  Shift+Space`・`列全体を選ぶ  Ctrl+Space` など）が先に並ぶ。`shift` と打つと `10 / 25` に絞られ、パレットの操作（`選んだ列を左へ移す  Ctrl+Shift+PageUp` など）とパレットに無いキー（`選択を広げる  Shift+Arrow keys`・`押したセルまで選択を広げる  Shift+Click`・`横に動かす  Shift+Wheel`・`前の配色を試す  Ctrl+Shift+Alt+T` など）が両方出る（`out/keys-all.png`・`out/keys-shift.png`）
- キーの割り当ては、一覧の各行を X11 と Windows のキーのテスト（`every_key_in_the_list_is_bound_as_written`）が確かめる

## 48. 列の幅のドラッグ（Issue #64）

- X11（Xvfb・2026-11-02・xdotool）: `screenshot.csv` の見出しの「社員番号」の右の境目（x = 122）を押して x = 242 へドラッグすると、列が 120 画素（半角 15 字）広がり、右の列が押し出されて横のスクロールバーが出る。選択は変わらない（`out/width-drag.png`）。パレットの「列の幅を元に戻す」で開いたときの幅に戻る（`out/width-reset.png`）
- カーソルの形（左右の矢印）は Xvfb の画像に写らないので、application のテスト（`only_a_header_border_shows_the_resize_cursor`）と ui-x11 の `CursorShape` のテストで確かめた。Windows（`WM_SETCURSOR`）は型検査だけ

## 49. 区切りの字を打って選ぶ（Issue #67）

- X11（Xvfb・2026-11-02・xdotool）: `#` で区切ったファイル（`id#name#note` と、値にカンマとセミコロンを含む 3 行）は、判別でカンマ区切りの 2 列に崩れて開く（`out/hash-before.png`）。Ctrl+Shift+P → `delimiter` → Enter → `#` と打つと、候補が「「#」区切り」の 1 つ（`1 / 7`）になり、その場で 3 列に読める（`試し中: 「#」区切り`・`out/hash-trial.png`）。Enter で開き直すと状態の行が `4 行 · UTF-8 · 「#」区切り · LF · 3 列`（`out/hash-after.png`）

## 50. 引用の扱いの表示とシングルクォートの引用（Issue #68）

- X11（Xvfb・2026-11-02・xdotool）: `'` で囲んだファイル（`'Smith, John'`・`'O''Brien'`）は `'` の引用と判別され、値は `Smith, John`・`O'Brien`。状態の行は `4 行 · UTF-8 · カンマ区切り · ' で引用 · LF · 3 列`。Ctrl+Shift+P → `quote` → Enter で 5 つの候補（`5 / 5`）が出て、「シングルクォート（'）で囲む」に `今の読み方` の印（`out/single-quote.png`）。最初の名前（`… '' で ' を書く`）では試し中の案内が右で切れたので短くした

## 51. 公開リポジトリの CI と ruleset（GitHub Issue #1・2026-11-04）

公開リポジトリ `hideyukiMORI/nene-ledger` へ移り（ADR 0021）、`xtask validate-git` を PR のメタデータを読む形に戻した。以下は実行した結果のみ。

| 規則 | 入れた違反・設定 | 実出力・事実 |
| --- | --- | --- |
| GIT-003 | `xtask commit-msg` に件名 `update docs` | `GIT-003: subject must be ...`・終了 1。正しい件名 `build: 公開リポジトリとして置き直す (#1)` は終了 0 |
| GIT-001 / GIT-004 / QLT-012 | `cargo test -p xtask`（92 件） | `pull_request_rejects_draft_missing_close_and_bad_title`（QLT-012 / GIT-001 / GIT-004 / GIT-003 の 4 つが出る）・`pull_request_field_must_start_a_line`（本文の途中の `目的:` は欄ではない）・`pull_request_body_with_a_machine_path_is_rejected`（GIT-005） |
| GIT-002 / GIT-004 | リポジトリ設定 | squash のみ（`merge_commit` / `rebase` 不可）・`delete_branch_on_merge: true`（`gh repo view --json`） |
| GIT-002 / GIT-004 | ruleset `main`（id 26530802・enforcement active・対象 `~DEFAULT_BRANCH`） | `deletion` / `non_fast_forward` / `pull_request`（`allowed_merge_methods: ["squash"]`・会話の解決必須）/ `required_status_checks`（`check`・strict）。`gh api repos/hideyukiMORI/nene-ledger/rulesets/26530802` |
| QLT-005 / GIT-001〜005 | PR #2 の CI `check`（`check.yml` → `make ci`・`ready_for_review` で起動） | 実行 41832653816: `rustc 1.98.0`・`Git conventions passed.`・`git diff --check origin/main...HEAD`・success。手元の `make ci` と同じ出力 |
| GIT-003（CI） | PR #2 のタイトルを `公開リポジトリの CI と ruleset の実測を記録する`（type と `(#N)` 無し）に編集 | 実行 41832778822（`edited`）: `GIT-003: subject must be ... (PR title)`・`make: *** [Makefile:61: validate-git] Error 1`・failure。タイトルを戻して success |

## 52. 判別の候補から選んで開き直す（GitHub Issue #3）— 仮想の X サーバでの確認（QLT-013）

見本 `crates/app/fixtures/screenshot.csv`（7 行・引用の中の改行・欠けたセル）を開き、xdotool で送って撮った（2026-11-04）。この見本は core の判別が確信を持たない（1 位 カンマ・`""` 350、2 位 カンマ・`\"` 299。差が 100 に満たない。app の `the_screenshot_sample_is_opened_with_a_doubt_that_names_the_top_two_scores`）。

| 操作 | 画面 |
| --- | --- |
| 開く | 状態の行に `判別に確信なし（1 位 35%・2 位 29%）· 候補は Ctrl+Shift+P → candidates`（一度だけの知らせ）。表は 1 位のカンマ区切りで、値はそのまま |
| Ctrl+Shift+P → `candidates` → Enter | 2 段目に候補 7 つが点数の順（`7 / 7`）。`カンマ区切り・"" で引用符（RFC 4180）` に `今の読み方 · 35% · 6 列`、次が `カンマ区切り・" で引用・中は \ で逃がす　29% · 6 列`、`カンマ区切り・' で引用　10% · 6 列`、タブとセミコロンの 4 つは `0% · 1 列`。上に `試し中: カンマ区切り・"" で引用符（RFC 4180）　Enter で開き直す・Esc で戻す` |
| ↓ | 表がその場で `\` で逃がす読み方で読み直され（`"二重""` のセルが変わる）、状態の行は `索引を作る準備をしています · UTF-8（BOM） · カンマ区切り · " で引用・中は \ で逃がす · LF · 6 列` |
| Esc | 開いた直後の画面に戻る（撮った PNG が `cmp` でバイト一致） |
| Ctrl+Shift+P → `candidates` → Enter → ↓↓ → Enter | シングルクォートの候補で開き直され、索引が作り直されて `8 行 · UTF-8（BOM） · カンマ区切り · ' で引用 · LF · 8 列`（`"` はセルの値として残る） |

名前が長いときは右の字（印と点数）の手前で切る（application の `a_long_name_is_clipped_before_the_aside`）。試しの案内の行は窓の幅 1024 では末尾が切れる（Issue #52 の読み方の段と同じ・幅 1280 以上なら収まる・残るリスク）。Windows の実機での確認はまだ。

## 53. 左の列の固定（GitHub Issue #4）— 仮想の X サーバでの確認（QLT-013）

20 列 × 30 行の見本（見出し `c01`〜`c20`・値 `r行-c列`・3〜6 字の列）を幅 1024 の窓で開き、xdotool で送って撮った（2026-11-05）。

| 操作 | 画面 |
| --- | --- |
| → → → Ctrl+Shift+P → `freeze` → Enter | 状態の行に `左の 3 列を固定しました（横に動いても残ります）`。`c03` の右に境の線（窓の縁の色・2 画素）。横のスクロールバーの溝は固定した列の分だけ短くならず、つまみが動く列の幅で測られる |
| → × 14（カーソルが `c17` まで） | `c01`〜`c03` は左に残り、その右は `c08` から `c18`（`c04`〜`c07` が隠れる）。状態の行は `2 行目 · 17 列目` |
| Home | カーソルが `c01` へ戻るが横の表示は動かない（`c08` からのまま）。固定した列は常に見えているので動かす理由が無い |
| Ctrl+Shift+P → `unfreeze` → Enter | `列の固定を外しました`。表示の最初の列は `c08` のまま（`c01` のカーソルは隠れ、横のスクロールバーの印が左端に残る） |
| Home | `c01` からの表示に戻る（撮った PNG が開いた直後と `cmp` でバイト一致） |

固定した列の幅が本体の幅（行番号の列を除く）の 2/3 を超えるときは、収まる数まで減らす（application の `freezing_more_than_fits_keeps_room_for_the_scrolling_columns`）。固定した列と動く列のクリックの位置は `frozen_and_scrolled_cells_are_hit_where_they_are_drawn`、横の移動の下限は `scrolling_left_stops_at_the_frozen_columns_until_they_are_unfrozen`、配置は core の `frozen_columns_stay_left_of_the_scrolled_ones` ほか 2 つ。列の入れ替えと幅のドラッグは固定した列でも同じ経路（`visible_columns` を見る）だが、撮っていない。Windows の実機での確認はまだ。

## 54. 判別の標本の境目（GitHub Issue #10）— 仮想の X サーバでの確認（QLT-013）

日本郵便の郵便番号の UTF-8 版（`utf_ken_all.csv`・18.4 MB・124,527 行・BOM 無し）を、直す前（`main` `458bf5f`）と直した後（`main` `a8b247b`）の release の窓で開き、幅 1024 で撮った（2026-11-05）。標本の 65,536 バイト目が字の途中にあるファイル。

| 版 | 画面 |
| --- | --- |
| `458bf5f` | 状態の行が `124,527 行 · Windows-1252 · カンマ区切り · CRLF · 15 列`。4 列目以降が `ãƒ›ãƒƒã‚«ã‚¤ãƒ‰ã‚¦` のように化け、詳細ペインは `Windows-1252 で 5 バイト` |
| `a8b247b` | 状態の行が `124,527 行 · UTF-8 · カンマ区切り · CRLF · 15 列`。`ホッカイドウ`・`サッポロシチュウオウク`・`北海道`・`札幌市中央区` と読め、詳細ペインは `UTF-8 で 5 バイト` |

試験は core の `a_sample_read_up_to_the_read_limit_keeps_a_character_split_at_the_sample_limit` と app の `a_character_across_the_sample_boundary_keeps_the_encoding`。実データの突き合わせは benchmarks 第 2 節、経緯は [research/2026-11-05-quality-sweep](../research/2026-11-05-quality-sweep.md)。Windows の実機での確認はまだ（読む経路は OS に依らない）。

## 55. 三点リーダーの省略（GitHub Issue #7）— 仮想の X サーバでの確認（QLT-013）

`id,memo,short` の 3 行（`memo` は日本語の住所の長い値・英文の長い値・`ちょうど`）を、release の窓（1024×640）で開いて撮った（2026-11-05）。

| 操作 | 画面 |
| --- | --- |
| 開く | `memo` の列は 40 字の幅。`東京都千代田区丸の内一丁目一番一号 丸…`・`This is a fairly long English sentence…`。`ちょうど` と `short` の列の `ok` には `…` が付かない。英文は字体の実際の幅が近似（半角 1）より狭いので、`…` の右に余白が残る（Issue のとおり、幅は近似で数え、矩形での切り取りを残す） |
| `memo` の右の境を x = 420 から 980 へドラッグ | `…` が消え、2 つの長い値が全文で描かれる |

省略は描く文字列だけで、値は変えない（コピーと詳細ペインは全文）。試験は application の `values_that_do_not_fit_end_with_an_ellipsis`・`a_huge_value_is_cut_to_the_cell`（1 MiB の値で頭の 39 字 ＋ `…`）・`values_wider_than_the_column_end_with_an_ellipsis`。固定した列・入れ替えた列も同じ描画の経路（`FramePainter::text`）だが、撮っていない。Windows の実機での確認はまだ。

## 56. 切り詰めた行を示す（GitHub Issue #36・ADR 0018 の決定 2）— 仮想の X サーバと `--dump`（QLT-013）

`id,amount,memo` の 10 行で、3 行目の `memo` だけが 2 MiB の引用のセルのファイルを、release で確かめた（2026-11-05）。

| 操作 | 結果 |
| --- | --- |
| 窓で開き ↓ ↓ → → | カーソルが 4 行目の `memo`。表のセルは `zzzz…`（三点リーダー・Issue #7）。詳細ペインの見出しの行が `memo　4 行目　3 列目　1 MiB で切り詰め（行の全体 2.0 MiB）`（文字数とバイト数の代わり）。本文は読んだ先頭の部分を折り返し、`〈あと 8,453 行〉`。11 行が全部見える（Issue #32） |
| `nene-ledger --dump` | 標準エラーに `nene-ledger: 1 MiB を超えるレコード 1 件は先頭の 1 MiB だけを書いた（最初は 4 行目）`。終了 0（出力は従来どおり）。切り詰めの無い 100 MB の CSV では何も出ない |

コピーは、切り詰めた値の行を含めば `… をコピーしました（うち N 行は 1 MiB で切り詰めた値）`（application の `a_cut_row_says_so_in_the_detail_pane_and_in_the_copy_notice`）。窓では撮っていない。索引を作り終える前は全体の大きさが分からず `1 MiB で切り詰め（行は 1 MiB を超える）`（app の `records_cut_at_the_limit_are_listed_with_their_whole_size`）。Windows の実機での確認はまだ。

## 57. 表の上でセルを直す（GitHub Issue #69・ADR 0026）— 仮想の X サーバでの確認（QLT-013）

`id,name,city` の 3 行（CRLF・2 行目の `name` は引用の `"Sato, Hanako"`）を、debug の窓（1024×640）で開き、xdotool でキーを送って撮った（2026-11-05）。

| 操作 | 画面 |
| --- | --- |
| → で `name` へ、`Kato` と打つ | セルが地の色の欄になり `Kato` とカーソルの棒。詳細ペインも `Kato`。状態の行の右端が `入力中 · Enter 確定 · Esc やめる · F2 編集へ` |
| Enter → ↑ → F2 → ← ← → `Z` | Enter で確定して下へ動き、題名が `● t.csv — NeNe Ledger`。F2 で `Kato` の末尾から、← で欄の中を動き `KaZto`（状態の行は `編集中 …`） |
| Enter → ↓ → Delete | 3 行目の `name` が空になり、詳細ペインが `〈空のセル〉` |
| （開き直して）→ F2 → Alt+Enter → `x` → Tab → Ctrl+C | 欄の中で改行し、詳細ペインに 2 行（`Sato, Hanako` と `x`）。Tab で確定して右へ動く。Ctrl+C の知らせが出る |
| （開き直して）→ F2 → ` Tanaka` | 欄より長い値は頭を飛ばして描き、カーソルの棒が右端に見えたまま（`nako Tanaka▏`） |

確定の 3 通り（見えていた値のまま・原本の値に戻した・それ以外）と、直せない所（列の足りない行・1 MiB で切り詰めた行）は application のテスト（`typing_the_source_value_back_removes_the_edit`・`a_missing_cell_and_a_cut_record_are_not_edited` など）。キーの割り当ては両方の OS のキーのテスト（`every_key_in_the_list_is_bound_as_written`）。

🔴 **正直に:** 画面からの保存（Ctrl+S）はまだ無いので、直した値はファイルに残らない（次の Issue）。Windows の実機・IME の確定前の見え・ダブルクリックでの編集はまだ。選んだ集計（VIEW-14）は原本の値で数える（書き換えを重ねていない）。

## 58. 画面から保存する（GitHub Issue #71・ADR 0026 の決定 6）— 仮想の X サーバでの確認（QLT-013）

`id,name,city` の 3 行（CRLF・2 行目の `name` は引用の `"Sato, Hanako"`）を、debug の窓で開き、xdotool でキーを送った（2026-11-05）。

| 操作 | 結果 |
| --- | --- |
| → F2 ` Jr.` Enter → `Kyoto` Ctrl+S | 状態の行が `保存しました`、題名の `●` が消え、カーソルは 3 行目の `city` のまま。ファイルは `0012,"Sato, Hanako Jr.",Tokyo\r\n` と `0013,Suzuki,Kyoto\r\n` の 2 行だけが変わり、見出しの行・引用・CRLF は元のバイト列のまま（`od -c`・`diff`）。一時ファイルは残っていない |
| ↓ `9` Enter → 外から `printf ... > t.csv` で中身を替える → Ctrl+S | 状態の行が `開いた後にほかのアプリがファイルを変えたので、上書きしませんでした（直した値は残っています）`。題名の `●` は残る。外で書いた中身はそのまま |

保存の後に `--resave` の経路（`ScanBytes` が原本の口を持つように変えた）で、Pollock と実データの置き場の写し 2,740 ファイルを保存し直し、失敗 0・元と違うファイル 0・残った一時ファイル 0（benchmarks 第 4 節と同じ手順・2026-11-05）。

保存の流れ（書き換えが無いときの知らせ・保存中は直せない・成功で差分を空にして同じ位置から読み直す・失敗で差分を残す）は application のテスト、裏の仕事は store の `a_save_job_writes_in_the_background_and_wakes_the_window`・`a_failed_save_job_reports_the_failure_once`。

🔴 **正直に:** Windows の実機（`ReplaceFileW`・Excel が開いているときの止まり方）は未確認。保存の進み具合と中止（SAVE-02）・別名で保存・閉じるときに尋ねる画面はまだ。保存の後は Undo の履歴が無い（UNDO-01 がまだ）。

## 59. 未保存のまま閉じるときに尋ねる（GitHub Issue #73・ADR 0026 の決定 7）— 仮想の X サーバでの確認（QLT-013）

`id,name` の 3 行（CRLF）を debug の窓で開き、xdotool でキーを送り、タイトルバーの × を押した（押して 150 ms 後に離す・2026-11-05）。

| 操作 | 結果 |
| --- | --- |
| ↓ → `zz` Enter → × | 窓の真ん中に `直したセルが保存されていません。閉じる前に保存しますか？` と 3 つのボタン（`保存して閉じる (S)`・`保存しないで閉じる (D)`・`やめる (Esc)`）。窓は閉じない |
| Esc | 尋ねる画面が消え、窓は残る |
| × → S | 保存してから窓が閉じる。ファイルの 3 行目が `2,zz\r\n`（ほかの行は元のまま） |
| （開き直して）↓ `9` Enter → × → D | 保存せずに閉じる。ファイルは変わらない |
| （開き直して）× | 書き換えが無いので、尋ねずにすぐ閉じる |

選び方（Enter・S・D・Esc・ボタンを押す）・保存に失敗したら閉じないこと・保存の途中で閉じようとしたときは application のテスト（`closing_with_edits_asks_and_escape_keeps_the_window`・`a_failed_save_does_not_close`・`closing_while_saving_asks_and_save_closes_after_it` など）。

🔴 **正直に:** 窓の管理者からの閉じる頼み（`WM_DELETE_WINDOW`）は、Xvfb に窓の管理者が居ないので試していない（× のボタンと同じ `close` を通る）。Windows の `WM_CLOSE`（× と Alt+F4）は `make lint-windows` の型検査だけ。

## 60. セルの編集・保存・閉じる・元に戻す — 施主の Windows の実機での確認（GitHub Issue #77・QLT-013）

main `da1f474` の exe（`make build-windows`）を施主の Windows 11 の作業用のフォルダ（OneDrive の同期の外）に置き、`edit-test.csv`（UTF-8・CRLF・見出し ＋ 3 行・2 行目の氏名は `"佐藤, 花子"`）を開いた（2026-11-05）。操作は施主の手、ファイルの中身は WSL から元の写しと `diff` で比べた。

| # | 操作 | 施主の報告と、ファイルの確かめ |
| --- | --- | --- |
| 1 | 「鈴木一郎」のセルで、IME（Microsoft IME）で「やまだ」→「山田」と確定 → Enter | 全部 OK（確定した字がセルに入る・Enter で下へ動く・題名に `●`）。確定前の文字列はセルの外の IME の窓に出る（IMM32 の Issue はまだ・想定どおり） |
| 2 | Ctrl+S | 全部 OK（`保存しました`・`●` が消える・値が残る・固まらない）。ファイルは 3 行目だけが `0013,山田,…` に変わり、引用・CRLF は元のまま。一時ファイルは残っていない（`ReplaceFileW` の実機の確認） |
| 3 | Excel で同じファイルを開いたまま、「田中」→「田中太郎」で Ctrl+S → Excel を閉じてもう一度 Ctrl+S | 全部 OK（1 回目は `ほかのアプリがファイルを開いているので保存できませんでした（直した値は残っています）`・`●` と値が残る。2 回目は `保存しました`）。ファイルは 4 行目だけが変わった（FILE-04 の書く前の確かめ） |
| 4 | F2 → ← ← → `X` → Alt+Enter → `2丁目` Enter・Tab / Shift+Tab / Shift+Enter・Delete | 全部 OK（F2 で末尾にカーソル・途中に挿せる・詳細ペインで 2 行・Alt+Enter で警告の音が鳴らない・Tab で動く・Delete で `〈空のセル〉`） |
| 5 | Ctrl+Z × 3 → Ctrl+Y | 全部 OK（戻したセルへカーソルが動く・`元に戻しました（1 セル）`・保存した「田中太郎」も「田中」に戻せる・Ctrl+Y で戻ると `●` が消える） |
| 6 | 「1200」→「1300」→ × → Esc → Alt+F4 → 「保存して閉じる (S)」を押す | 全部 OK（× と Alt+F4 で尋ねる画面・Esc で残る・押すと保存してから閉じる）。ファイルの 2 行目が `…,1300`、プロセスは終わっていた |

最後のファイルは、元と比べて 3 つのセル（1300・山田・田中太郎）だけが違い、`"佐藤, 花子"` の引用と CRLF は元のまま。

🔴 **正直に:** 確定前の文字列をセルの中に出すこと（IMM32）は未実装。尋ねる画面の「保存しないで閉じる」は実機では試していない（Xvfb の第 59 節だけ）。IME は Microsoft IME だけ。

## 61. IME の確定前の文字列をセルの中に出す — 施主の Windows の実機での確認（GitHub Issue #81・ADR 0026 の決定 4・QLT-013）

PR #82 の枝 `60d59e1` の exe（`make build-windows`）を施主の Windows 11 の作業用のフォルダ（OneDrive の同期の外）に置き、見本の `screenshot.csv` を開いた（2026-11-05）。操作は施主の手で、IME は Microsoft IME。

| # | 観点 | 施主の報告 |
| --- | --- | --- |
| 1 | 表の上で「にほんご」と打ち始めると、セルの中に下線つきで出て、IME の組み立ての窓は出ない | OK |
| 2 | 変換の候補の窓がセルの左下に出る | OK |
| 3 | Enter で確定するとセルに入り、もう一度 Enter で確定して下へ動く | OK |
| 4 | F2 の後に打つと、元の値の後ろに入る | OK |
| 5 | 組み立て中の Esc で組み立てだけが消え、もう一度 Esc で編集をやめる | OK |
| 6 | パレット（Ctrl+Shift+P）で英字が今までどおり打てる | OK |

施主の言葉は「うん、どれもオッケー」〔伝聞：施主の報告〕。

🔴 **正直に:** IME は Microsoft IME だけ。パレットで IME をオンにしたときの見え（確定前の文字列はどこにも出ない）と、欄より長い確定前の文字列は試していない。施主の気づき：列の数が足りない行の「セルが無い」所には書けない（`この行には、この列のセルがありません`）。これは最初の縦切りの作り（`begin_edit` と保存の計画の `NoSuchCell`）で、IME の経路の問題ではない。

## 62. 列の数が足りない行の、セルの無い所に書く（GitHub Issue #83・ADR 0024 の施主の判断 4・5）— 仮想の X サーバでの確認（QLT-013）

PR の枝の debug の exe を Xvfb（1024×640）に出し、`xdotool` で打って Ctrl+S で保存し、ファイルを `od -c` で見た（2026-11-05）。

| # | 元のファイル | 操作 | 保存した後のファイル |
| --- | --- | --- | --- |
| 1 | `id,name,city\r\n1,"Sato"\r\n2,Suzuki,Osaka\r\n` | 2 行目の 3 列目（セルが無い）で `Tokyo` → Enter → Ctrl+S | `id,name,city\r\n1,"Sato",Tokyo\r\n2,Suzuki,Osaka\r\n`（`"Sato"` の引用と CRLF は元のまま・ほかの行はバイト列のまま・状態の行に `保存しました`） |
| 2 | `a,b,c\r\n1\r\n2,x,y\r\n` | 2 行目の 3 列目で `z` → Enter → Ctrl+S | `a,b,c\r\n1,,z\r\n2,x,y\r\n`（間の 2 列目は空のセル。開き直した画面では「セルが無い」の色ではなく空のセル） |

施主の Windows の実機（PR の枝 `8c36482` の exe・作業用のフォルダ）でも確かめた（2026-11-05）。`short.csv`（`id,name,city,memo\r\n1,"Sato"\r\n2\r\n3,Suzuki,Osaka,ok\r\n`）の 2 行目の city と memo、3 行目の memo に施主の手で書いて Ctrl+S。施主の報告は「いいと思う」〔伝聞〕。ファイルは WSL から `od -c` で元の写しと比べた：`1,"Sato",123,123\r\n`・`2,,,1\r\n`。1 行目と 4 行目・`"Sato"` の引用・CRLF は元のまま、一時ファイルは残っていない〔実測〕。

🔴 **正直に:** 区切りがタブの表と、引用の要る値（区切りを含む値）を足すことは core と application のテストだけ（`row_writing`・`save_planning`）。

## 63. 行を足す・消す・戻す（GitHub Issue #95・ADR 0027）— 仮想の X サーバでの確認（QLT-013）

PR の枝の debug の exe を Xvfb（1024×640）に出し、`xdotool` で操作して Ctrl+S で保存し、ファイルを `od -c` で見た（2026-11-05）。
元のファイルは `id,name\r\n1,"Sato"\r\n2,Suzuki\r\n3,Tanaka`（最後の行に終わり方が無い）。

| # | 操作 | 見え・ファイル |
| --- | --- | --- |
| 1 | ↓（`2,Suzuki`）→ パレットで `insert row below` → `9` Tab `Kato` Enter | `2,Suzuki` の下に行が入り、番号は 1〜5 の通し番号。足した行の番号は強調の色 |
| 2 | パレットで `append row` → `10` Enter | 表の末尾に行が入る |
| 3 | Ctrl+Home → パレットで `delete row` | `1,"Sato"` の行が消え、状態の行に `1 行を消しました（Ctrl+Z で戻す）` |
| 4 | Ctrl+S | `id,name\r\n2,Suzuki\r\n9,Kato\r\n3,Tanaka\r\n,10\r\n`。触っていない行はバイト列のまま。終わり方の無かった最後の行の後ろに CRLF を補ってから足した行を書いた（`,10` はカーソルが 2 列目にあったため） |

🔴 **正直に:** キー（ADR 0027 の Ctrl++ と Ctrl+-）はまだ割り当てていない（文字の大きさの Ctrl++ / Ctrl+- とぶつかる・施主に諮る）。Windows の実機ではまだ。Undo / Redo は application のテストだけ。

## 64. 列を足す・消す・戻す（GitHub Issue #99・ADR 0027）— 仮想の X サーバでの確認（QLT-013）

PR の枝の debug の exe を Xvfb（1024×640）に出し、`xdotool` で操作して Ctrl+S で保存し、ファイルを `od -c` で見た（2026-11-05）。
元のファイルは `\xEF\xBB\xBFid,name\r\n1,"Sato, T"\r\n2\r\n`（BOM 付き・2 行目は 1 列だけ）。

| # | 操作 | 見え・ファイル |
| --- | --- | --- |
| 1 | パレットで `insert column right`（カーソルは `id`）→ `40` Enter `35` Enter | `id` の右に空の見出しの列が入り、`40`・`35` が入る |
| 2 | Ctrl+S | `\xEF\xBB\xBFid,,name\r\n1,40,"Sato, T"\r\n2,35\r\n`。BOM・`"Sato, T"` の引用・CRLF は元のまま。1 列だけの行は、足した列まで書き、`name` のセルは「セルが無い」のまま。開き直した画面も同じ |

🔴 **正直に:** キーはまだ（パレットだけ）。足した列の見出しの名前は変えられない（EDIT-04）。列を消す・戻すは application のテストだけ。Windows の実機ではまだ。

## 65. Ctrl++ と Ctrl+- で行と列を足し消しする（GitHub Issue #103・ADR 0020 の施主の判断・ADR 0027）— 仮想の X サーバでの確認（QLT-013）

PR の枝の debug の exe を Xvfb に出し、`id,name` ＋ 2 行の表で `xdotool` のキーを送った（2026-11-06）。

| # | 操作 | 見え |
| --- | --- | --- |
| 1 | Ctrl++（カーソルは本体の 1 行目の `id`） | 上に空の行が入り、番号は 1〜4・足した行の番号は強調の色・`1 行を足しました` |
| 2 | Ctrl+- | 足した行が消える |
| 3 | Ctrl+Space（列全体）→ Ctrl++ | `id` の左に列が入る・`1 列を足しました` |
| 4 | Ctrl+Z × 3 → Ctrl+0 | 開いた直後の表に戻る。Ctrl+0 は文字の大きさを 100% に戻す（行と列は変わらない） |

🔴 **正直に:** JIS の配列の Ctrl+Shift+; と、Windows の実機のキー（`VK_OEM_PLUS`）は試していない（単体テストだけ）。

## 66. 行と列を足し消ししただけの表を保存する（GitHub Issue #107）— 仮想の X サーバでの確認（QLT-013）

直す前は、セルを書き換えずに行か列を足し消しすると、Ctrl+S が「変えたセルが無いので保存しませんでした」になり、閉じるときも尋ねなかった。
PR の枝の debug の exe を Xvfb に出し、`id,name` ＋ 2 行（CRLF）の表で `xdotool` のキーを送った（2026-11-06）。

| # | 操作 | 見え・結果 |
| --- | --- | --- |
| 1 | Ctrl++（カーソルは `id`） | 2 行目に空の行が入る・題名の頭に `●`・`1 行を足しました` |
| 2 | Ctrl+S | `保存しました`・題名の `●` が消える |
| 3 | 保存したファイルを `od -c` | `id,name\r\n,\r\n1,Sato\r\n2,Kato\r\n`（足した行は列の数に合わせた空のセル・改行は原本の CRLF） |

閉じるときの確認と読み方を変えられないことは、application のテスト（`adding_or_removing_rows_and_columns_alone_is_unsaved`）と app の `Viewer::reading_blocked` が同じ `Document::has_unsaved` を使うことで見る。

## 67. 新しい表を開く（GitHub Issue #109・ADR 0028 の決定 2）— 仮想の X サーバでの確認（QLT-013）

PR の枝の debug の exe を `--new` で Xvfb に出し、`xdotool` のキーを送った（2026-11-06）。

| # | 操作 | 見え・結果 |
| --- | --- | --- |
| 1 | 開いた直後 | 題名 `無題 — NeNe Ledger`（`●` 無し）・列の番号の行（1 2 3）・1 行 × 3 列の空のセル |
| 2 | `abc` → Tab → `x` → Enter | 1 列目に `abc`、2 列目に `x`・題名の頭に `●` |
| 3 | Ctrl++ | 上に行が入り 2 行 |
| 4 | Ctrl+S | 保存しない（名前を付けて保存は次の Issue）。端末に `新しい表にはまだファイルが無い` |

保存の中身（BOM 付きの UTF-8・CRLF・カンマ・要るときだけ引用）は app のテスト `a_blank_table_is_planned_as_utf8_with_a_bom_crlf_and_commas` で見る。

## 68. 名前を付けて保存（GitHub Issue #111・ADR 0028 の決定 4・5）— 仮想の X サーバでの確認（QLT-013）

PR の枝の debug の exe を Xvfb に出し、`xdotool` のキーを送った（2026-11-06）。保存したファイルは `od -c` で見た。

| # | 操作 | 見え・結果 |
| --- | --- | --- |
| 1 | `--new` → `id` Tab `0012` Tab `a,b` Enter → Ctrl+S | パレットの欄が開き、初めの値は作業のフォルダ ＋ `無題.csv`。案内 `Enter で、この道筋に名前を付けて保存する（方言と文字コードは今のまま）` |
| 2 | Enter | 題名が `無題.csv — NeNe Ledger`（`●` 無し）・`保存しました`・状態の行の改行は `CRLF`。ファイルは `EF BB BF` `id,0012,"a,b"\r\n` |
| 3 | `rows.csv`（`id,name` ＋ 2 行・CRLF）で `Z` Enter → Ctrl+Shift+S → `other.csv`（既にある）Enter | 欄の下に `このファイルは既にあります。Enter をもう一度で上書き・Esc でやめる`（強調の色） |
| 4 | Enter | 題名が `other.csv — NeNe Ledger`。`other.csv` は `id,name\r\nZ,Sato\r\n2,Kato\r\n`。🔴 `rows.csv` は元のまま（`1,Sato`） |
| 5 | 深いフォルダの道筋 | 欄は頭を `…` で略し、ファイルの名前の側を見せる |
| 6 | 欄で Esc | 何も書かない（ファイルは増えない）・題名の `●` は残る |

🔴 **正直に:** Windows の OS の保存の画面（`GetSaveFileNameW`・`OFN_OVERWRITEPROMPT`）は `make lint-windows` で組み立てを確かめただけで、実機ではまだ開いていない（QLT-013）。

## 69. Ctrl+N で新しい表を新しい窓で開く（GitHub Issue #113・ADR 0028 の決定 1）— 仮想の X サーバでの確認（QLT-013）

PR の枝の debug の exe で `rows.csv` を Xvfb に出し、Ctrl+N を送った（2026-11-06）。

| # | 操作 | 見え・結果 |
| --- | --- | --- |
| 1 | Ctrl+N | 子のプロセス `nene-ledger --new` が起き、`無題 — NeNe Ledger` の窓（1 行 × 3 列）が前に出る。元の窓とプロセスは残る |
| 2 | 子を止める | 親の子の一覧に残らない（`ps --ppid` が空・裏のスレッドが待って引き取った） |

🔴 **正直に:** Windows の実機（`CreateProcessW`・新しい窓が前に出るか）はまだ。

## 70. Ctrl+V で貼る（GitHub Issue #115・CLIP-01 の 1）— 仮想の X サーバでの確認（QLT-013）

PR の枝の debug の exe で `rows.csv`（`id,name` ＋ 2 行・CRLF）を Xvfb に出し、`xclip` でほかのアプリのクリップボードを置いて `xdotool` のキーを送った（2026-11-06）。

| # | 操作 | 見え・結果 |
| --- | --- | --- |
| 1 | `xclip` に `0099\r\n` → ↓ → Ctrl+V | 3 行目の `id` が `0099` になる（`SelectionNotify` を待って受け取った・先頭の 0 を保つ）・題名に `●` |
| 2 | `xclip` に 2 行 × 2 列 → Ctrl+V | 表は変わらない。状態の行に `2 行 × 2 列の貼り付けはまだできません（1 つのセルか、編集中の欄には貼れます）` |
| 3 | `Sato` のセルで Ctrl+C → ↓ → Ctrl+V | 自分が持ち主のクリップボードから `Sato` を貼る |
| 4 | Ctrl+S → `od -c` | `id,name\r\n1,Sato\r\n0099,Sato\r\n` |

🔴 **正直に:** Windows の `CF_UNICODETEXT` の読み取りと、Excel からの貼り付けは実機でまだ。直した値の列の幅は広がらない（`0099` は `0…` と出る。手で打った値と同じ・前からの振る舞い）。

## 71. ダブルクリックでセルを直し始める（GitHub Issue #120・ADR 0026 の決定 1）— 仮想の X サーバでの確認（QLT-013）

PR の枝の debug の exe で `rows.csv`（`id,name` ＋ 2 行・CRLF）を Xvfb に出し、`xdotool` で `Sato` のセルをダブルクリックした（2 度の押しの間 120 ms・2026-11-06）。

| # | 操作 | 見え・結果 |
| --- | --- | --- |
| 1 | `Sato` をダブルクリック | セルが編集の状態になり、カーソルは末尾（欄は末尾が見えるよう `ato｜` を出す）。詳細ペインは `Sato` |
| 2 | `X` → Enter → Ctrl+S → `od -c` | `id,name\r\n1,SatoX\r\n2,Kato\r\n`（続けて直した・置き換えではない） |

🔴 **正直に:** Windows の実機（`WM_LBUTTONDBLCLK`・OS のダブルクリックの間）はまだ。

## 72. ファイル全体で改行の混在を見つける（GitHub Issue #128・OPEN-08）— 仮想の X サーバでの確認（QLT-013）

`id,name` ＋ LF の 20,000 行のあとに CRLF の 3 行を足した 238 KB の表（先頭の 64 KiB の標本には LF しか無い）を Xvfb で開いた（2026-11-06）。

| # | 時点 | 状態の行 |
| --- | --- | --- |
| 1 | 索引を作り終えた後 | `20,004 行 · UTF-8 · カンマ区切り · 改行が混在（CRLF 3 · LF 20,001 · CR 0） · 2 列` |

直す前は標本だけで数えるので `LF` と出る。数え方が参照実装（`Record::terminator`）と同じことは、scan の無作為の入力（`\r`・`\n`・引用・Shift_JIS・UTF-16）の索引の比較のテストが見る（`SparseIndex` の比較に内訳を含めた）。

### 索引の時間（1 GB・10 列・1,222 万行・release・3 回 × 2 巡を交互に）

| 版 | 時間 |
| --- | --- |
| 前（#127 の枝） | 4.78〜4.92 s |
| 後 | 4.22〜4.36 s |

1 バイトごとの比較を足さず、`\r` の後ろの状態でだけ `\r\n` と `\r` を数え、`\n` は終わったレコードの数から引いて出す。遅くはならなかった（速くなった分は、組み立ての並びの違いとみて、改善とは書かない）。最初の版（1 バイトごとに比べる）は 4.92 → 5.15 s で約 5% 遅かった。

## 73. 保存の進み具合と中止（GitHub Issue #132・SAVE-02・ADR 0025 の施主の判断 4）— 仮想の X サーバでの確認（QLT-013）

1 GB・10 列・1,222 万行の CSV を release の exe で Xvfb に開き、索引を作り終えてから列を足して Ctrl+S を押した（2026-11-06）。

| # | 時点 | 状態の行・ファイル |
| --- | --- | --- |
| 1 | Ctrl+S の 6 秒後 | `保存しています… 30%（Esc でやめる）` |
| 2 | Esc の 2 秒後 | `保存をやめました。ファイルは元のままです（直した値は残っています）`・題名の `●` は残る |
| 3 | ファイル | `cmp` で元と同じ・inode も同じ（置き換えていない）・`.nene-ledger-*.tmp` は 0 個 |

書いている途中でやめると、置き換えず一時ファイルを残さないことは store のテスト `a_cancelled_save_job_leaves_the_file_alone` で見る。

🔴 **正直に:** 割合は原本の大きさに対する目安（列を足すと少し越えるので 99% で止める）。Windows の実機はまだ。

## 74. 大きなクリップボード（INCR）を貼る（GitHub Issue #140・CLIP-01）— 仮想の X サーバでの確認（QLT-013）

PR の枝の debug の exe で `rows.csv`（25 バイト）を Xvfb に出し、`xclip` で大きな中身を置いて Ctrl+V を押した（2026-11-06）。`xclip` はこの大きさでは `INCR` で分けて渡す。

| # | クリップボード | 見え・結果 |
| --- | --- | --- |
| 1 | 15 万行 × 2 列のタブ区切り（2.18 MB） | `150,000 行 × 2 列の貼り付けはまだできません…`（全部を受け取って数えた・直す前は何も起きない） |
| 2 | `Z` × 600,000 の 1 つの値 | カーソルのセルに入り、Ctrl+S の後のファイルは 25 − 1 ＋ 600,000 ＝ 600,024 バイト（全部が届いた） |

64 MiB を越える中身を捨てることは ui-x11 のテスト `parts_are_joined_and_too_much_is_dropped` で見る。

## 75. 複数のセルの貼り付け（GitHub Issue #146・#147・CLIP-01・ADR 0029）— 仮想の X サーバでの確認（QLT-013）

PR の枝の debug の exe で `t.csv`（`id,name` ＋ 2 行・CRLF）を Xvfb に出し、`xclip` で `007⇥A⇥x` / `008⇥B⇥y` の 2 行 × 3 列を置いて、最後の行の 1 列目で Ctrl+V を押した（2026-11-06）。

| # | 操作 | 見え・結果 |
| --- | --- | --- |
| 1 | Ctrl+V | 最後の行に `007 A x`、足した 4 行目に `008 B y`。足した 3 列目が出る。状態の行は `2 行 × 3 列を貼りました（1 行と 1 列を足しました）`。貼った範囲を選び、カーソルは左上 |
| 2 | Ctrl+Z | 3 行 × 2 列の元の表。状態の行は `元に戻しました（6 セル）`。カーソルは 3 行目 1 列目 |
| 3 | Ctrl+Y → Ctrl+S | ファイルは `id,name,` / `1,Sato,` / `007,A,x` / `008,B,y`（CRLF）。足した列で短くなった行は空のセルで埋まる（ADR 0024 の施主の判断 4・5）。先頭の 0 は残る |

#2 は、直す前はカーソルが消えた 4 行目に残り、詳細ペインに「カーソルの行は見えている範囲の外」と出た。末尾に足した行を戻すときは Ctrl++ でも同じだった（GitHub Issue #147）。単体のテストは表示を読み直さないので通っていた。今は application のテスト `undoing_rows_added_at_the_end_leaves_the_cursor_on_a_real_row` が、読み直した表示で確かめる。

1 GB の表への 100 万セルの貼り付けの時間と記憶は benchmarks 第 7 節。

## 76. 切り取りと貼り付け（GitHub Issue #149・EDIT-16・ADR 0029 の決定 5・6）— 仮想の X サーバでの確認（QLT-013）

PR の枝の debug の exe を Xvfb に出し、`xdotool` で操作した（2026-11-06）。

| # | 表・操作 | 見え・結果 |
| --- | --- | --- |
| 1 | `c.csv`（見出し `id,name` ＋ `1,Sato` / `2,Kato` / `3,Ito`）で `2,Kato` の 2 つのセルを選んで Ctrl+X | 範囲に点線の枠。状態の行は `1 行 × 2 列を切り取りました（貼るまで元は残ります・Esc でやめる）`。値はまだ変わらない |
| 2 | `3,Ito` の 1 列目で Ctrl+V → Ctrl+S | `2,Kato` が `3,Ito` の所に入り、元の行は空のセル。状態の行は `1 行 × 2 列を動かしました`。ファイルは `id,name` / `1,Sato` / `,` / `2,Kato`（CRLF・`od -c`） |
| 3 | `e.csv`（`1,2` / `3,4` / `5,6`・見出しなし）で Ctrl+X → Esc → 1 つ下で Ctrl+V → Ctrl+S | 枠が消え、ただのコピーとして貼る。ファイルは `1,2` / `1,4` / `5,6`（元の `1` は残る） |

## 77. 検索の画面（GitHub Issue #152・FIND-01・FIND-05・ADR 0030 の決定 1・3・4）— 仮想の X サーバでの確認（QLT-013）

PR の枝の exe を Xvfb に出し、`xdotool` で操作した（2026-11-06）。

| # | 表・操作 | 見え・結果 |
| --- | --- | --- |
| 1 | `f.csv`（見出し `id,city,note` ＋ `1,Tokyo,a` / `2,Osaka,ｔｏｋｙｏ branch` / `3,Kyoto,x`）で Ctrl+F | パレットに探す段。欄の案内は `探す文字（Enter で次・Shift+Enter で前）`、下の行に `Alt+C 大文字小文字を区別しない · Alt+W 部分一致` |
| 2 | `tokyo` と打って Enter → Enter → Enter | 2 行目の `Tokyo` → 3 行目の全角の `ｔｏｋｙｏ branch`（全角半角を区別しない）→ 2 行目の `Tokyo` に戻り、状態の行に `表の端まで探して、反対の端から続けて見つけました` |
| 3 | 当たらない文字で Enter | カーソルは動かず `見つかりませんでした` |
| 4 | release の exe で 1 GB（1,222 万行）の索引を作り終えてから `R12224851` を探す | 2 秒後の状態の行は `探しています… 22%（Esc でやめる）`（画面は動く）。最後の行の 10 列目へカーソルが動き、窓がそこまで動いた。押してから CPU 時間が止まるまで 9.21 / 9.32 / 9.21 s（benchmarks 第 8 節） |

最初の版は、欄の下の行に Enter の案内と条件を全部並べ、パレットの幅から切れた（`Alt+W 部…`）。案内を欄の空のときの文に移した。

## 78. 置換の画面（GitHub Issue #161・#163・#165・FIND-02・ADR 0030 の決定 1・6）— 仮想の X サーバでの確認（QLT-013）

PR の枝の exe を Xvfb に出し、`xdotool` で操作した（2026-11-06）。

| # | 表・操作 | 見え・結果 |
| --- | --- | --- |
| 1 | `r.csv`（見出し `id,city,note` ＋ `1,Tokyo,tokyo tower` / `2,Osaka,` / `3,ＴＯＫＹＯ,Kyoto` / `4,tokyo2,x`）で Ctrl+H、`tok`・Tab・`TYO` | パレットに置換の段。欄に `TYO`、下の行に `探す: “tok”（Tab で替える）`、その下に `Enter 置換 · Alt+A 全部 · Alt+C 大小を区別しない · Alt+W 部分一致` |
| 2 | Enter（カーソルは 1 列目の `1`・当たらない） | 置き換えずに、次に当たる 2 列目の `Tokyo` へ動く |
| 3 | Alt+A | パレットが閉じ、状態の行に `4 セルを置き換えます（Enter で置き換える・Esc でやめる）`。表はまだ変わらない |
| 4 | Enter | `TYOyo`・`TYOyo tower`・`TYOＹＯ`（当たった全角の `ＴＯＫ` だけが替わり、`ＹＯ` は全角のまま）・`TYOyo2`。状態の行に `4 セルを置き換えました（Ctrl+Z で戻す）`、窓の名前に未保存の印 |
| 5 | Ctrl+Z | 4 セルとも元に戻り `元に戻しました（4 セル）`、未保存の印が消える |
| 6 | release の exe で 1 GB（1,222 万行）の索引を作り終えてから、`R12224851` / `Name12` / `Name1` を全部置換 | 1 セル・137,643 セル（`grep -c` と同じ数）・100 万セルを越えるので `100 万セルを越えるので置き換えませんでした（範囲や条件を狭めてください）`。時間と記憶は benchmarks 第 9 節 |

最初の版は、欄の下の行の置き換え方のキーと条件がパレットの幅から切れた（`… · Alt+`）。第 77 節と同じ失敗で、言葉を縮めた（`置き換えて次へ` → `置換`・`大文字小文字を` → `大小を`・置換の段だけ）。

## 79. OS の窓の名前の未保存の印（GitHub Issue #167・WIN-01）— 仮想の X サーバでの確認（QLT-013）

PR の枝の exe を Xvfb に出し、`xdotool` で操作して `xprop _NET_WM_NAME` を読んだ（2026-11-06）。

| # | 操作 | `_NET_WM_NAME` |
| --- | --- | --- |
| 1 | `r.csv` を開いた直後 | `r.csv — NeNe Ledger` |
| 2 | 右へ動いて `9` を打って Enter | `● r.csv — NeNe Ledger` |
| 3 | Ctrl+Z | `r.csv — NeNe Ledger` |

前は開いたときに 1 度だけ付けていて、自前のタイトルバーの `●` が OS の側（タスクバー・Alt+Tab・窓の管理者）に出なかった。同じ日の測りで、OS の窓の名前の `●` を待つ script が 120 s の上限まで待って気づいた。Windows（`SetWindowTextW`）は `make build-windows` と `make lint-windows` まで・実機はまだ。

## 80. core に `regex-automata` を足す（GitHub Issue #171・ADR 0031・ARC-013・CNF-013）

許可を書く前と後で回した（実ファイル・本物の依存・2026-11-06）:

| 検査 | 許可を書く前 | 許可を書いた後 |
| --- | --- | --- |
| CNF-013 | `crates/core/Cargo.toml` に `regex-automata` → `crates/core/Cargo.toml:16 — regex-automata への依存は crates/core/ に許されていない（ARC-002 / ARC-013）`・終了 1 | xtask の許可表（`ALLOWED_DEPENDENCIES` の `crates/core/`）に足す → `規約違反なし` |
| `make deny` | — | `advisories ok, bans ok, licenses ok, sources ok`（`MIT OR Apache-2.0` は既に許可。重複の版なし） |

`cargo tree -p nene-ledger-core -e normal`: `regex-automata` 0.4.18 が `regex-syntax` 0.8.11 だけを連れてくる（`alloc`・`syntax`・`meta`・`unicode`。速い道の `perf-literal-*` を入れないので `aho-corasick`・`memchr` は来ない）。core の `#![no_std]` のまま組み立つ（CNF-007 は変わらず通る）。

release の exe：2,603,088 → 4,180,800 バイト（+1.58 MB）。正規表現を使わない検索の速さ：ASCII でないセルだけの 42 MB で `--find` の CPU 時間を 8 回ずつ交互に測り、main 0.885 s・この枝 0.899 s（ばらつきの中）。

## 81. 正規表現の切り替え（GitHub Issue #173・FIND-03・ADR 0031）— 仮想の X サーバでの確認（QLT-013）

PR の枝の exe を Xvfb に出し、`xdotool` で操作した（2026-11-06）。

| # | 表・操作 | 見え・結果 |
| --- | --- | --- |
| 1 | `d.csv`（見出し `id,date,note` ＋ `1,2026/11/06,a` / `2,２０２５/０１/０２,b` / `3,none,c`）で Ctrl+H・Alt+R・`(\d+` | 置換の段の 2 行目に `Alt+C 大小を区別しない · Alt+W 部分一致 · Alt+R 読めない正規表現`（強調の色） |
| 2 | そのまま Enter | 探さず、状態の行に `正規表現の式が読めないので探しませんでした（Alt+R で文字どおりに）` |
| 3 | 式を `(\d{4})/(\d+)/(\d+)` にして Tab・`$3.$2.$1`・Alt+A・Enter | `06.11.2026` と `０２.０１.２０２５`（全角の日付は組の元の字のまま・`.` は置き換える文字の半角）・`none` は変わらない。`2 セルを置き換えました（Ctrl+Z で戻す）` |
| 4 | release の exe で 1 GB | benchmarks 第 10 節 |

最初の版は、読めない式の印 `正規表現（式が読めません）` がパレットの幅から切れた（第 77・78 節と同じ）。`読めない正規表現` に縮めた。置換の段は案内を 2 行にした（1 行目に置き換え方のキー、2 行目に条件）。

## 82. パレットの案内の長さ（GitHub Issue #175・QLT-013 の取りこぼしを機械に）

同じ日に、パレットの欄の下の案内が幅から切れる失敗を 3 度くり返した（第 77・78・81 節）。どれもスクリーンショットで気づいた。application のテスト `find_and_replace_guides_fit_the_narrowest_palette` が、探す段と置換の段を条件の全部の組み合わせ（大文字小文字・セル全体・正規表現 × 打った文字 `""`・`b`・読めない式 `(b` × 探す段・置換の段の両方の欄＝72 通り）で描き、打った文字を除く全部の字の表示の桁数（全角 = 2）が 64 桁以下であることを見る。

| 試したこと | 結果 |
| --- | --- |
| 読めない式の印を第 81 節で直す前の `正規表現（式が読めません）` に戻す | `74 桁: "Alt+C 大小を区別しない · Alt+W 部分一致 · Alt+R 正規表現（式が読めません）"（Find）` で落ちる |
| 今の `読めない正規表現` | 通る（いちばん長い案内は 64 桁） |

64 桁の根拠：Xvfb の画面写し（Noto Sans CJK JP・倍率 100%）で 1 桁 ≈ 6.6 px。パレットのいちばん狭い幅（480 px）で字を置ける幅 ≈ 444 px ≈ 67 桁。切れた案内は 74・78・83 桁、収まった案内は 64・65 桁。🔴 Windows の字体（DirectWrite）の幅では測っていない。

## 83. 版の表示と「このアプリについて」（GitHub Issue #179・ADR 0032 の決定 6）— 仮想の X サーバでの確認（QLT-013）

PR の枝の exe（2026-11-06）:

| # | 操作 | 結果 |
| --- | --- | --- |
| 1 | `nene-ledger --version` | `NeNe Ledger 0.1.0`・終了 0（`Cargo.toml` の版・`env!("CARGO_PKG_VERSION")`） |
| 2 | Xvfb で Ctrl+Shift+P・`about`・Enter | パレットに `NeNe Ledger 0.1.0` / `Copyright (c) 2026 Hideyuki Mori` / ライセンスの要旨 / `無保証（NeNe Ledger License 1.0・LicenseRef-NeNe-Ledger-1.0）` / `全文は LICENSE、部品の著作権表示は THIRD-PARTY-NOTICES.txt` / リポジトリの URL。どの行もパレットの幅に収まる（64 桁以下をテストで見る） |

Windows の exe の `--version`（コマンドプロンプトに結んで書く）は実機でまだ。`THIRD-PARTY-NOTICES.txt` は配る zip に入れる（次の Issue）。

## 84. 外部クレートの著作権表示の一覧（GitHub Issue #181・ADR 0032 の決定 6）

`make notices` が `cargo tree --target x86_64-pc-windows-gnu -e normal -p nene-ledger` の 23 クレート（`encoding_rs` とその依存・`regex-automata`・`regex-syntax`・`windows` の 12 クレートと、手続きマクロの `syn`・`quote`・`proc-macro2`・`unicode-ident`）について、`Cargo.toml` のライセンスの式と配布元のファイルから `THIRD-PARTY-NOTICES.txt`（32,607 バイト）を作った。選べるものは MIT の本文、`encoding_rs` は `COPYRIGHT` と BSD-3-Clause（`LICENSE-WHATWG`）も、`unicode-ident` は Unicode-3.0 も載せる。23 の節のどれにも著作権の行がある（2026-11-06）。

| 試したこと | 結果 |
| --- | --- |
| 一覧から 1 行（`==== simdutf8 …`）を消して `notices-check` | `ADR 0032: THIRD-PARTY-NOTICES.txt が Cargo.lock の依存と食い違う。…作り直す`・終了 1 |
| `make notices` で作り直す | 消す前と 1 バイトも違わない（`cmp`）・`notices-check` が通る |

## 85. 配る zip（GitHub Issue #183・ADR 0032 の決定 2・3）

`make dist`（`tools/dist.sh`）を手元で回した（2026-11-06・版 `0.1.0-preview.1`）。`notices-check` → `build-windows` → `dist/nene-ledger-0.1.0-preview.1-windows-x86_64.zip`（5,549,555 バイトを展開・10 項目：`nene-ledger.exe` 5,509,175 バイト・`LICENSE`・`THIRD-PARTY-NOTICES.txt`・`README.txt`・`samples/` の見本 4 つ）と `.sha256`。説明書の類は改行を CRLF にした（メモ帳で読めるように）。

タグから GitHub Actions で同じ `make dist` を回して Releases に置く `.github/workflows/release.yml` の初回（2026-11-07・タグ `v0.1.0-preview.1` = `1d7df4d`・run 42242279971）：全部の段が成功（版とタグの一致・`make dist`・プレリリースの公開）。公開物は `nene-ledger-0.1.0-preview.1-windows-x86_64.zip`（1,809,293 バイト）と `.sha256`。落とし直して `sha256sum -c` が OK、10 項目。

🔴 CI の exe（5,501,395 バイト）は、施主が試した手元の exe（5,514,003 バイト）と同じではない（ソースは同じ・mingw-w64 の版が違う）。CI の exe を施主の機械に置き、`--version` が `NeNe Ledger 0.1.0-preview.1`・10 秒後も検疫されないことを確かめた。窓を開いての操作は、手元の exe で施主が確かめた（第 86 節）。

## 86. 検索の欄から置換の欄への切り替え（GitHub Issue #187）— 施主の Windows の実機（QLT-013）

施主が候補の exe（0.1.0-preview.1）で「Ctrl+F で検索、そのあと Ctrl+H で置換、そのあとに Alt+A でコマンドパレットが閉じる」と報告した（2026-11-06）。施主の機械で、私が PowerShell の `WScript.Shell.SendKeys` でキーを送り、`PrintWindow` で窓を撮って再現した（施主が操作している間の 1 回目は、施主の操作と混ざって当てにならなかった・施主「しばらくさわらないからもういちどやって」）。

| # | 操作 | 直す前 | 直した後 |
| --- | --- | --- | --- |
| 1 | 検索の欄で `00` と打って Ctrl+H | パレットが閉じる（同じ欄を開くキーで閉じる扱い） | 置換の欄に切り替わり、`探す: “00”`・置き換える欄を打つ |
| 2 | 表から Ctrl+H で開いた置換の欄 | 探す欄が空（検索の欄の文字は Enter で探したときしか覚えない） | （1 で引き継ぐ） |
| 3 | 探す文字が空のまま Alt+A | パレットが閉じ、何もせず、何も知らせない | 閉じずに探す欄へ戻る（application のテスト） |
| 4 | 置き換える欄に `Z`・Alt+A・Enter | — | 状態の行に `6 セルを置き換えます（Enter で置き換える・Esc でやめる）` → `Z123`〜`Z128`・窓の名前に `●`・`6 セルを置き換えました（Ctrl+Z で戻す）` |

施主がこの直しの候補（`candidate-2`）で手順を試し「うん、できてるよ」（2026-11-07）。ほかの項目（開く・このアプリについて・貼り付けと切り取り・検索と置換・窓の名前の `●`・名前を付けて保存・Ctrl+N・ダブルクリック・保存の Esc）も施主「全部いいと思う」。Alt+ のキーは Steam などが握っていると効かないことがある（施主）。

展開先の exe が施主の開いている窓で使われていて上書きできず、1 度は古い exe を起動していた（`rm` の失敗で手順が止まったのに、続きの起動だけが走った）。新しい候補は別のフォルダ（`candidate-2`）に展開した。

## 87. 当たったセルと字の印・置換の件数をパレットの中に（GitHub Issue #189・ADR 0030 の決定 7）— X11（Xvfb）

2026-11-07（施主は外出中）。debug の exe を Xvfb（1024×480）に開き、`xdotool` でキーを送って `import` で撮った。見本は 5 行 4 列の CSV（`Tokyo Tower`・全角の `ＴＯＫＹＯ Dome`・`全角のＴＯＫ`・引用の中の改行の後ろの `tokyo`・セルに収まらない値の終わりの `tok`）。

| # | 配色 | 操作 | 見えたもの |
| --- | --- | --- | --- |
| 1 | 既定（暗い） | Ctrl+F → `tok` と打つ（Enter は押さない） | 当たった 6 つの字が明るい黄の背景に濃い字。全角の `ＴＯＫ` は 3 字ぶん・改行の印 `¶` の後ろの `tok`・切った値は `…` に印。当たったセルは薄い黄の背景（カーソルのセルは選択の色のまま）。見出しの行（`name`）には付かない |
| 2 | `neutral-light` | Ctrl+H → `tok` → Tab → `X` → Alt+A | パレットは閉じず、1 行目に強調の色で `6 セルを置き換えます · Enter 置き換える · Esc やめる`。表の印は数えた問いのまま |
| 3 | `neutral-light` | 2 の後に Enter | `Xyo Tower`・`XＹＯ Dome` ほか 6 セル。窓の名前に `●`。1 行目は `6 セルを置き換えました · 表で Ctrl+Z で戻す` |

🔴 Windows の実機（DirectWrite の幅で塗る所）はまだ見ていない（QLT-013）。`make lint-windows` は通る。施主の実機で見るまで「Windows で動く」と書かない。

## 88. 全角半角とひらがなカタカナの切り替え（GitHub Issue #199・FIND-06）— X11（Xvfb）

2026-11-07（施主は外出中）。`ガス`・`がす`・`ｶﾞｽ`・`gas` の 4 行を開き、Ctrl+F → Alt+Z → Alt+K。案内の 2 行目が `Alt+Z 全角半角を区別しない · Alt+K ひらがなカタカナを区別する`（既定）から `Alt+Z 全角半角を区別する · Alt+K ひらがなカタカナを区別しない` に変わった。照合そのものの答えは core のテスト（`options_make_the_match_stricter_or_looser`）。Windows の実機はまだ（QLT-013）。条件を設定に残して次の起動でも同じにすること（決定 3）はまだ。

## 89. core の直接の依存に `regex-syntax` を書く（GitHub Issue #223・ADR 0031 の追記・ARC-013・CNF-013）

許可を書く前と後で回した（実ファイル・本物の依存・2026-11-08）:

| 検査 | 許可を書く前 | 許可を書いた後 |
| --- | --- | --- |
| CNF-013 | `crates/core/Cargo.toml` に `regex-syntax` → `` crates/core/Cargo.toml:19 — `regex-syntax` への依存は crates/core/ に許されていない（ARC-002 / ARC-013） ``・終了 1 | xtask の許可表（`ALLOWED_DEPENDENCIES` の `crates/core/`）に足す → `規約違反なし（検査 831 ファイル）` |
| `make deny` | — | `advisories ok, bans ok, licenses ok, sources ok` |

`cargo tree -p nene-ledger-core -e normal`：`regex-automata` 0.4.18 → `regex-syntax` 0.8.11 と、直接の `regex-syntax` 0.8.11（同じ版が 1 つ・配るクレートは増えない）。`cargo build -p nene-ledger-core --target thumbv7m-none-eabi --locked` が通る（`no_std` のまま）。

## 90. 窓の行は見えている列の範囲だけを持つ（GitHub Issue #22・ADR 0033 の決定 1・3・4・BIG-04・BIG-08）

`prlimit --as=536870912`（アドレス空間 512 MiB）の下で、カンマだけの約 1 MiB の行 × 200（210 MB・2 億セル）を release の exe で Xvfb に開いた（2026-11-08・手で 1 回ずつ）：

| 版 | 結果 |
| --- | --- |
| 1 つ前（#231・窓の行は全部の列を持つ） | `memory allocation of 8388608 bytes failed` で落ちる（終了 -6） |
| この実装（見えている列の範囲だけを持つ） | 6.7 s で開き、最大の常駐 83,488 kB |

🔴 これは手の実測で、CI と `make test` は回さない（210 MB のファイルと窓が要る）。ADR 0033 の決定 4 の 1 つ目・2 つ目は planned のまま。3 つ目（窓の列を変えても答えが同じ）は application のテスト `holding_only_the_shown_columns_answers_like_holding_them_all` で active：種を固定した無作為の表 40 × 動き 60 回で、見えているセル（画面の配置から数えた列）・詳細ペイン・カーソルのセルの置換を全部の列を持つ表と突き合わせる。`held_columns` から (1) 左に固定した列を外す (2) カーソルの列を外す、(3) `apply` の後の読み直しをやめる、のどれでも落ちることを確かめた（戻した）。

画面（Xvfb）：20,000 列 × 50 行で行の終わり（20,000 列目）へ飛ぶと、窓を読み直して `r2c19999` などの値と、その列の幅が出る。行の終わりから左へ 30 列動いても値が出る。

## 91. 検索の印・全角半角とかなの切り替え・Ctrl+Enter・正規表現・数万列（GitHub Issue #234）— 施主の Windows の実機（QLT-013）

2026-11-08、main `b726991` の Windows の exe（`make build-windows`）と試しのファイルを施主の機械の試しの置き場（`candidate-1108`・場所は開発機の `CLAUDE.local.md`）に置き、施主が 5 つを確かめた。施主「うん、おおむねいいと思うよ」「全部見た」「引っかかったところはない」〔伝聞：施主の報告・私は画面を見ていない〕。

| # | 項目 | 試したこと | 施主の報告 |
| --- | --- | --- | --- |
| 1 | 当たった所の印（FIND-01・GitHub Issue #189・ADR 0030 の決定 7） | `find-sample.csv`（`ガス`・`がす`・`ｶﾞｽ`・`ＧＡＳ` の 6 行）で Ctrl+F → `ガス` → Enter。当たったセルと字の印（DirectWrite で字の幅を測って塗る `ui-win32` の `text_span`） | 引っかかりなし |
| 2 | Alt+Z / Alt+K（FIND-06・GitHub Issue #199） | 検索の欄で切り替え、案内の 2 行目と当たりの変わり | 引っかかりなし |
| 3 | Ctrl+Enter（EDIT-02・GitHub Issue #203） | 範囲を選んで打ち、Ctrl+Enter で範囲のすべてのセルに書き、Ctrl+Z で 1 手で戻す | 引っかかりなし |
| 4 | 正規表現の検索（FIND-03・GitHub Issue #223） | `regex-1gb.csv`（954,198,463 バイト・benchmarks 第 12 節と同じファイル）で Alt+R → `R1222485\d$` → Enter | 「１秒くらい？」（Linux の Xvfb では 1.60〜1.64 s・benchmarks 第 12 節） |
| 5 | 数万列の表（BIG-08・GitHub Issue #22） | `wide-20000.csv`（20,000 列 × 50 行・値は `r<行>c<列>`）で End で行の終わりへ・左右へ | 引っかかりなし |

メモリの使用量（タスクマネージャー）は見ていない（案内の「記憶」の言い方が伝わらなかった・施主「記憶の意味がよくわからなかった」）。Linux の測りは benchmarks 第 14 節。時間はどれも施主の手応えで、測っていない。

## 92. 読み取り専用に切り替える（GitHub Issue #238・EDIT-13）— X11（Xvfb）

2026-11-08。release の exe で `crates/app/fixtures/screenshot.csv` を開き、Ctrl+Shift+P → `yomitori` → Enter（「読み取り専用を切り替える」）→ `a`。タイトルバーが `ro.csv（読み取り専用） — NeNe Ledger` になり、`a` では直し始めず、状態の行に `読み取り専用です。直すには Ctrl+Shift+P の「読み取り専用を切り替える」で外します` と出た。止める操作の一覧と外した後に直せることは application のテスト `a_read_only_table_refuses_every_change_until_released`（ダブルクリックの守りと貼り付けの守りを外すと落ちることを確かめた）。Windows の実機はまだ（QLT-013）。

## 93. 窓にファイルを落とすと新しい窓で開く（GitHub Issue #243・CLIP-04）— X11（Xvfb）

2026-11-08。release の exe で `first.csv` を開き、XDND の送り手を真似る使い捨ての道具（x11rb 0.14.0・scratchpad・リポジトリには無い）から、`file://<ホスト名>/…/drop%20dir/%E6%9D%B1%E4%BA%AC%20a.csv` と `file:///…/first.csv` の 2 つを落とした。道具の見たこと：`XdndAware = 5`・`XdndStatus` は受け取る（1）で動きは写す・`SelectionRequest` の形は `text/uri-list`・`XdndFinished` は受け取った（1）で写す。落とした後の窓は `東京 a.csv — NeNe Ledger` と `first.csv — NeNe Ledger` が増えて 3 つ（元の窓は残る）。Windows の `WM_DROPFILES` は型と lint だけで、実機はまだ（QLT-013）。

## 94. 読み取り専用・ドラッグ＆ドロップ・Excel が開いているファイル（GitHub Issue #245）— 施主の Windows の実機（QLT-013）

2026-11-08、main `8ef8ed8` の Windows の exe（`make build-windows`）と試しのファイルを施主の機械の試しの置き場（`candidate-1108b`・場所は開発機の `CLAUDE.local.md`）に置き、施主が 3 つを確かめた。施主「うん、全部オッケー」〔伝聞：施主の報告・私は画面を見ていない〕。

| # | 項目 | 試したこと |
| --- | --- | --- |
| 1 | 読み取り専用（EDIT-13・GitHub Issue #238） | Ctrl+Shift+P → `yomitori` → Enter でタイトルバーに「（読み取り専用）」が付き、字を打っても直らず知らせが出る。もう一度切り替えると直せる |
| 2 | ドラッグ＆ドロップ（CLIP-04・GitHub Issue #243） | エクスプローラーから `drop-a.csv`・`drop-b.csv` を一緒に落とすと、それぞれ新しい窓で開き、元の窓は残る |
| 3 | Excel が開いているファイル（FILE-04・GitHub Issue #240） | `drop-a.csv` を Excel で開いたまま NeNe Ledger の窓へ落とすと、新しい窓が読み取り専用で開き、状態の行に「ほかのアプリ（Excel など）が開いているので、読み取り専用で開きました」 |

## 95. 外へ送らず、ほかのプログラムを起こさない（CNF-016・GitHub Issue #249・FR-031・SEC-01・SEC-05）

2026-11-08。CNF-012 の対象の外（scan・store）と Cargo.toml に、本物のファイルで禁じた行を書いて `cargo run -p xtask -- conformance` を回した（書いた行は戻した）：

| 書いた所 | 書いた行 | 結果 |
| --- | --- | --- |
| `crates/scan/src/source.rs`（CNF-012 の対象の外） | `use std::net::TcpStream;` | `CNF-016: crates/scan/src/source.rs:3 — `std::net` は外へ送る経路である（FR-031・SEC-05）` |
| `crates/store/src/saving.rs`（同） | `use std::process::Command;` | `CNF-016: crates/store/src/saving.rs:3 — `std::process::Command` はほかのプログラムを起こす。自分の exe を起こし直す new_window の外で使わない（FR-031・SEC-01）` |
| `crates/ui-win32/Cargo.toml` | `"Win32_Networking_WinHttp",` | `CNF-016: crates/ui-win32/Cargo.toml:39 — `Win32_Networking` は外へ送る機能である（FR-031）` |

`規約違反 3 件（CNF-016）`。戻した後は `規約違反なし（検査 427 ファイル）`。正例・反例は xtask のテスト（`cnf016_catches_sending_out_in_every_crate`・`cnf016_allows_spawning_only_the_own_exe_in_new_window`）。数式を評価しないこと（評価する処理が無いこと）は、数式に見えるセル（`=1+1`・`=HYPERLINK(…)`・`+cmd|…`・`@SUM(…)`・`=WEBSERVICE(…)`）が表とコピーで字のまま残る application のテスト `formula_like_cells_stay_as_text` と、触っていない行は保存でバイト列が同じテスト（KEEP-02）で示す。

## 96. JSON の整形・三点リーダー・列・キー・引用符・判別の候補・改行の混在・OS の文字の大きさ（GitHub Issue #266）— 施主の Windows の実機（QLT-013）

2026-11-08、main `e94f5d5` の Windows の exe と試しのファイル（`check.csv`・`single-quote.csv`・`mixed-eol.csv`）を施主の機械の試しの置き場（`candidate-1108c`・場所は開発機の `CLAUDE.local.md`）に置き、施主が 10 の項目を確かめた〔伝聞：施主の報告・私は画面を見ていない〕。

| # | 項目 | 施主の報告 |
| --- | --- | --- |
| 1 | 見えない文字の印の控えめな色（VIEW-05・GitHub Issue #261） | 「うーん、カラーのテーマのせいかな？薄い色かわからないな。よく見ると若干薄いかも？」→ 色の選び方を見直す |
| 2 | 詳細ペインの JSON の整形（VIEW-07・GitHub Issue #247） | 「出る」 |
| 3 | 三点リーダー（VIEW-17） | 「なってる」 |
| 4 | 列の幅と順のドラッグ（VIEW-02） | 「変わる」 |
| 5 | 左の列の固定（VIEW-01） | 「のこる。でも、狭めてlongだけ表示にした後広げてもdata列が出ない。出す方法が無い」→ GitHub Issue #265（Xvfb で再現した） |
| 6 | キーだけの操作（A11Y-01：Shift+Space・Ctrl+Space・Ctrl+Shift+PageUp / PageDown・キーの一覧） | 「大丈夫」 |
| 7 | 引用符の扱い（OPEN-05：シングルクォートを選んで開き直す） | 「OK」 |
| 8 | 判別の候補（OPEN-02） | 「OK」 |
| 9 | 改行の混在の知らせ（OPEN-08） | 「OK」 |
| 10 | OS の文字の大きさ（A11Y-03・GitHub Issue #263・「テキストのサイズ」を大きくして開き直す） | 「OK」 |

## 97. 印の中間の灰色と、固定した後に窓を広げること（GitHub Issue #271）— 施主の Windows の実機（QLT-013）

2026-11-08、main `476c5cb` の Windows の exe と `check.csv` を施主の機械の試しの置き場（`candidate-1108d`・場所は開発機の `CLAUDE.local.md`）に置き、施主が 2 つを見直した〔伝聞：施主の報告〕。

| # | 項目 | 施主の報告 |
| --- | --- | --- |
| 1 | 見えない文字の印を字の色と背景の中間の灰色で描く（VIEW-05・GitHub Issue #269）。本物の `→`・`¶` はふつうの濃さ | 「見える」 |
| 2 | 2 列目まで固定し、狭い窓で long 列まで右へ動いてから広げると、memo と data が戻る（VIEW-01・GitHub Issue #265） | 「大丈夫」 |

## 98. 2 行の見出しと、OS の高コントラスト（GitHub Issue #277・#275）— 施主の Windows の実機（QLT-013）

2026-11-08、main `5af6ea6` の Windows の exe と `two-header-rows.csv`（前書き 1 行 ＋ 2 行の見出し）を施主の機械の試しの置き場（場所は開発機の `CLAUDE.local.md`）に置き、施主が 2 つを見た〔伝聞：施主の報告〕。

| # | 項目 | 施主の報告 |
| --- | --- | --- |
| 1 | パレットの「見出しの行を選ぶ…」で「2〜3 行目が見出し」を選ぶと、2 行とも上に固定され、詳細ペインの見出しが「売上 / 今年」になる（OPEN-11・ADR 0034・GitHub Issue #277） | 「OK」 |
| 2 | OS の高コントラストを有効にして起動すると、高コントラストの配色で開く（A11Y-02・GitHub Issue #275） | 「OK」 |

## 99. 前書きの行・任意の区切りの字・影と角丸（GitHub Issue #281）— 施主の Windows の実機（QLT-013）

2026-11-08、main `d530458` の Windows の exe と `preamble.csv`（前書き 2 行 ＋ 見出し）・`tilde.csv`（`~` 区切り・セルに `,` `;` `|`）を施主の機械の試しの置き場（場所は開発機の `CLAUDE.local.md`）に置き、施主が 4 つを見た〔伝聞：施主の報告〕。`tilde.csv` の手順は先に Xvfb で通した。

| # | 項目 | 施主の報告 |
| --- | --- | --- |
| 1 | 前書きの 2 行が字も行番号も薄く、見出しが上に固定され、開いた直後のカーソルがデータの 1 行目にある（OPEN-11・GitHub Issue #273） | 「OK」 |
| 2 | `~` は判別されず 1 列に並び、パレットの「区切りを選んで開き直す…」で `~` を打つと 4 列に分かれ、セルの `A,B`・`C;D`・`E\|F` はそのまま、状態の行は「「~」区切り」（OPEN-01・Issue #67） | 「OK」 |
| 3 | 最大化ボタンに指を乗せたままにすると Snap Layouts が出る（WIN-05） | 「でない」→ GitHub Issue #282 |
| 4 | 窓の周りの影と角の丸み（WIN-06・Issue #55） | 「OK」 |

## 100. Windows 11 の Snap Layouts（GitHub Issue #282・WIN-05）— 施主の Windows の実機（QLT-013）

2026-11-08、第 99 節の 3 で Snap Layouts が出なかった。施主の機械では、ほかのアプリ（エクスプローラー・メモ帳）では出る〔伝聞：施主の報告〕。原因を探すため、main `dd302ea` から試しの exe を作り（リポジトリには入れていない）、施主の機械の試しの置き場で施主が見た。

| 試し | 変えたこと | 施主の報告 |
| --- | --- | --- |
| A | 窓の部品の上で `TrackMouseEvent` を呼ばない | 出ない（指を外すとボタンの色は消える） |
| B | `TrackMouseEvent` に `TME_HOVER` も頼む | 出ない |
| 記録 | 窓の手続きに届いたメッセージを全部ファイルに書く | 出ない。最大化ボタンの上で 3.6 秒止めた間、`WM_NCHITTEST` は毎回 9（`HTMAXBUTTON`）を返し、`WM_NCMOUSEMOVE`（wparam 9）と `WM_SETCURSOR`（当たり 9）は全部既定の手続きに渡っていた。窓の型は `0x04CF0000`（`WS_OVERLAPPEDWINDOW` ＋ `WS_CLIPSIBLINGS`） |
| C | `DwmExtendFrameIntoClientArea`（上の枠を 1 画素中身へ広げる）を呼ばない | **出た**。影と角丸は「いいと思う」 |

当たり判定とメッセージの渡し方は Microsoft の説明どおりで、違いは DWM の枠を中身へ広げるかだけだった。GitHub Issue #282 で C と同じ変更を入れた（ui-win32 の `decorate`）。Windows 11 は角丸（`DWMWA_WINDOW_CORNER_PREFERENCE`）の窓に影を付けるので、影は残る。Windows 10 の影は確かめていない。

## 101. 保存先のロックと置き換えの再試行 — Windows の上の単体テスト（GitHub Issue #336・FILE-04・ADR 0025 の施主の判断 3）

`make test-windows`（WSL の相互運用で Windows 11 の上で直に走らせる・GitHub Issue #211）で、`adapters-windows` の `file_replacing` の試験を走らせた（2026-11-09・積んだ枝 #335 の上）。

| 試験 | 何を確かめたか | 結果 |
| --- | --- | --- |
| `an_unheld_file_is_replaced` | 掴まれていない原本は `ReplaceFileW` で一時ファイルの中身に置き換わり、一時ファイルは無くなる | ok |
| `a_file_held_without_sharing_stays_and_is_reported_locked` | ほかのアプリが共有の指定なし（`share_mode(0)`）で掴んでいる原本は、再試行（50・100・200・400・800 ms）の後に `Locked`。原本は元のまま、一時ファイルも残る。放せばもう一度で置き換わる | ok |

`nene_ledger_adapters_windows` の 7 件が 1.56 秒で通った（再試行の待ちの合計 1.55 秒と合う・時間は試験では見ていない）。

🔴 **正直に:** Excel そのものでの確かめは第 60 節の 3（2026-11-05・施主の実機・書き始めない確かめと閉じてからの再保存）。新しい知らせの文言（「閉じてから Ctrl+S · 別名は Ctrl+Shift+S」）は実機でまだ見ていない。

## 7. active の規則と、その証明

CNF-009 は active の規則ごとに `| ID |` で始まる行がここにあることを要求する。

| 規則 | 最小の違反 | 検証経路 | 実測 |
| --- | --- | --- | --- |
| RS-001 | 非公開 newtype をモジュール外で直接構築 | `make prove` P-15 | E0603 |
| RS-002 | `match` に `_`・variant の漏れ | P-04 / P-05 | `wildcard_enum_match_arm` / E0004 |
| RS-003 | 未初期化・部分構築・`Default` | P-16 / P-17 ＋ CNF-001 | E0381 / E0063 / xtask |
| RS-004 | `unwrap` | P-13 | `unwrap_used`（forbid） |
| RS-005 | `unwrap`・slice の添字 | P-13 / P-14 | `unwrap_used` / `indexing_slicing` |
| RS-006 | `dyn Any` | CNF-002 の反例テスト | xtask |
| RS-007 | `as` で数値変換 | P-06 | `as_conversions` |
| RS-008 | 未文書の公開関数 | P-09 | `missing_docs` |
| RS-009 | `OnceLock` | P-18 ＋ CNF-002 | `unsafe_code` / xtask |
| RS-010 | `unsafe` ブロック・`build.rs` | P-18 ＋ CNF-002 / CNF-008 / CNF-014 | `unsafe_code` / xtask |
| RS-011 | 認知的複雑度 13・62 行の関数 | P-11 / P-12 | `cognitive_complexity` / `too_many_lines` |
| RS-012 | `mod.rs`・2 つ目の主要宣言 | P-10 ＋ CNF-003 | `mod_module_files` / xtask |
| RS-013 | 束縛名 `data`・型名の語尾 `Manager` | P-26 ＋ CNF-004 | `disallowed-names` / xtask |
| RS-014 | `println!` | P-19 | `print_stdout` |
| RS-015 | core の `std::time`・ui の `std::env` | CNF-012 の反例テスト | xtask |
| RS-016 | `HashMap` | P-07 | `disallowed-types` |
| RS-017 | release で `255_u8 + 1_u8` | P-21 | `attempt to add with overflow`・終了 101 |
| RS-018 | 型注釈の無いリテラル | P-08 | `default_numeric_fallback` |
| RS-019 | forbid を `#[expect]`・腐った `#[expect]`・`#[allow]` | P-01 / P-02 / P-03 ＋ CNF-006 | E0453 / unfulfilled / `allow_attributes` |
| ARC-003 | core の `#![no_std]` を外す・core で `std::time`・core に `regex` | CNF-007 / CNF-012 / CNF-013（実ファイル・第 8 節）＋ `cargo check` | xtask / E0433 |
| QLT-009 | 閾値を実測より上に置く | `make coverage`（第 8 節） | cargo-llvm-cov・終了 1 |
| ARC-002 | クレートの循環・xtask に依存を足す | P-23 ＋ CNF-013（実ファイル・第 5 節） | `cyclic package dependency` / xtask |
| ARC-007 | アダプタ・scan・store の外で `std::time` | CNF-012 の反例テスト | xtask |
| ARC-013 | 許可外のライセンス・先回りの許可・lock 無し・xtask の依存 | P-24 / P-25 / P-22 ＋ CNF-013（実ファイル・第 5 節） | cargo-deny / `--locked` / xtask |
| QLT-002 | 警告・整形・rustdoc | P-01〜P-20・`make doc-check` | clippy / rustfmt / rustdoc |
| QLT-003 | `#[allow]`・設定での `allow` | P-03 ＋ CNF-011（実ファイル・第 5 節） | `allow_attributes` / xtask |
| QLT-004 | 整形の差分 | P-20 ＋ CNF-011 | `cargo fmt --check` |
| QLT-011 | lock の無い状態で `--locked` | P-22 | `cannot create the lock file` |
| CNF-001 | `#[derive(Default)]` ほか | `cargo test -p xtask` の正例・反例 | xtask |
| CNF-002 | `dyn Any`・`OnceLock` ほか | 同上 | xtask |
| CNF-003 | 2 つ目の主要宣言 | 同上 | xtask |
| CNF-004 | 型名の語尾・モジュール名 | 同上 | xtask |
| CNF-006 | 規則 ID の無い `reason` | 同上 | xtask |
| CNF-007 | `-core` の `lib.rs` に `#![no_std]` が無い | 同上 | xtask |
| CNF-008 | `build.rs` | 同上 | xtask |
| CNF-009 | 本文とマトリクスの不一致・証明行の欠落 | 同上 ＋ 実ファイル（第 5 節） | xtask |
| CNF-010 | Issue 番号の無いタスクの印 | 同上 ＋ 実ファイル（第 5 節） | xtask |
| CNF-011 | 設定での `allow` | 同上 ＋ 実ファイル（第 5 節） | xtask |
| CNF-012 | 中核と UI の `std::time` / `std::env` | 同上 | xtask |
| CNF-013 | 許されない依存 | 同上 ＋ 実ファイル（第 5 節） | xtask |
| CNF-014 | unsafe を許すクレートの lint の表のずれ | 同上 | xtask |
| CNF-015 | scan の `File::create`・`.write(true`・書き込める写像 | 同上 | xtask |
| CNF-016 | どのクレートでも `std::net`・`new_window` の外の `Command`・`Win32_Networking` | 同上 ＋ 実ファイル（第 95 節） | xtask |
| GIT-001 | `Closes #N` の無い PR 本文 | 単体テスト ＋ CI の `make ci`（第 51 節） | xtask |
| GIT-002 | 形の違う枝名・`main` への直接 push | `validate-git`（第 6 節）＋ ruleset（第 51 節） | xtask / GitHub |
| GIT-003 | 形の違う件名・PR タイトル | commit-msg フック（第 6 節）＋ CI（第 51 節・実行 41832778822 が失敗） | xtask |
| GIT-004 | 欄の無い PR 本文・squash 以外のマージ | 単体テスト ＋ CI（第 51 節）＋ ruleset（第 51 節） | xtask / GitHub |
| GIT-005 | 管理するファイル・コミット・PR 本文の絶対パス | `validate-git`（第 6 節・第 51 節） | xtask |
| QLT-005 | CI 側にだけある検査 | `check.yml` は `make ci` だけを呼ぶ。手元と同じ出力（第 51 節） | GitHub Actions |
| QLT-012 | draft の PR で必須 check が走る・欄の無い本文 | 単体テスト（`pull_request_rejects_draft_missing_close_and_bad_title`）＋ `check.yml` の `if: draft == false` | xtask / GitHub Actions |
