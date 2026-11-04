# ADR 0021 — 公開リポジトリを GitHub に置く。Issue は GitHub の Issue、PR が統合の境界、CI は `make ci` を呼ぶだけ、`main` は ruleset で守る

- 状態: 受理
- 日付: 2026-11-04
- Issue: #1
- 置換: [ADR 0004](0004-local-only-repository.md)
- 影響する規則: GIT-001 / GIT-002 / GIT-003 / GIT-004 / GIT-005 / QLT-005 / QLT-010 / QLT-012

## 文脈

ADR 0004（2026-10-22）は「リモートを置かない」と決め、GitHub の Issue・PR・CI・ruleset の役割をリポジトリの中のファイル（`docs/issues/`）と手元の手順（統合記録・`make ci`・`git merge --squash`）に置き換えた。
その形で 68 の Issue を進め、M1 の大半（core / scan / application / ui-x11 / adapters-x11 / ui-win32 / adapters-windows / app の 8 クレート）ができた。

ADR 0004 が正直に書いた穴は、そのまま残っていた:

- フックの迂回（`--no-verify`）を止める層が無い
- バックアップが無い（WSL の仮想ディスクが壊れれば全部失う）
- `main` への直接のコミット・squash 以外の統合・force push を拒む層が無い

施主は 2026-11-04 に、制作の過程を描いた記録（Issue のファイル・経過の要約）を外し、規約と現状のコードから**公開リポジトリ**で開発を続けると決めた。

## 決定

**`hideyukiMORI/nene-ledger`（PUBLIC）を置き、GitHub の Issue・PR・CI・ruleset を規約の置き場に戻す。** 出典の NeNeDocket と同じ形。

1. **Issue は GitHub の Issue。** 欄は ADR 0004 のファイルと同じ（問題と根拠・意図する結果・規則 ID・受け入れ条件・検証の計画・やらないこと）。コミットの `(#N)` は GitHub の番号
2. **PR が統合の境界。** 本文の欄は `.github/pull_request_template.md`（統合記録と同じ欄）。draft で出し、検証記録がそろったら Ready にする
3. **CI は `make ci` を 1 行呼ぶだけ**（`.github/workflows/check.yml`・`ready_for_review` と非 draft の `synchronize` / `edited` / `reopened`）。判定は `xtask validate-git`（PR のメタデータ ＋ `base..HEAD` の全コミット ＋ 管理しているファイルの絶対パス）と `git diff --check`。手元の `make ci` も同じ 1 行
4. **`main` は ruleset で守る。** PR 必須・`check` 必須・squash のみ・force push と削除の禁止。リポジトリの設定は squash のみ・マージで head の枝を消す
5. **ローカルの履歴は持ち込まない。** 公開リポジトリの最初のコミットは現状のコードと規範文書だけ（Issue #1）。過程は残した記録（gate-proofs・benchmarks・FEATURES）の `Issue #N` にだけ残り、それらは公開前の番号である旨を前書きで示す
6. GIT-005（開発機に固有の絶対パスの禁止）と `perf` の type は残す。PR 本文も GIT-005 の検査を通る

## 強制

- **active**（Issue #1 の PR #2 で実測・[gate-proofs](../quality/gate-proofs.md) 第 51 節）: `make ci` が PR 本文の `Closes #N`・欄・ブランチ名・全コミットと PR タイトルの形・絶対パスを見る（CI の実行 41832653816 が success、形の外のタイトルで 41832778822 が failure）。ruleset 26530802 が PR 必須・`check` 必須・squash のみ・force push と削除の禁止を課す
- **不能**: Issue と記録の中身の妥当性（レビュー事項）

## 結果

得られるもの:

- フックを迂回したコミットも PR で CI が同じ検証器で見直す
- `main` への直接のコミット・force push・squash 以外の統合を ruleset が拒む
- リモートがバックアップを兼ねる
- Issue と PR の画面で差分とレビューを追える

失うもの:

- 作業の内容が手元の外に出る（施主の決定。ライセンスは ADR 0009 のまま）
- 公開前の 68 Issue の本文は公開リポジトリから引けない。記録に残る `Issue #N`（#1〜#68）は旧番号で、GitHub の番号とは別物

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| ローカルの履歴ごと push する | 過程の記録を外すという施主の決定に反する。Issue のファイルへの参照（`docs/issues/`）が切れた履歴を公開しても追えない |
| private リポジトリにする | 施主が公開を決めた。NeNeDocket と同じく公開で進める |
| `docs/issues/` の形を残したまま GitHub にも置く | Issue の置き場が 2 つになる（ARC-001 の「一つのことに一つの経路」に反する） |
| 統合記録を PR 本文と併用する | 同じ欄が 2 か所に増える。PR 本文の 1 か所にする |
