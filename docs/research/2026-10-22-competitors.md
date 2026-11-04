# 調査: 競合の GUI の CSV / TSV エディター・ビューアーの機能

> Issue #3 / 2026-10-22 時点。Web の調査だけ（各社の公式ページと第三者のレビュー）。※未確認は、情報源の間で食い違うか一次情報を確かめられなかったもの。
> 一覧への反映は [機能の一覧](2026-10-22-feature-catalog.md)。

## 1. 製品ごとの概要

| 製品 | OS | 価格 | 規模の上限（公称） | 主な機能 |
| --- | --- | --- | --- | --- |
| **Modern CSV** | Win / Mac / Linux | 無料版あり。Premium Personal $39・Business $59（買い切り） | 行・列とも最大 2,147,483,647。100 MB を超えると読み取り専用で開くか確認（閾値は設定）。読み取り専用は索引を作ってディスクから読む。メモリは編集時にファイルの約 3〜4 倍、読み取り専用で 20〜25%（開発者の 2022 年の発言） | コマンド 200 以上・command launcher・キー割り当て・複数セルの編集・セルの結合と分割・転置・reshape・ID 列・重複行と空行の削除・大文字小文字と日付と数の書式変換（Premium）・独自構文のフィルター・統計・列の分析・一意値とヒストグラム・ピボット・Column Lookup（VLOOKUP 相当）・JSON / XML の入出力・印刷と PDF（Premium）・外部の変更で自動再読み込み・テーマ 5 種・ファイルの複製 / 改名 / 削除 |
| **EmEditor**（CSV モード） | Windows | Free（制限あり）・Pro 月 $6 / 年 $60 | 最大 16 TB・約 1 兆行（旧資料は 248 GB）。それ以上は Large File Controller で一部ずつ開く。多スレッドと SIMD（AVX-512 / AVX2） | CSV / TSV / 任意区切りを最大 64 形式・セル選択モード・見出しの固定・複数列の並べ替え・Advanced Filter（列ごと・正規表現・除外）・Join CSV（INNER / OUTER / LEFT / RIGHT）・列の結合と分割・Flash Fill・オートフィル・転置 / ピボット / アンピボット・重複行の削除・共通文字列の数え上げ・セル内の改行の除去・余分な引用符の除去と全セルの引用・固定幅への変換・列数をそろえる・ファイルの分割と結合・**CSV の検証（列数の不一致・不正な `"` を一覧し該当行へ飛ぶ）**・マクロの記録・Shift_JIS を含む Windows の全コードページ・文字コードの判別と読み直し |
| **Ron's Data Edit** | Windows | Lite（無料・保存 2,500 行まで・広告）・Pro $37.50 | 公称なし | 区切りと文字コードの判別（UTF-7/8/16/32）・列の型の判別・Page per Row（1 行を 1 画面のフォームで）・Cleaners（再利用できる自動処理の手順）・Toolbox（ライブプレビュー付きの変換）・Data Summary・グラフ・カスタムビュー・行と列の分割と結合・ピボット・JSON / XML / HTML / Markdown / Excel の入出力・Web から取り込み・ファイルのロック |
| **Tad** | Win / Mac / Linux | 無料（MIT） | 公称なし（DuckDB） | 閲覧専用。CSV / TSV / Parquet / SQLite / DuckDB・ピボット・AND / OR の述語フィルター・数の列のヒストグラム・数の右寄せ・ズーム・Excel / Google Sheets へのコピー・CSV / Parquet への書き出し（最新 0.14.0・2024-06・更新は少ない） |
| **CSVed** | Windows | 無料 | 仮想モードの一覧 | 区切りの変更・列の追加 / 削除 / 並べ替え・分割と結合・接頭辞と接尾辞・列単位の検索・並べ替え・フィルター・重複の削除・追記とマージ・選択やフィルター結果だけの保存・HTML / Excel / Word / XML・連番列。Unicode は別版（一部機能）。2023 年以降は実質更新なし |
| **Easy CSV Editor** | macOS | $9.99 ＋ 入出力 $7.99 | 2 GB 超・1,500 万行（第三者の記事） | Column Explorer（列ごとの統計）・フィルターの保存と読み込み・範囲の並べ替え・**2 つの版の比較**・ファイルの分割・縦の行表示・数の再書式化（小数点・桁区切り）・JSON の整形・入力の候補・スペルチェック・AppleScript / Unix スクリプト・多数の形式へ変換・印刷 / PDF・ダークモード |
| **Tablecruncher** | Win / Mac / Linux | 無料（GPLv3・2025-04 から） | 2 GB 超・1,500 万行超 | 形式と文字コードの判別（UTF-8/16・Latin-1・CP1252。Shift_JIS の記載なし）・正規表現の検索置換・JavaScript のマクロ・行のフラグと書き出し・JSON・テーマ |
| **SmoothCSV v3** | Win / Mac / Linux | 全機能無料・任意の買い切り（約 $29 ※）・Team | 100 MB を Excel の約 12 倍で開く（公称）。Windows で 640 MB 付近が頭打ち（※） | command palette・**SQL Console（SELECT）**・ビジュアルなクエリ作成・**CSV Compare（差分とマージ）**・Smart Sort（数 / 日付 / 文字列を判別・複数列）・SQL 風フィルター・Quick Filter・Record View・Cell Value View・転置・Fill Blanks Down・集計の固定表示・テキスト変換・結合と分割・**引用の制御と引用の表示**・**空白文字の可視化**・形式のプリセットとルール・Copy As（Markdown / HTML / JSON / SQL / LaTeX）・xlsx の入出力・PDF・CLI・ディープリンク・自動バックアップ・hot exit・外部の変更で自動再読み込み・11 言語（日本語）・Shift_JIS / EUC-JP（作者は日本人） |
| **Delimit** | Windows | Pro（有料）・Lite（無料） | 20 億行・200 万列（RAM 8 GB） | 複数文字の区切り・固定幅・ブックマーク・**式で列を作る**・式で行を抽出・別ファイルとの照合（一致 / 不一致・別ファイル基準の重複削除）・縦 / 横 / 値ごとのファイル分割・列の値でのファイル結合・区切りの変換 |
| **Table Tool** | macOS | 無料（MIT） | 小規模 | 区切り・引用・見出し・文字コード・小数点の判別・保存で形式を変換。意図的に機能を絞る（並べ替えなし）。派生の Tabletool-Swift は日本語 UI |
| **CSView（Kothar）** | macOS（v2）・3 OS（v1・OSS） | £1.99 | 数 GB・3 億行超 | 閲覧専用・**裏で索引を作りながら先頭から表示**・全文検索と件数・Markdown / リッチテキストでコピー・ズーム |
| **CSVFileView（NirSoft）** | Windows | 無料 | 「非常に大きなファイルは読めない」と明記 | 列の選択と並べ替え・数 / 日付の自動の並べ替え・Shift+クリックで第 2 キー・quick filter・SQL の WHERE 風の表示フィルター・多形式へ書き出し・コマンドラインでの変換 |
| **Rainbow CSV**（VS Code） | VS Code / vscode.dev | 無料 | 30 万行か 20 MB を超えると設定変更、50 MB を超えると無効 | 列の色分け・区切りの判別・**ファイルを変えない仮想の整列**・sticky header・交互の行の色・列の追跡・ホバーで列の情報・CSVLint（引用と列数）・RBQL（SQL 風）・マルチカーソルでの列編集・Excel / Markdown 形式のコピー・コメント行 |
| **Edit csv**（VS Code） | VS Code・Web | 無料 | 大きなファイルは苦手 | 表形式の編集（handsontable）・オートフィル・引用の保持（列内で混在すると統一される）・コメント行。編集中に元ファイルが変わっても反映しない |
| **LibreOffice Calc** | Win / Mac / Linux | 無料 | 1,048,576 行（新しい版で約 1,600 万行 ※未確認） | 取り込み: 文字コード・言語・開始行・区切り（複数）・連続した区切りの統合・空白の除去・文字列の区切り・引用されたフィールドを文字列として扱う・特殊な数と指数表記の判別・数式の評価・**列ごとの型（Text / Date DMY など / Hide）**・固定幅。書き出し: 文字コード・区切り・引用符・**全テキストセルの引用**・表示どおりに保存・固定幅 |
| **Excel / Power Query** | Win / Mac | M365 | 1,048,576 行 × 16,384 列・1 セル 32,767 文字・元に戻す 100 段・並べ替えのキー 64 | From Text/CSV: File Origin（BOM が無いと UTF-8 と判別しない）・区切り（任意の文字列・固定幅）・型の判別（先頭 200 行 / 全体 / しない）・引用内の改行・Extract Table Using Examples・データモデル（行数の制限を回避）。自動データ変換の個別のオフ（先頭の 0・15 桁超・E 表記）と警告 |
| **Gigasheet** | Web | Free 3 GB・有料 月 $19〜$95（※） | 10 億行超 | フィルター・グループ化・ピボット・結合・エンリッチメント・AI・DB 連携。最近は医療の価格データ向けに方針転換 |
| その他 | — | — | — | **Data.olllo**（Win・Polars・100 GB 超を公称・AI・手順の記録）・**Row Zero**（クラウド・20 億行以上・Python・Databricks が買収）・**CSV Explorer**（Web・無料 500 万行 / 有料 2,000 万行）・**Large Text File Viewer**（Win・ライブ追従・正規表現）・**reCsvEditor**（OSS）・**VisiData**（TUI）・**Cassava Editor**（国産・無料・巨大ファイルは苦手） |

