# ADR 0001 — 厳格さは機械で強制する

- 状態: 受理
- 日付: 2026-10-22
- Issue: #1
- 影響する規則: すべて

## 文脈

本リポジトリの規約は、AYANE 厳格規約ポリシーの 6 回目の適用である。先行は NeNe Loupe（C++23 / MSVC）、
NeNe Folio（C23 / clang-cl）、NeNe Nib（C++23 / clang-cl）、Rust では xi-tools・nene-ruler・NeNeDocket。共通する中核は 1 つに要約できる。

> **一つのことを実現する方法を 1 つに固定し、そのことを人の記憶ではなく機械に守らせる。**

規約は、破ったときに何も起きなければ、時間とともに必ず腐る。

### 施主の条件と、言語の選定

施主が示した条件は「Windows と Linux（X11）で動く」「巨大なファイルでも速さを約束する」「CSV / TSV のエディター」「Modern CSV の有料機能に当たるものも速く安定して載せる」「全部自前で作る」（2026-10-22）。
言語は次の理由で **Rust** に決めた（施主 2026-10-22・「全部その方向でいい」）。

- 速さは C / C++ と同じ水準で、CSV の分野の速い道具（xsv・qsv・Polars）の多くが Rust である
- 巨大で中身の分からないファイルを扱う。メモリを壊す種類の欠陥を言語が塞ぐことは、そのまま「データを失わない」に効く
- 裏で動く一括処理（並べ替え・集計）のデータ競合を、コンパイラが拒む
- NeNeDocket が同じポリシーのもとで、Win32 ＋ Direct2D と x11rb ＋ cairo の自前描画・Shift_JIS の CSV の読み取り・Windows のロック対策・規約の機械を Rust で揃えている

NeNeDocket が rustc 1.98.0 で測った出力（NeNeDocket の `docs/quality/gate-proofs.md`）:

```text
P-05  match の variant 漏れ        → error[E0004]: non-exhaustive patterns
P-15  非公開 newtype の直接構築     → error[E0603]: tuple struct constructor `Id` is private
P-16  未初期化の束縛（Go のゼロ値）  → error[E0381]: used binding `z` isn't initialized
P-17  構造体の部分構築              → error[E0063]: missing field `b`
P-01  forbid した lint を #[expect] → error[E0453]: incompatible with previous forbid
P-02  発火しない #[expect]          → error: this lint expectation is unfulfilled
P-23  クレートの循環                → error: cyclic package dependency
```

**そして、いま実装が空である。** 規則を後から被せる場合に必要になる baseline が、今は要らない。違反ゼロから始められる時点は今しかない。

### NeNe Ledger に固有の事情

NeNe Ledger は**利用者のデータそのもの**を扱う。開いて保存しただけで値が変わる（Excel が先頭の `0` を落とす類い）ことは、この製品が存在する理由の否定である（FR-003）。
同じバイト列から同じセルが出ること・並列に読んでも同じ結果が出ること（ARC-007）は、美点ではなく正しさそのものである。

一方で、速さのために `unsafe`（写像と SIMD）が要る。NeNeDocket は `unsafe` を Windows の 2 クレートに閉じたが、本リポジトリはもう 1 区画を要する。
**例外の区画を増やすこと自体を ADR で決め、機械で固定する**（[ADR 0006](0006-confined-unsafe-for-scan.md)）。

## 決定

**すべての規範規則に「機械強制の状態」を持たせ、その状態を文書の側で機械検査する。**

1. 規則の正本を `docs/ARCHITECTURE_CONSTITUTION.md` / `docs/CODING_RULES.md` / `docs/QUALITY_GATES.md` / `docs/COMMIT_CONVENTIONS.md` に置く。
   すべての規則に ID（ARC / RS / QLT / CNF / GIT）を与え、状態を **active / planned / 不能 / 不採用** で明示する。**planned を active と書かない**
2. 🔴 **抑制を二段構えにする。** `forbid` 層は例外が取れない（E0453）。`deny` 層は `#[expect(lint, reason = "<規則 ID>: …")]` だけで抑制でき、
   発火しなくなれば `unfulfilled_lint_expectations` が落とす。`reason` の規則 ID が規範文書に実在することは CNF-006 が見る
