# 開発ワークフロー — NeNe Ledger

> Status: normative（規範）/ 2026-10-22 初版 / 2026-11-04 公開リポジトリへ移行（[ADR 0021](adr/0021-public-repository-on-github.md)）。出典は NeNeDocket `docs/DEVELOPMENT_WORKFLOW.md`

ワークフローは制約系の一部である。**追跡可能な手順の外で作られた技術的に正しいパッチは、未完成である。**

---

## 1. 作業の単位

production コード・ビルド・依存・方針・利用者向けドキュメントの変更は、すべて 1 つの焦点を持つ Issue（GitHub）から始める。Issue には次を書く。

- 問題と根拠
- 意図する結果
- 影響する規則 ID とモジュール
- 受け入れ条件
- 検証の計画
- **やらないこと**

読み取りだけで、リポジトリを変更しない調査は Issue 無しで行ってよい。

---

## 2. 標準の流れ

1. 焦点を持つ Issue を 1 つ作る／選ぶ
2. 正本のドキュメントと関係する ADR を読む
3. **編集の前に**正典の実装経路を特定する
4. 最新の `main` からブランチを切る（`<type>/<N>-<kebab>`）
5. 設計そのものを変えるなら、**先に**方針文書か ADR を変える
6. 最小で完結する変更を実装する
7. テストを足す／更新する
8. 差分の挙動と直接の依存先・呼び出し元から、起こり得る退行を検出する最小限の検証を選んで回す（QLT-001）
9. PR を **draft** で出す
10. 影響した規則 ID を 1 つずつ自己レビューする
11. 検証結果・記録の所在・再利用の根拠を PR に書いて Ready にする。CI の必須 check（`make ci`）は Git 規約・記録の欄・絶対パス・空白だけを見る
12. 必須 check が通ってから squash merge する（`gh pr merge --squash --subject '<GIT-003 の件名>'`）
13. ローカル枝と作業木を消す（remote 枝はマージで自動的に消える）

---

## 3. 検証の選び方

個別の target を使う。

```bash
make fmt-check     # 整形
make lint          # clippy（全 target・--locked）
make lint-windows  # Windows のクレート（Linux では cfg で外れて見えない）
make test          # cargo test（xtask の正例・反例を含む）
make conformance   # CNF-0xx
make deny          # 依存の許可制
make doc-check     # rustdoc
make build         # release
make prove         # ゲート発火の証明（ゲートを触ったときだけ）
```

全件は、限定した検証では影響を覆えない具体的な理由がある場合だけ、理由を添えて回す。

```bash
make check REASON='共通の lint 集合を変えたので全クレートに影響する'
```

実行していない結果を書かない。本件が原因の失敗は直し、無関係な既存失敗は根拠とともに別 Issue へ記録して本件を続ける。
成功するまでの再試行や無関係な修正・全件再実行は禁止。正本は QLT-001 / QLT-012 と [ADR 0003](adr/0003-diff-scoped-verification-and-result-reuse.md)。

---

## 4. 決定の記録（ADR）

### ADR が要るもの

- 憲章規則・依存方向の変更
- モジュール境界・状態の所有者・隔離区画の変更
- 原本と差分の扱い・保存の手順（ADR 0005 の改訂）
- 設定ファイルの形式と版
- 合成／DI の方針
- 並行性・スレッドモデル・一括処理の土台
- lint の集合・閾値・`forbid` からの降格・免除の追加（QLT-010）
- 対応プラットフォームの変更
- 実行時依存の追加（ARC-013）
- `unsafe` を要する経路（RS-010）
- 文字入力（IME）の方式・印刷の方式・外部の変更の検出の方式

### ADR が要らないもの

- 承認済みの境界の内側に閉じた実装の詳細
- 文書化された振る舞いを回復するだけの不具合修正
- 方針を変えないテストとドキュメント
- **ゲートを強める変更**（弱める変更だけが ADR 事項）

---

## 5. 規則を変える手順

MUST / MUST NOT を変えるには次がそろう必要がある。

1. 矛盾と根拠を書いた Issue
2. 受理された ADR
3. 正本ドキュメントの更新
4. 強制（ゲート）の更新と `make prove` の再実行
5. コードの移行
6. **旧経路が拒否されることを示す回帰テスト**

`V2` / `Legacy` / `New` / `Alternative` という名の並行実装を作らない。

---

## 6. 依存の方針

新しい実行時依存を受け入れるのは、次がすべて満たされるときだけである。

- その機能が具体的にいま必要である
- 自前で書くほうが危険か、保守しにくい（文字コードの表はこれに当たる。自前で書かない）
- ライセンスと保守状況が受け入れられる（`make deny`）
- 正しい境界の裏に隔離できる（中核には入らない）
- 版が固定でき再現可能である（QLT-011）
- ADR がある