## 2. 機能の全体像（カテゴリ別・括弧は対応製品の例）

**開く・文字コード・方言の判別**

- 区切りの判別（ほぼ全製品）・任意の文字や複数文字の区切り（Delimit・Rainbow・EmEditor・Power Query）・固定幅（Delimit・LibreOffice・Power Query）
- 文字コードの判別（Tablecruncher・Easy・Ron's・Table Tool・EmEditor）・Shift_JIS / EUC-JP（EmEditor・SmoothCSV）・BOM の扱い
- 引用・見出し・小数点の判別（Table Tool）・ファイルごとの方言とルールの自動適用（SmoothCSV・EmEditor）
- 列数のそろわない行や引用内の改行を壊さず保つ（SmoothCSV・Easy・Modern）・開始行の指定（LibreOffice）
- 列ごとの型の指定・文字列のまま扱って先頭の 0 を守る（Ron's・Easy・LibreOffice・Power Query・Excel の自動変換オフ）

**表示**

- 見出しや列の固定（Modern・EmEditor・Ron's・Rainbow）・列の幅の変更と自動調整・配置・ズーム（Tad・CSView）
- 折り返し（SmoothCSV・Rainbow）・テーマとダークモード・行の交互色
- 列の色分け（Rainbow）・行と列の非表示・引用符の一時非表示（EmEditor）・引用や空白の可視化（SmoothCSV）
- レコード / フォーム表示（SmoothCSV・Easy・Ron's）・セルの値の詳細ペイン（SmoothCSV）・長い文字列のツールチップ・数の右寄せ（Tad）