3. `make` の個別 target と `xtask` を判定の唯一の置き場とする。リモートも CI も無いので、統合の前の `make ci` が必須の検査になる（[ADR 0004](0004-local-only-repository.md)）
4. baseline を持たない。クレート単位の `#![allow]` は `clippy::allow_attributes` が forbid なので書けない。設定側の無力化は CNF-011 が拒む
5. ゲートを弱める変更は ADR を要する（QLT-010）
6. 道具の版は `rust-toolchain.toml` だけが決める。2 か所に書かない
7. **ゲートは意図的な違反で発火することを証明してから active と書く。** `make prove` が本物の道具に違反を食わせ、記録を `docs/quality/gate-proofs.md` に残す
8. 規約検査（`xtask`）は依存ゼロで書く。Python を持ち込まない（Rust 1 言語）
9. **文書を先に置き、機械を次に置く。** Issue #1 は文書だけで、すべての規則を planned と書く。Issue #2 で NeNeDocket の機械（lint の表・`clippy.toml`・`deny.toml`・`xtask`・`Makefile`・フック）を移植し、このリポジトリで発火を測り直してから active にする

## 強制

- **planned**: すべて。Issue #2 で `xtask` を置き、`cargo test -p xtask` と `make conformance` と `make prove` で実測してから active にする
- **不能**: 「`planned` を勝手に `active` と書き換えないこと」そのもの。マトリクスの行と実装の対応は、証明行の有無までしか機械は見ない

## 結果

得られるもの:

- 規約が自分について嘘をつく経路が塞がる
- 新しい貢献者（人でも AI でも）が「何がいま守られているか」を 1 か所で読める
- waiver 台帳という人手の制度を持たずに済む（Rust の forbid / expect が代わりに執行する）
- NeNeDocket の `xtask` と設定を移植できるので、Issue #2 の大半は写しと再測定で済む

失うもの:

- 文書のメンテナンスコスト。規則を足すたびに 3 か所（本文・マトリクス・実装）が同期を要求される
- 書き味の制約: `as` 禁止・数値リテラルの型注釈・`v[i]` 禁止は、バイト列を走査する scan で最初に摩擦になる。速さのための抜け道を作らず、`get` と反復子で書いて測る。足りなければ ADR
- `forbid` は後から緩めにくい。降格には ADR が要る（それが狙いだが、コストである）
- **Issue #1 と #2 の間、規則は紙の上にしか無い。** その間に書かれたコードは無いので、実害は無い

**正直に記録しておくこと:**

- 🔴 本リポジトリでは**まだ何も測っていない**。この端末には Rust の道具がまだ入っていない（2026-10-22）。上の実出力は NeNeDocket のものである
- 🔴 リモートも CI も無いので、フックを迂回したコミットを止める層が無い。`make ci` が `main` の履歴を検査することだけが後から見つける手段になる（ADR 0004）
- 🔴 「不正な状態を表現不能にする」は Rust でも完全ではない。`#[derive(Default)]` でゼロ値を作れる（CNF-001 で塞ぐ）。同じモジュールに同居する型はファクトリを迂回できる（CNF-005 は planned）
- 🔴 `unsafe` を許す区画が 3 つになる。その区画の中の正しさは、参照実装との一致のテスト（QLT-008）でしか見られない
- **本 ADR の厳格さは、実装がまだ空である今だからこそ無傷で導入できる。** 緑であることは、規則が良いことの証明ではない。
  実装で規則が邪魔になったとき、緩めるのではなく **ADR で判断を残すこと**が本 ADR の眼目である

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| 散文の規約だけを置く | 守られているかを確かめる手段が無く、時間とともに必ず腐る。**本 ADR の出発点そのもの** |
| 既製 lint の既定セットだけで済ませる | 「NeNe Ledger として守るべきこと」（原本を書き換えない・速い経路が参照実装と一致する）は見られない |
| 実装が入ってから導入する | 既存違反を凍結する baseline が必要になる。今なら違反ゼロで始められる |
| 文書と機械を同じ Issue で入れる | この端末に Rust の道具が無い。機械を置いても測れないので active と書けない。文書だけなら全部 planned で嘘をつかない |
| NeNeDocket で active だった規則をそのまま active と書く | 道具の版も workspace も無いこのリポジトリで発火することを、まだ誰も見ていない。**planned を active と書かない**の最初の適用例 |
| Python で検査器を書く | Rust のリポジトリに第 2 の言語を持ち込むと道具の版が 2 系統になる。`xtask` に統一する |
| 規約を `CLAUDE.md` にだけ書く | 人間と AI の遵守に依存する。先行事例で実証済みの失敗 |
| 言語を C++23 にする | 写像と SIMD の自由度は高いが、安全性を規約と人の注意で守ることになり、Rust でコンパイラが守っている部分を失う。検査器も Python になる |
| 言語を C#（.NET）にする | Windows の画面は作りやすいが、巨大なデータでのメモリ回収（GC）の停止と、X11 の画面が弱い |
| 言語を Zig にする | 安定版が無く、道具の版を固定する QLT-011 と相性が悪い。ポリシーの実測が無い |

## 関連

- 先行: NeNeDocket ADR 0001 / 0005、nene-ruler ADR 0001
- ポリシー: 施主の private ワークスペース（本リポジトリには置かない）
