# Rust コーディング規約 — NeNe Ledger

> Status: normative（規範）/ 2026-10-22 初版
> 判断の根拠は [ADR 0001](adr/0001-strictness-is-mechanically-enforced.md)。
> 本書は Rust（edition 2024）を本リポジトリで承認された部分集合に狭めるためのものであり、
> ここが沈黙している領域は公式の Rust スタイル（Rust API Guidelines / rustfmt の既定）に従う。
> 出典は NeNeDocket `docs/CODING_RULES.md`（Rust での 4 回目の適用）。P-NN は本リポジトリの `make prove` の番号（Issue #2 で測った・NeNeDocket と同じ番号）。

読み方（**active / planned / 不能 / 不採用**）と強制の層は
[ARCHITECTURE_CONSTITUTION.md](ARCHITECTURE_CONSTITUTION.md) 第 0 節と同じ。
🔴 Issue #2 で機械を置き、このリポジトリの道具で発火を実測した規則だけを active にした。実測は [quality/gate-proofs.md](quality/gate-proofs.md)。

---

## 1. 型と状態（RS-0xx）

### RS-001 — 境界でのプリミティブ執着を禁じる

行番号・列番号・バイト位置・版番号・画素数のように**単位や不変条件を持つ値**は、非公開フィールドを持つ newtype にする。
生成経路は唯一のファクトリ（`new` が `Result` か `Option` を返す）に限る。

🔑 `RowIndex(1)` はモジュールの外では E0603。**唯一のファクトリを言語が保証する。**

- 機械強制: **active**（コンパイラ E0603。実測 P-15）
- 補足: newtype にすべき値かどうかの判断はレビュー事項

### RS-002 — 閉じた選択肢は `enum` で表す。`_` を書かない

区切り文字・文字コード・改行・比較の方法・部品の種類・一括処理の状態を `bool` の組み合わせ・マジック整数・裸の文字列で表さない。
`match` の網羅性はコンパイラが見る（E0004）。したがって**規約が守るべきなのは「網羅性検査を無効化しないこと」**である。
`_` を書いた瞬間に検査は死ぬ。自クレートの `enum` に `#[non_exhaustive]` を付けない。

- 機械強制: **active**（E0004 ＋ `clippy::wildcard_enum_match_arm` / `wildcard_in_or_patterns` を **forbid**。実測 P-04 / P-05）

### RS-003 — 不変条件を持つ型に「作り置きの値」を用意しない

未初期化の変数は E0381、構造体の部分構築は E0063 で落ちる。
**しかし `#[derive(Default)]` を書けば自分でゼロ値を作れてしまう。** これが Rust に残る実質的な穴である。

- `Default` を実装・derive しない（型を問わず。「不変条件を持つ型に限る」は機械が判定できない）
- `..Default::default()` による構造体の穴埋めを書かない
- 不変条件を持つ型は**自分のモジュールに単独で置く**（子モジュールは祖先の非公開項目に到達できる）

- 機械強制: **active**（E0381・E0063（実測 P-16 / P-17）・`Default` の禁止＝CNF-001。単独モジュール＝CNF-005 は planned・判定方法が未定）

### RS-004 — 不在は `Option` だけが表す

`Option::None` は「省略可能な値が無い」だけを意味する（見出しの行が無い・選択が無い）。無効・未読込・失敗・未知を表さない。それらは専用の `enum` を作る。**空のセル（`""`）は `None` ではない。** 空文字列と「セルが無い（列が足りない行）」を区別する。
`Option<Result<T, E>>` と `Result<Option<T>, E>` を場当たりに混ぜない。

- 機械強制: **active**（言語に `null` が無い。`unwrap_used` / `expect_used` を **forbid**。実測 P-13。`Option` の意味の流用そのものはレビュー事項）

### RS-005 — 期待される失敗は `Result`。panic しない

