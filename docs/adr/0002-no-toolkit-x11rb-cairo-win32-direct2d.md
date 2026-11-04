# ADR 0002 — X11 は x11rb ＋ cairo、Windows は Win32 ＋ Direct2D。UI ツールキットを使わず、部品は自前で描く。Windows を主な対象とする

- 状態: 受理
- 日付: 2026-10-22
- Issue: #1
- 影響する規則: ARC-003 / ARC-005 / ARC-013 / RS-010 / CNF-002 / CNF-012 / CNF-013 / CNF-014 / QLT-013

## 文脈

施主の条件は「Windows と Linux で動く」「巨大なファイルでも速い」「全部自前で作る」。
施主は当初「Windows だけに絞ってネイティブにするか、X11 で作って Windows の表示部分を NeNeDocket のように作るか」を問い、後者に決めた（2026-10-22）。

NeNeDocket は同じ条件で、Linux は `x11rb` ＋ `cairo-rs`（`ImageSurface` に描いて転送）、Windows は `windows` クレートで Win32 の窓に Direct2D / DirectWrite で描く形を作った。
**しかし NeNeDocket の Windows 版は組み立てただけで、長く Windows で一度も動いていなかった。** Linux で開発していると Windows が後回しになる。

CSV エディターの利用者の大半は Windows（Excel の利用者）である。Linux の利用者はコマンドラインの道具（VisiData・qsv）で間に合わせることが多い。

## 決定

**UI ツールキットを使わない。Linux は `x11rb` ＋ `cairo-rs`、Windows は `windows`（Win32 ＋ Direct2D ＋ DirectWrite）だけで、すべての部品を自前で描く。そのうえで Windows を主な対象とする。**

1. クレートは NeNeDocket と同じ分け方: `ui-x11` / `ui-win32` が画面、`adapters-x11` / `adapters-windows` が OS 資源。core と application は `#![no_std]`
2. 部品の配置・当たり判定・表示文字列は core が物理画素で計算する。UI はそれを描き、クリックと鍵を意図に変える。**UI に業務の判断を置かない**（ARC-011）
3. 描画は画面外の面に完成させてから窓へ転送する。画素の列は OS 無しで単体テストする
4. `unsafe` は Windows の 2 クレートにだけ `deny` で許し、Win32 を呼ぶ小さな関数ごとに `#[expect(unsafe_code, reason = "RS-010: …")]` と `// SAFETY:` を置く。lint の表は workspace の写しで、CNF-014 が同じであることを見る（もう 1 区画は ADR 0006）
5. **Windows を後回しにしない。** 最初の画面の Issue で Windows の窓も出し、Windows の実機で確かめるまで「動く」と書かない（QLT-013）。IME・画面ごとの倍率（Per-Monitor v2）・Excel とのクリップボード・Shift_JIS・ファイルのロックは、あとから足すと設計に響くので早い Issue で扱う
6. 文字入力（IME）の方式は別の ADR で決める。NeNeDocket ADR 0020（Windows は IMM32 で確定前の文字列を欄に描く）を出発点にし、TSF と比べる

## 強制

- **planned**: CNF-013（ui / adapters の外部 crate）・CNF-002（`XlibSurface` / `XCBSurface`）・CNF-014（unsafe を許すクレートの lint の表）・`deny.toml` の `allow`。Issue #2 と最初の画面の Issue で実ファイルの発火を記録してから active にする
- **不能**: 「UI に業務の判断を置いていないか」はレビュー事項（ARC-011）。「Windows を後回しにしていないか」もレビュー事項

## 結果

得られるもの:

- 依存が少なく、配る物が 1 つの exe / 1 つのバイナリになる（NeNeDocket の実績: Windows の exe 1.9 MB）
- 部品の見た目と振る舞いが 2 OS で同じになる
- 表の配置と当たり判定が純関数なので、1 億行の表の配置を OS 無しで測れる（FR-004）
- NeNeDocket の表示部分・IME・ロック対策の資産を写せる

失うもの・残る穴:

- 🔴 **IME は重い。** 日本語の利用者には、セルの中に確定前の文字列が出ないと使ってもらえない
- 🔴 **アクセシビリティは無い。** スクリーンリーダは自前の部品を読めない
- 🔴 **文字の整形は OS 任せ。** 絵文字・結合文字・右から左の文字は正しく出ないかもしれない。日本語と英数字だけを「動く」と書く
- 🔴 **Windows の実機で動かして確かめる手順が、まだ決まっていない。** どの機械でどう確かめるかは施主が決める。決まるまで「動く」と書かない（QLT-013）
- Wayland ネイティブと macOS は対象外（SPECIFICATION 第 3 節）

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| Windows だけに絞り、ネイティブで作る | 施主が 2 OS を選んだ。中核は OS に依らないので、表示部分を 2 つ持つ費用は小さい（NeNeDocket の実績） |
| X11 を先に作り、Windows をあとから足す | NeNeDocket で Windows が長く動かなかった。利用者の大半は Windows |
| Electron / Tauri | Chromium か WebView を抱え、軽くも速くもならない |
| GTK / Qt | ツールキット。依存が数十に増え、Windows での配布が重い |
| egui / iced / slint 等の Rust のツールキット | 「全部自前」に反する。描画のバックエンドが依存の木を広げる |
| winit ＋ 自前描画 | 窓・IME・倍率を任せられるが、Wayland / macOS / Web を抱えて依存が多い。Windows の IME の細かい挙動は自前のほうが調整できる。NeNeDocket の資産もある |
| Windows も cairo で描く | Windows に cairo の DLL を配ることになる。Direct2D は OS にある |

## 関連

- NeNeDocket ADR 0002・ADR 0009（Windows 版）・ADR 0020（IME）
