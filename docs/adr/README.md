# アーキテクチャ決定記録（ADR）

トレードオフのある判断を、**理由ごと**残す場所。正本ドキュメントが「いま何が規範か」を書くのに対し、
ADR は「なぜそう決めたか」「何を却下したか」を書く。

## 書き方

- 連番 4 桁 ＋ kebab の題名: `NNNN-short-kebab-title.md`
- `0000-template.md` を複製して書く
- 状態は `提案` / `受理` / `却下` / `置換（→ NNNN）`
- **却下した選択肢を必ず書く。** 再提案は、その理由への反論から始めること
- 受理された ADR を後から書き換えない。変えるときは新しい ADR で置き換える
- 文脈は可能な限り実データ（コード・履歴・計測・コンパイラの出力）で裏を取る

## 一覧

| ADR | 題名 | 状態 |
| --- | --- | --- |
| [0001](0001-strictness-is-mechanically-enforced.md) | 厳格さは機械で強制する | 受理 |
| [0002](0002-no-toolkit-x11rb-cairo-win32-direct2d.md) | X11 は x11rb ＋ cairo、Windows は Win32 ＋ Direct2D。UI ツールキットを使わず、部品は自前で描く。Windows を主な対象とする | 受理 |
| [0003](0003-diff-scoped-verification-and-result-reuse.md) | 差分に必要な検証と成功結果の再利用 | 受理 |
| [0004](0004-local-only-repository.md) | リモートを置かない。Issue はリポジトリの中のファイル、PR の代わりは統合記録と squash | 置換（→ 0021） |
| [0005](0005-source-is-read-only-until-save.md) | 原本は保存の置き換えまで書かない。編集・変換・並べ替えは原本に重ねる派生値で、触っていない行はバイト列のまま書き戻す | 受理 |
| [0006](0006-confined-unsafe-for-scan.md) | 原本の写像と SIMD の探索のために、`scan` の 1 クレートにだけ `unsafe` を許す | 受理 |
| [0007](0007-own-engine-and-external-benchmarks.md) | 並べ替え・集計・統計は自前で作る。DuckDB などは組み込まず、比較相手として使う | 受理 |
| [0008](0008-cover-all-features-in-phases.md) | 機能は全部カバーする。一覧（FEATURES.md）を正本にしてフェーズで覆い、手順・SQL・プリセット・xlsx・和暦・印刷をこう決める | 受理 |
| [0009](0009-license-free-use-no-sale.md) | ライセンスは独自: 使うのは自由、改変はクレジット必須、配布は無償に限り、販売は禁止 | 受理 |
| [0010](0010-rust-foundation-and-measured-limits.md) | Rust の検査基盤を NeNeDocket から移し、このリポジトリで測れた規則だけを active にする | 受理 |
| [0011](0011-ui-language-is-a-versioned-setting.md) | 表示言語は 6 言語の閉じた集合で、文言は core の 1 表から出し、希望は版付きの設定に残して OS の観測値で解決する | 受理 |
| [0012](0012-theme-model-and-builtin-colorschemes.md) | 配色は nene-nib の組み込み 125 テーマを写し、画面の色は 3 色から整数の補間で導く | 受理 |
| [0013](0013-own-title-bar-and-frame.md) | 窓のタイトルバーと枠も自前で描く。移動・リサイズ・スナップ・影は OS の仕組みに乗る | 受理 |
| [0014](0014-background-jobs.md) | 重い処理は裏の一括処理。状態は application の状態機械、実行は scan / store のスレッド、結果は塊の順に合わせる | 受理 |
| [0015](0015-encodings-with-encoding-rs.md) | 文字コードは `encoding_rs` で復号・符号化する。原本のバイト列は復号した結果から作り直さない | 受理 |
| [0016](0016-ui-dependencies-x11rb-cairo-windows.md) | 画面の依存は `x11rb`・`cairo-rs`・`windows` の 3 つだけ。Windows のコードは当面型検査だけ | 受理 |
| [0017](0017-reading-the-source-and-sparse-row-index.md) | 原本は位置を指定した読み出しで塊ごとに読み、行の索引は 1,024 行ごとのまばらな目印。写像は測ってから | 受理 |
| [0018](0018-unclosed-quote-presentation.md) | 閉じない引用符は、規則の正本を厳格に保ったまま位置を示し、「その引用符を文字として読む」を利用者が選べる別の規則にする | 受理 |
| [0019](0019-text-units-in-the-splitting-rule.md) | 切り出しの規則に「文字の単位」を入れ、区切り・引用符・改行は文字の頭でだけ見分ける | 受理 |
| [0020](0020-zoom-and-scale.md) | 文字の大きさの変更は表と詳細ペインの字だけを変え、段階は application が決めて UI が測り直す。固定の画素は画面の倍率の 1 つの経路で掛ける | 受理 |
| [0021](0021-public-repository-on-github.md) | 公開リポジトリを GitHub に置く。Issue は GitHub の Issue、PR が統合の境界、CI は `make ci` を呼ぶだけ、`main` は ruleset で守る | 受理 |
| [0022](0022-calculation-on-values-not-formulas.md) | 計算は値に対して行い、数式を保存も評価もしない（集計パレット STAT-09・列の演算 XFRM-17） | 受理 |
| [0023](0023-real-world-corpus-measurement.md) | 実データの突き合わせを、出どころと正解の表をリポジトリに置いて再現できる測定にする（道具は `tools/corpus/` の Python・`make bench-corpus`） | 受理 |
| [0024](0024-edit-layer-over-the-source.md) | 差分は「行の並び」と「セルの書き換え」の 2 つの層で、原本のレコードの番号に重ねて持つ（型と規則は core、持ち主と Undo は application） | 受理 |
| [0025](0025-save-by-replacement-and-windows-locks.md) | 保存は store クレートが一時ファイルに書いて置き換える。原本は application のポート越しに読み、Windows の置き換えは adapters-windows に置く | 受理 |
| [0026](0026-cell-editing-entry-and-japanese-input.md) | セルの編集は Excel と同じ入り方にする。欄の規則は core、日本語は Windows を IMM32 で先に、X11 は当面の貼り付けから | 受理 |
| [0027](0027-row-and-column-insert-delete.md) | 行と列の挿入と削除は Excel と同じ見え方と操作にする。行の番号は上から通し番号、消すときは尋ねずに Undo で戻す | 受理 |
| [0028](0028-new-file-and-save-as.md) | 新規作成は新しい窓で 1 行 × 3 列から始め、名前を付けて保存は Windows は OS の画面、X11 はパレットの欄で道筋を打つ | 受理 |
| [0029](0029-paste-many-cells-and-cut.md) | 複数のセルの貼り付けは左上から束のまま書き、表の端を越えれば行と列を足して 1 手で戻す。切り取りは貼るまで元を消さない | 受理 |
| [0030](0030-find-and-replace.md) | 検索はパレットの欄の段で、見えている値（原本 ＋ 差分）を裏で探す。置換は 1 手の束にし、全部の置換は件数を見せてから | 受理 |

## これから書くもの（Issue が立ったら）

- 外部の変更の検出の方式（FR-019）
- 印刷と PDF の方式の細部（FR-017・ADR 0008 の 7）
- 正規表現・xlsx・Parquet の依存か自前か（ADR 0008 の 5。正規表現は ADR 0030 で `regex` に決めた）
- 写像（mmap）の導入（ADR 0017 の決定 6 の 3 つがそろったら）