検証エラー・見つからない・拒否・非互換・読めない版・判別できない方言・閉じていない引用符・ロック・OS の要求の失敗は `Result` で返す。

**禁止（forbid・抑制不可）**: `unwrap` / `expect` / `panic!` / `unreachable!` / `v[i]` 添字アクセス / `std::process::exit` / `mem::forget`。

- **テストコードは対象外**（`clippy.toml` の `allow-*-in-tests`）。免除は `clippy.toml` で行い、テスト側に `#[expect]` を書かせない
- `std::process::exit` は `main` の中では clippy が見逃す。終了コードは `main` から `ExitCode` で返す

- 機械強制: **active**（上記 forbid 群 ＋ `panic_in_result_fn` / `unwrap_in_result`。実測 P-13 / P-14）

### RS-006 — 汎用データバッグを禁じる

`dyn Any`・`BTreeMap<String, String>` を型の代用にしない。列を「見出しの文字列をキーにした辞書」で持たない（見出しは重複も空もありうる。列は番号で指す）。意味のある値を `(A, B)` のタプルで持ち回らない。名前付きの型を作る。

- 機械強制: **active**（`dyn Any` の禁止＝CNF-002。文字列キーのバッグ・意味を持つタプルの検出はレビュー事項）

### RS-007 — 数値変換に `as` を使わない

`as` は黙って切り詰める。`From` / `TryFrom` のどちらか一つを使う。画素数と cairo / Direct2D の浮動小数の間も、`u64` のバイト位置と `usize` の間も同じ（32 ビットの環境は対象外でも、変換は明示する）。

- 機械強制: **active**（`clippy::as_conversions` を **forbid**。実測 P-06）

---

## 2. 構築と可視性

### RS-008 — 可視性は最小

- 既定は非公開。`pub(crate)` で足りるものを `pub` にしない
- 公開項目には doc コメントを付ける（`missing_docs`）
- 構造体のフィールドは全公開か全非公開。混在させない
- 再エクスポート（`pub use`）を書かない。**同じ物に二つ目の名前を作る行為**である

- 機械強制: **active**（`unreachable_pub`・`missing_docs`・`partial_pub_fields`・`field_scoped_visibility_modifiers`・`clippy::pub_use`。実測 P-09）

### RS-009 — 可変グローバルを持たない

`static mut` は `unsafe` を要するので、`unsafe_code = "forbid"` により**書けない**。
`OnceLock` / `LazyLock` / `lazy_static` による遅延グローバルは書けてしまう。使わない。

- 機械強制: **active**（`unsafe_code` の forbid ＋ CNF-002）

### RS-010 — 言語マジックを禁じる

- `unsafe` を書かない（X11 の接続も cairo も safe な API だけを使う。`unsafe` が要る経路は ADR）
- 例外は 3 クレートだけ: Windows の 2 クレート（`adapters-windows` / `ui-win32`・ADR 0002）と、原本の写像と SIMD の探索を担う `scan`（ADR 0006）。`unsafe` を要する小さな関数ごとに `#[expect(unsafe_code, reason = "RS-010: …")]` と `// SAFETY:` の注釈を置く
- `scan` の `unsafe` の経路は、必ず core の参照実装と同じ結果を出すことをテストで固定する（ADR 0006・QLT-008）
- 手続きマクロを自作しない。`build.rs` を書かない。アイコンの資源を埋める必要が出たら、nene-ruler ADR 0023 と同じ形で例外を 1 つだけ ADR で足す
- 宣言的マクロ（`macro_rules!`）は、同じ形の重複が 3 箇所以上あり、かつ関数と generics で書けないときに限る

- 機械強制: **active**（`unsafe_code` を **forbid**（実測 P-18）。unsafe を許す 3 クレートは `deny`＋`undocumented_unsafe_blocks`＋CNF-014。proc-macro の禁止＝CNF-002。`build.rs` の禁止＝CNF-008。`macro_rules!` の判断はレビュー事項）

