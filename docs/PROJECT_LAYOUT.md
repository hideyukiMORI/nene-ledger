# モジュール構成と依存規則 — NeNe Ledger

> Status: normative（規範）/ 2026-10-22 初版
> workspace: `nene-ledger`（crate 名の接頭辞は `nene-ledger-`）。出典は NeNeDocket `docs/PROJECT_LAYOUT.md`

モジュールグラフはアーキテクチャの一部である。**モジュールの命名規約だけでは依存の境界にならない。**
Rust では層を**クレート**で表し、各クレートの `[dependencies]` が import 可能なものの全リストになる（ARC-002）。
承認された向きの表は `xtask` の CNF-013 が持ち、マニフェストと照合する。

🔴 クレートは具体的な責務とソースがある時だけ作る。**空の将来用モジュールは作らない。** Issue #20 の時点で在るのは `crates/core`・`crates/application`・`crates/scan`・`crates/ui-x11`・`crates/app` と `xtask` だけ。

---

## 1. 承認されたモジュール

```text
crates/app（nene-ledger・bin）
    合成ルート。ポートに実装を結び、UI を起動する。端末出力・引数・終了コードを持つ唯一の場所

crates/ui-win32（nene-ledger-ui-win32）
    Windows の画面。Win32 の窓に Direct2D / DirectWrite で部品を描く。IME（TSF か IMM32）。unsafe を書ける 3 クレートの 1 つ（ADR 0002）

crates/ui-x11（nene-ledger-ui-x11）
    Linux の画面。x11rb で窓を持ち、cairo の ImageSurface に部品を描いて転送する

crates/adapters-windows（nene-ledger-adapters-windows）
    Windows のアダプタ。時計・クリップボード・設定の置き場・ごみ箱・印刷・ファイルのロックの観測。unsafe を書ける 3 クレートの 1 つ（ADR 0002）

crates/adapters-x11（nene-ledger-adapters-x11）
    Linux のアダプタ。同じポートを X11 と POSIX で実装する

crates/store（nene-ledger-store）
    書き出し。保存（一時ファイル → 置き換え）・方言を変えた書き出し・文字コードの符号化・設定ファイルの形と版

crates/scan（nene-ledger-scan）
    読み取り。原本の写像（mmap）か読み出し・SIMD による区切りと改行の探索・並列の行の索引・並べ替えと集計の実行（作業のスレッド）。
    unsafe を書ける 3 クレートの 1 つ（ADR 0006）。規則の正本は core にあり、ここはそれを速く実行するだけ

    アダプタ 2 つと scan と store だけが、現在時刻・環境・ファイル・スレッドを読める製品モジュール（ARC-007 / CNF-012）

crates/application（nene-ledger-application）
    ポートの宣言、開いたファイルの状態の所有者（方言・差分・変換の規則・表の見え）、Undo の束、編集状態、一括処理の状態、表示フレームの生成、結果型。#![no_std]

crates/core（nene-ledger-core）
    値・不変条件・閉じた選択肢・方言・セルの切り出しの規則（遅いが正しい参照実装）・差分と変換の規則・比較と集計の規則・部品の核（配置・当たり判定・フォーカス・編集状態・描画命令）・整形。#![no_std]（外部: regex-automata（no_std・正規表現の照合・ADR 0031・GitHub Issue #171）と、それが連れてくる regex-syntax（式から先探しの字の並びを取り出す・GitHub Issue #223））

xtask
    規約検査・Git 規約・ゲート発火の証明。成果物ではない（publish = false）。依存 0
```

🔴 上の一覧は**承認**であって**存在**ではない。各クレートは、責務とソースができる Issue で初めて作る。

---

## 2. 許可された依存グラフ

