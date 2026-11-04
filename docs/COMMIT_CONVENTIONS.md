# コミットとリポジトリの規約 — NeNe Ledger

> Status: normative（規範）/ 2026-10-22 初版 / 2026-11-04 公開リポジトリへ移行（[ADR 0021](adr/0021-public-repository-on-github.md)）
> 出典: NeNeDocket `docs/COMMIT_CONVENTIONS.md`（GIT-001〜004・`commit-msg` フックと CI で機械強制）。GIT-005 と `perf` は本リポジトリで足した

変更の経路は一つである: Issue → 焦点を持つブランチ → 一貫したコミット → PR → 必須 check → squash merge → 同期した綺麗な `main`。

読み方（**active / planned / 不能 / 不採用**）は [ARCHITECTURE_CONSTITUTION.md](ARCHITECTURE_CONSTITUTION.md) 第 0 節と同じ。

---

### GIT-001 — すべての変更は 1 つの Issue から始まる

Issue は GitHub の Issue。問題と根拠・意図する結果・影響する規則 ID とモジュール・受け入れ条件・検証の計画・**やらないこと**を書く。
読み取りだけの調査は Issue 無しでよい。公開リポジトリの最初のコミット（Issue #1 を指す）だけが PR を通らない例外。

- 機械強制: **active**（PR テンプレート＋`make ci` が PR 本文の `Closes #N` を検査する。CI の実行証拠は PR #2 の実行 41832653816・[gate-proofs](quality/gate-proofs.md) 第 51 節）
- 不能: Issue の中身の妥当性

### GIT-002 — ブランチ名は一つの形

```text
<type>/<issue番号>-<short-kebab-summary>
```

例: `build/2-port-machinery`。`main` での直接開発・force push・既定ブランチの削除は禁止（最初のコミットを除く）。
クラウドセッションが harness から割り当てた枝名（`claude/…`）で作業するときは、PR の本文にその事実を書き、squash merge の件名を GIT-003 の形に揃える。

- 機械強制: **active**（ブランチ名の形は `make ci` が PR の head で検査する。`main` の保護（PR 必須・force push と削除の禁止）は ruleset 26530802・gate-proofs 第 51 節）

### GIT-003 — コミットメッセージは一つの形

Conventional Commits を、この正確な形で書く。

```text
<type>(<scope>): <日本語の説明> (#<issue番号>)
```

- `type` は `feat` / `fix` / `docs` / `refactor` / `test` / `build` / `ci` / `chore` / `perf` のいずれか（`perf` は速さの変更。測定の記録を伴う・QLT-014）
- `type`・`scope`（小文字・省略可）・`BREAKING CHANGE` 等の Conventional Commits の語は英語。**説明と本文は日本語**
- 件名は 100 文字以内。末尾に `(#N)` を必ず付ける
- 互換性を壊す公開契約の変更（設定ファイルの版を含む）は `:` の前に `!` を付け、フッタに `BREAKING CHANGE:` を書く
- 件名には「何を」ではなく**判断**を置く（例: `原本は保存の置き換えまで書かず、編集は差分として重ねる`）

- 機械強制: **active**（`.githooks/commit-msg` → `xtask commit-msg`・[gate-proofs](quality/gate-proofs.md) 第 6 節。CI は `make ci` が PR の全コミットと PR タイトルを同じ検証器で検査する。形の外の PR タイトルで CI が落ちた実行 41832778822・gate-proofs 第 51 節）
- 補足: squash merge の件名も同じ形。**マージ時に人が書き換える件名が最後の穴**なので、PR タイトルを同じ形に揃える。`gh pr merge --squash --subject` で件名を渡し、`(#Issue) (#PR)` の二重を作らない（nene-ruler 2026-10-20 の教訓）

### GIT-004 — PR が統合の境界

PR には目的・使った正典経路・規則 ID・振る舞いの変更・確認する退行・検証結果と記録の所在・再利用の根拠・抑制・残るリスク・`Closes #N` を書く
（[DEVELOPMENT_WORKFLOW.md](DEVELOPMENT_WORKFLOW.md) 第 7 節・雛形は `.github/pull_request_template.md`）。1 つの PR に 1 つの作業単位。マージは squash のみ。

実装中とレビュー中は draft。必要な差分検証と結果・再利用の根拠がそろったら Ready にする（QLT-001 / QLT-012）。
CI の必須 `check` は Git 規約・検証記録の欄・差分の空白を確認する。記録の妥当性はレビューし、CI 成功を製品テストの実行証拠とは扱わない。
文書だけの PR でも本文は全欄書く（nene-ruler で欄を欠いた PR が ruleset 未設定のまま merge された）。

- 機械強制: **active**（PR テンプレート＋`make ci` が欄を行の始まりで検査する。squash のみ・`check` の必須は ruleset 26530802・gate-proofs 第 51 節）

### GIT-005 — リポジトリには対外的に要るものだけを置く

管理するファイル・コミット・PR 本文には、このリポジトリを外から読む人に要るものだけを置く。
開発機に固有の情報（使ってはいけない道具・ホームディレクトリなどの絶対パス・社内の事情）は、git 管理外の `CLAUDE.local.md` と `.claude/settings.local.json` に置き、管理するファイルに写さない（`.gitignore` 済み）。

- 機械強制: **active**（`make ci` が管理するファイルと PR の範囲のコミット・PR 本文から、ホームディレクトリの絶対パス（`/home/<名前>/`・`/mnt/c/Users/`・`C:\Users\`）を拒む。「社内の事情かどうか」はレビュー事項。実測は [gate-proofs](quality/gate-proofs.md) 第 6 節）

---

## フックの導入

フックはリポジトリが所有する（`.githooks/`）。clone 直後に 1 回、次を実行する。

```bash
git config --local core.hooksPath .githooks
```

`pre-commit` は staged 差分の空白を、`commit-msg` はメッセージ（GIT-003 / GIT-005）を確認する。push / merge フックは置かない。
フックで全件検証や成功済みテストの再実行を強制しない。フック成功だけで検証済みとは扱わない。

🔴 フックはローカルでしか動かない。`--no-verify` で迂回できるので、**CI が PR の全コミットと PR タイトルを同じ検証器で検査する**ことで初めて active と書ける。