**便利だから、は理由にならない。速いから、も計測が無ければ理由にならない（QLT-014）。**

---

## 7. PR に書くこと

雛形は `.github/pull_request_template.md`。欄は行の始まりに置く（`make ci` が行の始まりで見る）。

```text
Closes #N

目的:
使った正典経路:
規則 ID:
振る舞い・スキーマの変更:
確認する退行:
検証（実行したコマンドと結果・記録の所在）:
再利用（元の記録と不変性の根拠、または初回検証）:
抑制（#[expect]）: none | 一覧
残るリスク:
```

レビューは次を拒否する。

- 既存の意味に対する第 2 の経路
- 所有者の書かれていない状態
- 弱められた／飛ばされたゲート
- `#[expect]` の理由に規則 ID が無いもの
- アダプタ固有の業務ロジック・UI に置かれた並べ替えや集計や状態遷移
- 「将来使うかもしれない」公開 API
- 機能変更に紛れ込んだ無関係な整理
- 原本をその場で書き換える経路（ADR 0005）

---

## 8. ドキュメントの方針

重要な決定を、チャット・コミットメッセージ・AI との会話ログだけに残さない。

- 長生きする不変条件 → 正本ドキュメント
- 判断とトレードオフ → ADR
- いまのタスク状態 → Issue / PR（要約は private の `nene-origin/internal-docs/ledger/todo/current.md`）
- コードのコメント → その場の非自明な理由（方針そのものは書かない）

設定ファイルとドキュメントが食い違ったら、**ドキュメントが決定の記録**である。ただし両方がそろうまで merge は止める。
**事故が起きたら、直すだけでなく再発を機械で塞ぐ規則を足し、Issue 番号と実測を規則本文に残す。**

---

## 9. 開発環境

### 開発機の用意（Windows 11 ＋ WSL2 の Ubuntu）

開発は WSL2 の Linux で行い、Windows の exe も Linux から組み立てる。クラウドのコンテナ（Claude Code on the web）では `.claude/hooks/session-start.sh` が同じ道具を揃える。

```bash
# rustup は apt で入れる。版は rust-toolchain.toml が正で、ここに書く数字はそれを写しただけ
sudo apt-get install -y rustup
rustup default 1.98.0
rustup component add clippy rustfmt llvm-tools-preview
rustup target add x86_64-pc-windows-gnu
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
rustup component add rust-analyzer   # LSP（エディターと Claude Code の rust-analyzer-lsp が使う）
cargo install cargo-deny --locked
cargo install cargo-llvm-cov --locked
sudo apt-get install -y --no-install-recommends libcairo2-dev pkg-config mingw-w64 xvfb imagemagick xdotool
git config --local core.hooksPath .githooks
```

Windows の実機で動かして確かめるまで「Windows で動く」と書かない（QLT-013）。

### 画面の確かめ方（施主 2026-10-22）

| 対象 | 方法 | 記録 |
| --- | --- | --- |
| X11 版（AI） | `make screenshot` で Xvfb の上に窓を出して PNG を撮り、AI が画像を読んで確かめる。操作は `xdotool` で駆動して、操作の後の画面も撮る | 「仮想の X サーバでの確認」として gate-proofs に（QLT-013） |
| X11 版（人） | WSLg の上で窓を出し、施主が見て触る | 施主の確認として記録 |
| Windows 版 | `make lint-windows`（clippy・exe を作らない）で型と lint を見る。exe は施主の実機で組み立てて確かめる（試験用の exe が開発機で検疫された経緯があり、配る exe の経路は別の Issue） | 実機の確認を gate-proofs に記録してから `確認済み` と書く。それまで「Windows で動く」と書かない |

Wine での目安の確認は、要るときに施主の判断で入れる（Wine は Direct2D・IME・DWM を本物どおりには再現しないので、QLT-013 の実機の代わりにならない）。

### 開発機に固有の情報

開発機に固有の情報（使ってはいけない道具・ファイルの置き場・Windows での確かめ方）は、git 管理外の 2 つのファイルに置く（GIT-005）。

| ファイル | 中身 |
| --- | --- |
| `CLAUDE.local.md` | 開発機の制約と置き場。Claude Code が `CLAUDE.md` と一緒に読む |
| `.claude/settings.local.json` | 外への送信を拒む設定と、書き込み先の制限 |

リモートは `origin`（GitHub）だけ。配備先への `ssh` / `scp` / `rsync` は無い。

---

## 10. リリース

製品の版入力は `Cargo.toml` の `[workspace.package] version` だけとする。画面の表示はここから導出する。
配布の手順は別の焦点 Issue とする。