**編集**

- 複数セルの同時編集（Modern・SmoothCSV）・オートフィルと連番・Flash Fill（EmEditor）
- 行と列の挿入 / 削除 / 移動 / 複製・ドラッグ移動・セルの入れ替え・ID 列・乱数と現在日時の入力（Modern）
- 元に戻す / やり直す（ほぼ全製品・Excel は 100 段）・読み取り専用モード（Modern・SmoothCSV・EmEditor）・自動バックアップと hot exit（SmoothCSV）

**検索・置換**: 正規表現（多数）・ワイルドカード・列を限った検索・置換前のプレビュー（Modern）・全一致の選択（Modern）・件数（CSView）・ブックマーク（Delimit）

**並べ替え**: 複数キー・型を判別した数 / 日付の並べ替え・長さで並べ替え・安定（Modern・EmEditor）・範囲だけ・行の横方向。**自然順（natural sort）を明記した製品は見つからなかった** ※未確認

**フィルター**: 見出しからの quick filter・複数列の AND / OR・否定 / 除外・数の範囲・正規表現・式（Delimit・SmoothCSV の SQL・RBQL）・一意値の一覧から・条件の保存（Easy）・見えている行だけの保存・フィルター結果を削除して確定

**変換**: 大文字小文字と命名規則・trim・接頭辞と接尾辞・列の分割と結合・日付と数の書式・先頭の 0 の除去と 0 埋め・重複の削除・空行と空列の削除・転置・ピボット / アンピボット・reshape・空白を上の値で埋める・式で列を作る（Delimit）