### RS-011 — 複雑度に上限を置く

| 指標 | 上限 | lint |
| --- | --- | --- |
| 認知的複雑度（関数） | 10 | `clippy::cognitive_complexity` |
| 関数の長さ | 60 行 | `clippy::too_many_lines` |
| ネストの深さ | 4 | `clippy::excessive_nesting` |
| 引数の数 | 4 | `clippy::too_many_arguments` |
| 型の複雑さ | 250 | `clippy::type_complexity` |

閾値を満たすためだけに意味のある処理を割るのは目的に反する。超える必要があるときは**測定可能な理由**を添えて ADR にする。

🔴 `cognitive_complexity` は `nursery` 群にあり、`clippy.toml` に閾値を書くだけでは死んだ設定になる。`Cargo.toml` で名指しで有効化してある。

- 機械強制: **active**（実測 P-11 / P-12）

### RS-012 — ファイルとモジュールの置き方を一つに固定する

- `mod.rs` を作らない。`foo.rs` ＋ `foo/` の形だけを使う
- 1 ファイルに主要な宣言（`struct` / `enum` / `trait` / `union`）は 1 つ。`utils.rs` のような寄せ集めを作らない

- 機械強制: **active**（`clippy::mod_module_files`（実測 P-10）＋ CNF-003）

### RS-013 — 名前が役割を語る

常に禁止する型名の語尾: `Manager` / `Helper` / `Util` / `Utils` / `Common`。
常に禁止するモジュール名: `utils` / `helpers` / `managers` / `misc` / `common`。
常に禁止する束縛名: `foo` / `bar` / `baz` / `tmp` / `data` / `val` / `util` / `utils` / `helper` / `common`。

`Processor` や `Renderer` のように**文脈次第で妥当な語は機械では拒否しない**。判断が要る語はレビューの仕事である。

- 機械強制: **active**（束縛名は `clippy.toml` の `disallowed-names`（実測 P-26）、型名の語尾とモジュール名は CNF-004）

---

## 3. 実行時の規律

### RS-014 — 出力は合成ルートに集約する

標準出力・標準エラーに書いてよいのは合成ルート（`crates/app` の `main`）と `xtask` の `main` だけ。
`print_stdout` / `print_stderr` は `deny` なので、許可された場所は `#[expect]` で自己申告する。
**「どこが出力してよい場所か」がコードを grep すれば全部出る。**

- 機械強制: **active**（`clippy::print_stdout` / `print_stderr` / `dbg_macro`。実測 P-19）

### RS-015 — プロセス環境に触るのは adapters と scan と store と合成ルートだけ

環境変数・ファイル・現在時刻・乱数・スレッドを読んでよいのは `crates/adapters-x11`・`crates/adapters-windows`・`crates/scan`・`crates/store` だけ。
コマンドライン引数を読んでよいのは合成ルートだけ。中核と UI は値として受け取る。

- 機械強制: **active**（CNF-012 の字句検査。中核は `#![no_std]` で名前解決エラーが加わる（ARC-003））

### RS-016 — 反復順は決定的

`HashMap` / `HashSet` を使わない。**反復順が実行ごとに変わり、出力が再現しなくなる。** `BTreeMap` / `BTreeSet` を使う。
並べ替えの結果・集計の行の順・書き出しの順・検査器の報告順が実行ごとに変わると、差分が取れず、FR-003 / FR-009 が崩れる。並列の処理の結果の合わせ方も、スレッドの終わる順に依らないこと。

- 機械強制: **active**（`clippy.toml` の `disallowed-types` ＋ `clippy::iter_over_hash_type`。実測 P-07）

### RS-017 — 整数のオーバーフローを黙って巻き戻さない

リリースビルドでも `overflow-checks = true` にする。バイト位置・行数・画素の足し算が黙って巻き戻る経路を消す。

- 機械強制: **active**（`Cargo.toml` の `[profile.release]`。実測 P-21）

