# ADR 0016 — 画面の依存は `x11rb`・`cairo-rs`・`windows` の 3 つだけ。入れるのは各クレートを作る Issue で、版は NeNeDocket と同じ所から測り直す

- 状態: 受理
- 日付: 2026-10-22
- Issue: #8
- 影響する規則: ARC-013 / RS-010 / CNF-002 / CNF-013 / CNF-014 / QLT-013。ADR 0002 / 0013 の実装の依存

## 文脈

ADR 0002 は X11 を `x11rb` ＋ `cairo-rs`、Windows を `windows`（Win32 ＋ Direct2D ＋ DirectWrite）で自前で描くと決めた。ADR 0013 は自前のタイトルバーと枠を決めた。NeNeDocket は `x11rb` 0.14・`cairo-rs` 0.22・`windows` で同じ形を作った（NeNeDocket CODING_RULES 第 5 節）。
Issue #7 で、当面 Windows の exe を組み立てないことになった。

## 決定

**画面の外部クレートは `x11rb`（X11 のプロトコル・純 Rust）・`cairo-rs`（`ImageSurface` への描画）・`windows`（Win32・Direct2D・DirectWrite・DWM）の 3 つだけにする。各クレートを作る Issue で 1 つずつ入れ、`deny.toml` の `allow`・CNF-013 の許可表・CODING_RULES 第 5 節を実測で更新する。**

1. `x11rb`（`default-features = false`・要る拡張だけ）: ui-x11 と adapters-x11。XDND・XIM・`_NET_WM_MOVERESIZE` も x11rb の要求で書く
2. `cairo-rs`（`default-features = false`）: ui-x11 だけ。`XlibSurface` / `XCBSurface` は `unsafe` が要るので使わない（CNF-002）
3. `windows`: ui-win32 と adapters-windows。使う機能（features）は最小にし、足すたびに理由を残す。`unsafe` は小さな関数ごとに `#[expect]` と `// SAFETY:`（ADR 0002）
4. **Windows のコードは、当面 `make lint-windows`（`x86_64-pc-windows-gnu` の clippy・exe を作らない）で型と lint だけを見る**（Issue #7）。実機の確認は経路が決まってからまとめて行い、それまで「Windows で動く」と書かない（QLT-013）
5. 版は導入の時点の最新から選び、`Cargo.lock` で固定する。NeNeDocket と版をそろえる必要は無いが、重複する版（`multiple-versions = "deny"`）が出たら導入の Issue で解く

## 強制

- **active（既存）**: CNF-013（表に無い外部クレートを中核・scan・store で拒む。ui と adapters は ADR と `deny.toml`）・CNF-002・CNF-014・cargo-deny
- **planned**: 画面の実ファイルでの発火（最初の ui の Issue）

## 結果

- 得るもの: 依存が 3 つ（＋ 文字コードの 1 つ・ADR 0015）で止まる
- 失うもの: 文字の整形（結合文字・右から左）は OS 任せ（ADR 0002）
- 残る穴: `cairo-rs` は Linux で `libcairo2` を要する。配布の Issue で扱う

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| `winit` / `softbuffer` などで窓を抽象する | ADR 0002 で却下（依存の木・自前の窓の細部を持てない） |
| `windows-sys`（生の束縛）だけを使う | COM（Direct2D・DirectWrite・DWM）の `Drop` による解放が無く、`unsafe` が増える |
| 依存を今まとめて入れる | 使わない依存を先回りで入れない（`--deny license-not-encountered` で落ちる・ARC-013） |