**統計・要約**: 列の統計（件数・合計・平均・中央値・分散・パーセンタイル）・列の型 / 空の率 / 長さ・一意値と頻度・ヒストグラム・グラフ（Ron's）・集計行の固定表示（SmoothCSV）・共通文字列の数え上げ（EmEditor）

**結合・参照・マージ**: 2 ファイルの JOIN（EmEditor の 4 種・Delimit）・VLOOKUP 相当（Modern Business）・追記と連結・一致 / 不一致の抽出（Delimit）・ファイルの分割

**検証**: 列数と引用の検査（EmEditor・Rainbow の CSVLint）・列数をそろえる（EmEditor）。**スキーマによる検証を持つ製品は見つからなかった**

**比較・差分**: CSV Compare（SmoothCSV・差分とマージ）・版の比較（Easy）

**書き出し**: 区切り・引用・文字コード・改行の変更・全セルの引用 / 必要なときだけ引用・JSON / XML / HTML / Markdown / xlsx / Parquet / SQL / LaTeX / PDF・選択範囲だけの保存

**クリップボード・Excel 連携**: Excel との貼り付け・見出し付きのコピー・形式を選んだコピー

**拡張・自動化**: command palette・キー割り当て・マクロ（EmEditor の記録・Tablecruncher の JS・Easy の AppleScript）・SQL（SmoothCSV・RBQL）・CLI・手順の記録と再実行（Ron's Cleaners・Data.olllo）・Python（Row Zero・VisiData）

**大きなファイル**: 索引を作って遅延読み込み（Modern の読み取り専用・CSView・Large Text File Viewer）・ディスクを使った部分読み込み（EmEditor）・仮想表示（CSVed）・DuckDB / Polars（Tad・Data.olllo）・多スレッドと SIMD（EmEditor）・進み具合（Easy）

**ファイルの監視**: 外部の変更で自動再読み込み（Modern・SmoothCSV）・追記の追従（Large Text File Viewer）

**印刷・PDF**: Modern（Premium）・SmoothCSV・Easy

**アクセシビリティ**: **どの製品も明確な記載なし**（空白）

**多言語**: SmoothCSV（11 言語・日本語）・Tabletool-Swift（日本語）・EmEditor（日本語版）

## 3. 差別化の要素

