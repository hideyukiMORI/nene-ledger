# いまのタスク — NeNe Ledger

> GitHub Issue と PR が進行状態の正本。この文書はその要約（2026-11-04 時点・公開リポジトリの初日）。順番の正本は [ROADMAP.md](../ROADMAP.md)。
> 🔴 公開前（2026-10-22〜11-02）はリモートの無いローカルの git で Issue #1〜#68 を進めた。残した記録（[gate-proofs](../quality/gate-proofs.md)・[benchmarks](../quality/benchmarks.md)・[FEATURES](../FEATURES.md)）の `Issue #N` はその番号で、GitHub の番号とは別物（[ADR 0021](../adr/0021-public-repository-on-github.md)）。

## 状態

| | |
| --- | --- |
| 製品 | CSV / TSV のエディター。Windows（主）と X11。巨大なファイルでも速く、勝手に変えない（SPECIFICATION）。施主決定 2026-10-22 |
| 言語と土台 | Rust / Win32 ＋ Direct2D / x11rb ＋ cairo-rs（[ADR 0001](../adr/0001-strictness-is-mechanically-enforced.md)・[ADR 0002](../adr/0002-no-toolkit-x11rb-cairo-win32-direct2d.md)） |
| データの扱い | 原本は保存の置き換えまで書かない。差分・変換の規則・表の見えを重ねる（[ADR 0005](../adr/0005-source-is-read-only-until-save.md)） |
| 速さの区画 | `scan` にだけ `unsafe`（写像・SIMD）。正本は core の参照実装（[ADR 0006](../adr/0006-confined-unsafe-for-scan.md)）。いまの `unsafe` は 0 |
| 分析 | 自前。DuckDB などは比較相手（[ADR 0007](../adr/0007-own-engine-and-external-benchmarks.md)） |
| リポジトリ | GitHub（PUBLIC）。Issue → PR → `check`（`make ci`）→ squash merge。`main` は ruleset（[ADR 0021](../adr/0021-public-repository-on-github.md)） |
| ライセンス | 独自・販売禁止・クレジット必須（[ADR 0009](../adr/0009-license-free-use-no-sale.md)） |
| M0（仕様・規範文書・機械・機能の一覧 213 項目・ロードマップ M0〜M6） | ✅ 公開前の Issue #1〜#6 |
| M1 のクレート | ✅ core（切り出しの参照実装・方言と文字コードの判別・表の配置・配色・文言）/ scan（塊の読み出し・まばらな索引・`encoding_rs`・裏のスレッド）/ application（表示のフレーム・一括処理の状態機械・選択・パレット）/ ui-x11 ＋ adapters-x11 / ui-win32 ＋ adapters-windows / app（合成ルート） |
| M1 の画面 | ✅ 自前のタイトルバー・見出しの固定・スクロールバー・カーソルと矩形選択・離れた範囲・行全体と列全体・列の入れ替えと幅・詳細ペイン・125 配色と OS の明暗・文字の大きさと画面の倍率・コマンドパレット・キーの一覧・状態の行（読み方・カーソル・件数と合計と平均） |
| M1 の読み方 | ✅ 区切り（任意の字も）・引用（`\"` の流派・シングルクォート）・文字コード（BOM・UTF-8・CP932・EUC-JP・UTF-16・Windows-1252）・見出しの行・前書きの行・閉じない引用符・文字のファイルでないものの確認・見えない文字の印。パレットで試して開き直す |
| 正しく読めた率 | Pollock 2,290 ファイルで重み付き 9.6601（18 の道具の 2 位・[benchmarks](../quality/benchmarks.md) 第 1 節） |
| Windows | ✅ 実機で窓・倍率（125% / 150%）・起動の入口（ファイルを選ぶ画面・コンソール無し）を確認。配る exe の経路はまだ（試験用の exe が検疫された経緯） |
| 公開リポジトリへの移行（ADR 0021・CI・ruleset・GIT 規則の置き換え） | ✅ Issue #1 / PR #2（gate-proofs 第 51 節。CI の success と failure を両方見た） |

## M1 に残っている物（公開前の Issue #50 の棚卸しを引き継ぐ）

M1 の 56 項目は `確認済み` 21・`実装済み` 8・`未着手` 27。残りを分けると:

- **Windows**: IME（FR-020）・ファイルのドラッグ＆ドロップ（CLIP-04）・Issue #62〜#68 で入れた操作の実機での確認
- **方言の指定と開き直し**: 判別の候補と確信度を示すこと（OPEN-02）・見出しの行の数と読み飛ばす行の数（OPEN-11）
- **文字コードと改行**: JIS の Shift_JIS の区別（JP-01・OPEN-09 の残り）・ファイル全体での改行の混在（OPEN-08）
- **壊れた行**: 元のバイト列での隔離・移動・読み直しを尋ねる画面（OPEN-15・ADR 0018）・ファズと最悪の手間（SEC-04）
- **表示と移動**: 左の列の固定（VIEW-01）・JSON の整形とファイルの文字コードでのバイト数（VIEW-07）・数万列（BIG-08）

## 次の 1 手

上の「M1 に残っている物」から 1 つずつ Issue を立てる（最初の候補: OPEN-02 の候補と確信度の提示、VIEW-01 の左の列の固定）。