### RS-018 — 数値リテラルは型を明示する

型注釈の無い数値リテラルは `i32` に落ちる。`400_u32` / `1.5_f64` と書く。

- 機械強制: **active**（`clippy::default_numeric_fallback`。実測 P-08）

### RS-019 — 抑制は二段構え

| 段 | 水準 | 抑制 | 使う場面 |
| --- | --- | --- | --- |
| 1 段目 | `forbid` | **不可能**（E0453） | 例外を認めた瞬間に規則の意味が消えるもの |
| 2 段目 | `deny` | `#[expect(lint, reason = "<規則 ID>: <理由>")]` **のみ** | 「唯一の許可された場所」を宣言させたいもの |

- `clippy::allow_attributes` が **forbid** なので `#[allow]` は書けない。抑制は `#[expect]` だけ
- `clippy::allow_attributes_without_reason` が **forbid** なので、理由の無い抑制は書けない
- `unfulfilled_lint_expectations` が **deny** なので、**発火しなくなった `#[expect]` は失敗する**
- `reason` は `<規則 ID>: <理由>` の形で、その規則 ID が規範文書に実在すること（CNF-006）

⇒ 抑制は「必ず理由付きで」「必ず今も必要で」しか存在できない。**古い抑制が黙って腐り続ける経路が無い。**
🔑 これは C++ の先行事例が waiver 台帳（人手）で達成していたものを、コンパイラが代わりに執行している状態である。
だから本リポジトリは waiver 台帳を持たない（[ADR 0001](adr/0001-strictness-is-mechanically-enforced.md)）。

- 機械強制: **active**（実測 P-01 / P-02 / P-03 ＋ CNF-006）

---

## 4. 画面と OS の規約

### 共通

- 部品（表・セレクトボックス・入力欄・ボタン・スクロールバー）の配置と当たり判定は core が物理画素で計算する。UI はそれを描き、クリックと鍵を意図に変えるだけ（ARC-011）
- 描画は画面外の面（cairo の `ImageSurface` / Direct2D のビットマップ）に完成させてから窓へ転送する。更新途中の画面を見せない。画素の列は OS 無しでテストする（ADR 0002）
- 時刻は `ClockPort` から入る。UI がタイマーを持つなら、それは再描画や読み直しのきっかけであって値の正本ではない（ARC-007）
- 文字入力は IME を通る。確定前の文字列（preedit）は編集状態の一部として application が持ち、UI は描くだけ（FR-020・方式は ADR）

### X11 と cairo

- 窓・GC・`ImageSurface`・選択の所有は型の `Drop` で閉じる。`unsafe` を要する資源は持たない（RS-010）
- X の要求（`x11rb` の `Result`）を `?` で application の閉じた失敗へ写す。`unwrap` はテストにも書かない（RS-005）
- `XlibSurface` / `XCBSurface` は `unsafe` が要るので使わない（CNF-002）
- X のイベントは開いた集合なので、`x11rb::protocol::Event` の `match` にだけ `_` を許す。閉じた業務 `enum` の `_` とは区別する（RS-002）。
  その `_` は `#[expect(clippy::wildcard_enum_match_arm, …)]` では書けない（forbid）ので、`if let` の連なりで書く

### 原本の読み取り（scan）

- 原本のバイト列は読むだけ。写像を書き込み可能で開かない（ADR 0005 / ADR 0006）
- 写像を読んでいる間に他のプロセスがファイルを縮めると、Linux では読んだ瞬間にプロセスが落ちうる（SIGBUS）。この危険をどう扱うかは ADR 0006 の導入の Issue で決め、決まるまで写像を製品の経路に入れない
- SIMD の経路は CPU の機能を実行時に確かめてから使い、使えなければ素朴な経路へ落とす。どちらの経路も core の参照実装と同じ結果を出すことをテストで固定する（QLT-008）
- 並列の読み取りは、切り方（スレッドの数・区切りの位置）に依らず同じ結果を出す（ARC-007）