- **EmEditor**: 16 TB 級・Large File Controller・4 種の Join・Flash Fill・CSV の検証とエラー行へ飛ぶ・マクロの記録・日本語の文字コードへの強さ
- **Modern CSV**: 編集と読み取り専用を開いたまま切り替え・独自のフィルター構文・ファイル分析・Column Lookup・200 以上のコマンドにキーを割り当て
- **SmoothCSV**: SQL Console とビジュアルなクエリ・差分とマージ・Copy As・形式のルールの自動適用・ディープリンクと CLI・hot exit
- **Delimit**: 20 億行 × 200 万列・式の言語・別ファイルとの照合・値ごとのファイル分割
- **Ron's Data Edit**: Page per Row・再利用できる Cleaners・ライブプレビュー付きの Toolbox・グラフ
- **Tad**: Parquet や DuckDB も開ける閲覧専用・ピボット
- **Easy CSV Editor**: 版の比較・スペルチェック・入力の候補・AppleScript
- **Rainbow CSV**: ファイルを変えない仮想の整列・RBQL
- **Excel / Power Query**: Extract Table Using Examples・データモデル・自動変換のオフ
- **CSView / Large Text File Viewer**: 裏で索引を作りながら先頭からすぐ見せる・追記のライブ追従

利用者の不満と各製品の答え: Excel が先頭の 0 や 15 桁を超える数を壊す → 文字列のまま扱う。1,048,576 行の制限 → 数千万〜数十億行。クラウドに上げたくない → ローカルで処理（Delimit・Easy・Data.olllo）。

## 4. 出典

- https://www.moderncsv.com/ ・ https://www.moderncsv.com/documentation/commands-reference/ ・ https://www.moderncsv.com/documentation/sort-filter-hide/ ・ https://www.moderncsv.com/documentation/file-analysis/ ・ https://www.moderncsv.com/documentation/basics/ ・ https://www.moderncsv.com/buy/ ・ https://news.ycombinator.com/item?id=31576554
- https://www.emeditor.com/text-editor-features/powerful-csv-tools/ ・ https://www.emeditor.com/text-editor-features/more-features/csv-tsv-dsv/ ・ https://www.emeditor.com/text-editor-features/large-file-support/ ・ https://help.emeditor.com/en/cmd/csv/index.html ・ https://www.emeditor.com/forums/topic/msg-inconsistent-number-of-columns-detected/ ・ https://download.emeditor.info/doc/working-with-csv.pdf
- https://www.ronsplace.ca/products/ronsdataedit ・ https://www.ronsplace.ca/products/ronsdataedit/features
- https://www.tadviewer.com/
- https://home.hccnet.nl/s.j.francke/csved/index.html
- https://apps.apple.com/us/app/easy-csv-editor/id1171346381?mt=12
- https://tablecruncher.com/
- https://smoothcsv.com/features ・ https://smoothcsv.com/pricing ・ https://smoothcsv.com/blog/best-csv-editors-in-2026 ・ https://dev.classmethod.jp/articles/smoothcsv-3/ ・ https://zenn.dev/kohii/articles/23fee770d7dbe8
- https://delimitsoftware.com/
- https://github.com/jakob/TableTool ・ https://github.com/aoyamashou/Tabletool-Swift
- https://kothar.net/csview
- https://www.nirsoft.net/utils/csv_file_view.html
- https://marketplace.visualstudio.com/items?itemName=mechatroner.rainbow-csv ・ https://github.com/mechatroner/vscode_rainbow_csv
- https://marketplace.visualstudio.com/items?itemName=janisdd.vscode-edit-csv
- https://help.libreoffice.org/latest/en-US/text/shared/00/00000208.html ・ https://help.libreoffice.org/latest/en-US/text/shared/00/00000207.html
- https://support.microsoft.com/en-us/office/excel-specifications-and-limits-1672b34d-7043-467e-8e27-269d656771c3 ・ https://learn.microsoft.com/en-us/power-query/connectors/text-csv ・ https://support.microsoft.com/en-us/excel/set-automatic-data-conversions
- https://www.gigasheet.com/ ・ https://rowzero.com/ ・ https://olllo.top/no_code_csv_viewer_editor_offline ・ https://martech.zone/csvexplorer-large-csv-files/ ・ https://www.softpedia.com/get/Office-tools/Other-Office-Tools/Large-Text-File-Viewer.shtml ・ https://www.visidata.org/

補足: CSVed の新しい公式サイトと Easy CSV Editor の公式サイトは DNS エラーで開けず、旧サイトと App Store の情報で補った。