```text
core
    -> なし（alloc のみ）

application
    -> core

scan
    -> application, core（外部: encoding_rs（ADR 0015・Issue #18）。写像と SIMD は ADR 0006 の導入の Issue で決める）

store
    -> application, core（外部: encoding_rs（ADR 0025・GitHub Issue #58）。scan には依存しない）

adapters-x11
    -> application, core（外部: 予定。x11rb）

adapters-windows
    -> application, core（外部: 予定。windows。Windows でだけ）

ui-x11
    -> application, core（外部: 予定。x11rb, cairo-rs）

ui-win32
    -> application, core（外部: 予定。windows。Windows でだけ）

app
    -> すべて（明示的な合成のためだけに）。X11 のクレートは Windows 以外、Windows のクレートは Windows でだけ（target の節）
```

ここに無い依存はすべて禁止である。とくに:

- core -> 外部 crate: **禁止**（CNF-013 が拒む）
- core -> application: 禁止
- application -> scan / store / アダプタ / UI: 禁止（ファイルの読み方も形も application が知らないこと）
- scan -> store、store -> scan: 禁止（読みと書きを混ぜない。保存が原本を読むときは application のポート越し）
- ui -> scan / store / アダプタ: 禁止
- 何か -> 合成ルート: 禁止

---

## 3. 各モジュールの責務

### `core`

意味の正本を持つ。方言・行と列とセルの番号・セルの切り出しの規則（参照実装）・差分・変換の規則・比較の規則・集計の規則・部品の配置と当たり判定・表示文字列の整形。
UI 状態・ファイル・スレッド・現在時刻を持たない。`#![no_std]`。

### `application`

振る舞いの調整を持つ。ポート（`trait`）・開いたファイルの状態の所有者・Undo の束・編集状態・一括処理の状態機械・表示フレームの生成・結果型。
ファイル・時刻・スレッド・OS を知らない。`#![no_std]`。

### `scan`

原本を速く読む。写像か読み出し、SIMD の探索、並列の索引、並べ替えと集計の実行。
**結果は core の参照実装と同じでなければならない**（QLT-008）。`unsafe` を書ける（ADR 0006）。決定性の禁止を適用しない区画の 1 つ（ARC-007）。

### `store`

書き出しの**形**と**手順**。保存の置き換え（Windows は `ReplaceFileW` と再試行、Linux は `rename`）・方言を変えた書き出し・設定ファイルの版（ARC-009）。
決定性の禁止を適用しない区画の 1 つ（ARC-007）。

### `adapters-x11` / `adapters-windows`

時計・クリップボード・画面・IME・ごみ箱・印刷・設定の置き場のポートを OS の API で実装する。取得値をその場で core の型へ変換する（ARC-008）。

### `ui-x11` / `ui-win32`

application が作った値を描き、操作を意図として渡す（ARC-011）。部品の**配置と当たり判定は core** が物理画素で計算し、UI はそれを描くだけ。可変性の隔離区画（ARC-005）。

### `app`

依存を結ぶ。端末出力・`std::env::args`・終了コードを扱ってよい唯一の場所（ARC-006）。

---

## 4. ソースの置き場

| 種類 | 置き場 |
| --- | --- |
| production | `crates/core` / `crates/application` / `crates/scan` / `crates/store` / `crates/adapters-*` / `crates/ui-*` / `crates/app` |
| 単体テスト | 各クレートの `#[cfg(test)]`（ファイル末尾）と `tests/`。OS 資源と表示は使わない。scan と store のテストは一時ディレクトリを使ってよい |
| 性能の測定 | Issue で置き場を決める（比較相手と同じファイルで測る・ADR 0007） |
| 検査器と証明 | `xtask/`（`cargo test -p xtask` が正例・反例） |
| 作業単位（Issue） | GitHub の Issue。統合は PR（ADR 0021） |
| 検査設定 | `Cargo.toml` の `[workspace.lints]` / `clippy.toml` / `deny.toml` / `rust-toolchain.toml` / `Makefile` |
| 見本のファイル | `crates/app/fixtures/`（合成ルートのテストと `make screenshot` が開く。テストから書けないので置いておく・CNF-012） |
| 生成物 | `target/` / `out/` / `dist/` |

表示を伴う確認は環境検証であって単体テストではない（QLT-013）。記録は `docs/quality/` に置く。