### Win32 と Direct2D

- Win32 を呼ぶ関数は小さく切り、1 つずつ `#[expect(unsafe_code, reason = "RS-010: …")]` と `// SAFETY:` を置く（ADR 0002）
- COM の資源は `windows` クレートの型の `Drop` で解放する。生のポインタを構造体に持ち回らない
- `WM_DPICHANGED` は `DpiChanged` の意図として application に渡す。UI が倍率を自分で持たない（ARC-004）

---

## 5. 依存の方針

新しい依存を足すときは、次の 3 つを揃える。**1 つでも欠けたら足していない。**

1. ADR を 1 本立てる
2. `deny.toml` の `allow` を実測（`cargo deny list`）で更新する
3. 下の表に 1 行足す

| 依存 | 用途 | 根拠 | 状態 |
| --- | --- | --- | --- |
| `regex-automata` 0.4.18（`default-features = false`・`alloc`・`syntax`・`meta`・`unicode`） | core: 正規表現の照合と置換（`FindQuery`・`no_std`）。速い道の `perf-literal-*` は入れない | [ADR 0031](adr/0031-regex-in-core.md)・GitHub Issue #171 | 導入済み。連れてくる依存は `regex-syntax` 0.8.11 だけ（`cargo tree -p nene-ledger-core -e normal` の実測）。ライセンスは `MIT OR Apache-2.0`（`make deny` 通過）。release の exe は 2.60 MB → 4.18 MB（+1.58 MB・gate-proofs 第 80 節） |
| `regex-syntax` 0.8.11（`default-features = false`・機能は `regex-automata` の `unicode` が入れる） | core: 式から当たりの頭の字の並びを取り出す（`hir::literal::Extractor`・正規表現の先探し・`no_std`） | [ADR 0031](adr/0031-regex-in-core.md) の追記（2026-11-08）・benchmarks 第 11 節の案 C・GitHub Issue #223 | 導入済み。`regex-automata` が連れてくるのと同じクレートと版を直接の依存に書いただけで、exe に入るクレートは増えない（`cargo tree -p nene-ledger-core -e normal` の実測・gate-proofs 第 89 節）。ライセンスは `MIT OR Apache-2.0`（`make deny` 通過） |
| `encoding_rs` 0.8.42（`default-features = false`・`alloc`） | scan: 表示のための復号（CP932・EUC-JP・UTF-16・Windows-1252）。store: 書き換えた値の符号化（CP932・EUC-JP・Windows-1252。UTF-16 は自前・[ADR 0025](adr/0025-save-by-replacement-and-windows-locks.md)・GitHub Issue #58） | [ADR 0015](adr/0015-encodings-with-encoding-rs.md)・Issue #18 | 導入済み。連れてくる依存は `cfg-if`・`core_detect`・`multiversion_no_op`（手続きマクロ・何もしない）・`scopeguard`・`simdutf8`（`cargo tree` の実測）。依存が 1 つで済む 0.8.35 ではなく 0.8.42 にしたのは、0.8.40 の 2 バイトの文字コードの復号の境目の直しと、panic のときに初期化されていない記憶が `String` に見える問題の直しを取るため。ライセンスは Apache-2.0 / MIT / BSD-3-Clause（`cargo deny list`） |
| `x11rb` 0.14.0（既定の機能なし） | ui-x11: X11 の窓・イベント・`PutImage`（純 Rust の接続） | [ADR 0016](adr/0016-ui-dependencies-x11rb-cairo-windows.md)・Issue #20 | 導入済み。連れてくる依存は `x11rb-protocol`・`gethostname`・`rustix`・`linux-raw-sys`・`bitflags`（`cargo tree` の実測） |
| `cairo-rs` 0.22.9（既定の機能なし・glib なし） | ui-x11: 画面外の面（`ImageSurface`）への描画と字の寸法 | [ADR 0016](adr/0016-ui-dependencies-x11rb-cairo-windows.md)・Issue #20 | 導入済み。実行時は `cairo-sys-rs`・`bitflags`・`libc` とシステムの `libcairo2`。組み立て時に `system-deps`（`pkg-config`・`toml` など）。`target-lexicon`（組み立て時だけ）のために `deny.toml` に `Apache-2.0 WITH LLVM-exception` を足した |
| `windows` 0.62.2（使う機能だけ）・`windows-numerics` 0.3.1 | ui-win32: Win32 の窓・Direct2D・DirectWrite・DWM・クリップボード。adapters-windows: レジストリ（OS の明暗）。Windows でだけ引く | [ADR 0016](adr/0016-ui-dependencies-x11rb-cairo-windows.md)・Issue #55 | 導入済み（`make lint-windows` の型と lint だけ・exe は作らない）。連れてくる依存は `windows-core`・`windows-implement`・`windows-interface`・`windows-link`・`windows-result`・`windows-strings`・`windows-collections`・`windows-future`・`windows-threading`、手続きマクロの組み立て時に `syn`・`quote`・`proc-macro2`・`unicode-ident`（このために `deny.toml` に `Unicode-3.0` を足した）。使う機能は各 Cargo.toml に理由を書く |

