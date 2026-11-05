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
| GIT-001 | `Closes #N` の無い PR 本文 | 単体テスト ＋ CI の `make ci`（第 51 節） | xtask |
| GIT-002 | 形の違う枝名・`main` への直接 push | `validate-git`（第 6 節）＋ ruleset（第 51 節） | xtask / GitHub |
| GIT-003 | 形の違う件名・PR タイトル | commit-msg フック（第 6 節）＋ CI（第 51 節・実行 41832778822 が失敗） | xtask |
| GIT-004 | 欄の無い PR 本文・squash 以外のマージ | 単体テスト ＋ CI（第 51 節）＋ ruleset（第 51 節） | xtask / GitHub |
| GIT-005 | 管理するファイル・コミット・PR 本文の絶対パス | `validate-git`（第 6 節・第 51 節） | xtask |
| QLT-005 | CI 側にだけある検査 | `check.yml` は `make ci` だけを呼ぶ。手元と同じ出力（第 51 節） | GitHub Actions |
| QLT-012 | draft の PR で必須 check が走る・欄の無い本文 | 単体テスト（`pull_request_rejects_draft_missing_close_and_bad_title`）＋ `check.yml` の `if: draft == false` | xtask / GitHub Actions |