版は `Cargo.lock` で固定し、マニフェストに `*` や `=` を書かない。ゲートは lock が更新される状態を拒む（QLT-011）。

---

## 6. 採用しなかった検査と、その理由

**「有効にしなかった」ことも決定である。** 再提案するときは、ここに書かれた理由への反論から始めること。

| 検査 | 不採用の理由 |
| --- | --- |
| `clippy::restriction` 群の一括有効化 | 互いに矛盾する規則の在庫であって規則集ではない（`mod_module_files` と `self_named_module_files` は正反対）。群で入れると規則ではなく道具の機嫌に従うことになる |
| `clippy::nursery` 群の一括有効化 | 「まだ完成していない」と作者が宣言している群。必要なものは 1 つずつ名指しで入れる（`cognitive_complexity` はそうする） |
| `clippy::arithmetic_side_effects` | `a + 1` を全て `checked_add` に置き換えさせる。表の配置とバイト位置の計算でほぼ全行が対象になる。代わりに `overflow-checks = true`（RS-017） |
| `clippy::missing_docs_in_private_items` | 名前が役割を語っていれば（RS-013）重複であり、書かされた doc は更新されず嘘になる |
| `clippy::min_ident_chars` | `x` / `y` / `w` / `h` のような幾何で確立した短縮名と正面から衝突する |
| `clippy::pattern_type_mismatch` | match ergonomics を丸ごと否定する取引になる |
| waiver 台帳（先行事例の `WVR-NNNN`） | Rust では制度の大半をコンパイラが執行している。forbid 層は例外の申請窓口が存在せず、deny 層の `#[expect]` は理由が必須で、不要になれば失敗する。必要になったら（forbid を deny に降ろす判断をしたら）ADR で導入する |
| `rustfmt.toml` | 整形の選択肢を作る。既定のまま使うことが「一つの手段」である |
| Python の検査器 | 先行の C++ 事例は Python で書いたが、Rust のリポジトリに第 2 の言語を持ち込むと道具の版が 2 系統になる。検査器・Git 規約・証明を全て `xtask` に置く |

---

## 7. 規約に違反したくなったとき

1. **まず、規約が間違っている可能性を検討する。** 規約は実装より新しくない
2. 規約が正しいなら、設計を変える。抑制で通さない
3. 抑制が要るなら `#[expect(<lint>, reason = "<規則 ID>: <理由>")]` を最小スコープに書く。**forbid 層は抑制できない**（E0453）。抑制したくなった時点で設計の問題である
4. **規則そのものを緩めるなら ADR を書く**（QLT-010）。旧規則のどの部分が今も有効かを明記すること
