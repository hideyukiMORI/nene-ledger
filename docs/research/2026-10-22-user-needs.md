# 調査: ヘビーユーザーの要望と不満・パワーツールの機能

> Issue #3 / 2026-10-22。Web の調査だけ。（不確か）は情報源で食い違うか検証できていない項目。
> 制約: 検索の道具から reddit.com と superuser.com に届かなかった（ドメインのブロック）。Hacker News・各社のコミュニティ（Shopify・Microsoft Q&A・MrExcel・Biostars）・G2 / Capterra / AlternativeTo・国内（Qiita・Zenn・note・知恵袋・各社の FAQ）で補った。ベンダーのブログ（SplitForge・Gigasheet など）は自社に有利に書きがちなので割り引いた。

## 1. 利用者のタイプ別の要望と不満

### データアナリスト

- 不満: Excel は 1,048,576 行を超えた分を捨てる。警告の出方の説明は情報源で違い（不確か）、気付かずに一部で集計する恐れ。開くのが遅い
- 要望: Excel 風のドロップダウンのフィルター（チェックボックス）・統計と列のプロファイル。Modern CSV v2 の HN では「列フィルターが有料版だけ」「統計が法人版だけ」が不満

### データエンジニア / ETL

- 不満: 数百万〜数億行。「SQLite に一括で読み込むのが一番楽」が主流の回避策（HN 14919149）。数百万行の削除・コピーで固まる → 進み具合と裏での処理（HN 37140159）。小数点の地域差に弱い
- 要望: SQL での問い合わせ・列の検証（正規表現）・**読めなかった行を別に保存**（DuckDB の `store_rejects`）・**元の書式（引用の付け方）を保って保存**（SmoothCSV の売り）

### EC の商品カタログ（Shopify / Amazon / 楽天）

- 不満: Excel で開くと UTF-8 が化け、そのまま取り込んでストアが壊れる（`okosÃ³ra`）・スマートクォートで「Illegal quoting」・欧州の設定でセミコロン区切りになる・Handle の表記ゆれでエラーなしに重複商品・UPC / JAN が指数表記（2.5E+12）になり 15 桁を超える桁が 0 に置き換わって戻せない
- 楽天 RMS: JAN の先頭の 0 が消える・列が 1 つずれると全行エラーだがログは「項目数が一致しません」だけ・空欄が「変更なし」ではなく空で上書き・コントロールカラムの `d` でレビューごと消える事故。文字コードの指定は記事で Shift_JIS と UTF-8 に分かれる（不確か）
- 要望: テンプレートの列の順の固定・アップロード前の検証（必須・ID の重複・数の列の文字）・エラー行だけを取り出して直す

### 経理・バックオフィス

- 不満: 弥生会計は Shift_JIS・CRLF・項目数 25 / 27（Next は 28）しか取り込めない。改行が CR だと「インポート可能な形式ではない」でログも出ないことがある。Excel の「名前を付けて保存」で意図せず CSV UTF-8 になる。銀行の明細は銀行ごとにばらばら（日付の MM/DD と DD/MM・負の数の `(30.00)`・入金と出金が別の列・合計行やフッター）。入出金が別列のまま Xero に取り込むと出金が警告なしに捨てられることがある
- 要望: 文字コード・改行・項目数を見える形で示して変換できる・取り込み先の形式に合わせた変換のプリセット

### 研究者・バイオインフォマティクス

- 不満: 遺伝子名が日付に（SEPT1 → 9 月 1 日。HGNC は 27 個を改名。2016 年の調査で論文の約 5 分の 1）・`2310009E13` が指数表記に・数百 MB の TSV を Excel で扱えない（Biostars p/298429）
- 要望: 型を推し量らない表示・巨大な行列の部分的な読み込み・転置

### ローカライズ / 翻訳

- 不満: セル内の改行が正しく扱われず列がずれる（54 列の 33 列目の改行で以降がずれた例）・BOM が無いと Excel が Latin-1 で開いて化ける・ノーブレークスペースなど見えない文字・TSV でタブとスペースの取り違え
- 要望: セル内の改行を安全に編集・見えない文字の表示・戻ってきたファイルを元とキー（ID）で照合

### QA / テストデータ

- 要望: 空セルや重複を一定の割合で混ぜたダミーデータの生成・種を固定して再現・日本語ロケール。集まった声は「生成」側に偏り、「編集」側は少なめ（不確か）

### CRM / マーケティングのリスト

- 不満: Mailchimp はファイル内の重複・UTF-8 でない・セミコロン区切りで取り込みに失敗。HubSpot はファイル内の重複を検出せず、選択肢の大文字小文字の違いだけでエラーなしに空欄になる
- 要望: 重複の削除・メールアドレスの前後の空白の除去と小文字化・取り込み先の項目名への列の割り当て・失敗した行だけを抜き出して直す

### ログ分析

- 要望: 全体を記憶に載せない読み込み（索引）・正規表現の絞り込みで一致した行と全体の前後を 2 つのペインで（klogg）・時刻の認識と時刻ごとの件数のヒストグラム・SQL（lnav）・追記の追従（tail）。CSV で書き出したログに特有の声は見つからなかった（不確か）

### GIS

- 不満: 地域設定で `-58.030128` が -58,030,128 に・WKT が長すぎて Excel のセルの上限を超える・緯度経度の割り当てを間違えると Null Island（0, 0）
- 要望: 座標の列を文字列のまま保つ・小数点の地域差の設定・簡単な地図のプレビュー

### 行政のオープンデータ公開

- 作るときのルール: 1 セル 1 データ・和暦だけは不可（西暦を併記）・機種依存文字を使わない・UTF-8（BOM 付き）・セル結合をしない（狛江市のマニュアル・総務省の資料・国の落札実績の仕様）
- 検証: 海外では CSVLint や Frictionless で見出しの重複・空行・型を検証。セル内の LF と CRLF の混在で誤検知した例（frictionless #803）

## 2. 要望の多い機能（出現の多い順・定性的）

1. **データを勝手に変えない**（先頭の 0・日付・指数表記・15 桁の切り捨て）— 全タイプに共通し最多
2. **文字コードの判別・指定・変換**（UTF-8・BOM 付き・Shift_JIS / CP932・Latin-1）— 日本と欧米の両方
3. **巨大なファイルを開く**（100 万行超・GB 級・読み取り専用で素早く・進み具合）
4. **区切り・引用・改行を正しく扱い、保存で元の形を保つ**（セル内の改行・全項目を引用・CRLF / LF。Excel には全項目を引用する設定が無いという不満が teratail と知恵袋に）
5. **フィルター・並べ替え・検索**（正規表現・Excel 風のドロップダウン）
6. **取り込み前の検証**（必須・重複・型・正規表現・項目数）と**エラー行だけを取り出して直す**
7. **2 つの CSV をキー列で比べる差分**（Qiita「CSV に diff を使うのをやめろ」・Zenn の csvdiff・CSV 比較名人・SmoothCSV の比較）
8. **SQL での問い合わせ**
9. **重複の削除と表記の正規化**（空白・全角半角・大文字小文字）
10. **列の操作**（名前の変更・並べ替え・テンプレートへの割り当て）
11. **キーボードでの操作・見出しの固定・ダーク / ライト**（HN 37140159）
12. **統計とプロファイル**（値の頻度・欠損の数・型の推し量り）

## 3. パワーツールから GUI に取り入れられそうな機能

| 元 | 機能 | GUI での形 |
| --- | --- | --- |
| VisiData | 頻度表・列の要約・melt / pivot・正規表現による分割と抽出・操作の記録と再実行・元に戻す | 列の見出しから「値の分布」パネル。操作履歴をマクロとして保存・再実行 |
| OpenRefine | ファセット（値・数の範囲・空欄）・クラスタリング（fingerprint・n-gram・音の近さ・Levenshtein）・fill down / blank down・複数の値を含むセルの分割と結合・操作履歴の JSON を他のファイルに適用 | 表記ゆれを候補ごとにまとめて一括で統合するダイアログ・サイドバーで絞り込む・前処理の手順の再利用 |
| Miller | fill-down・fill-empty・unsparsify・nest / explode・reshape・step（前の行との差）・merge-fields・fraction（構成比）・top・count-distinct・clean-whitespace・latin1-to-utf8・sec2gmt・regularize | 列のメニューに「下へ埋める」「縦持ち ⇔ 横持ち」「前の行との差・累計・構成比を列として足す」 |
| csvkit / xsv / qsv | stats・frequency・schema / sniff・dedup・sample・slice・split・join・explode / implode・transpose・datefmt・validate（JSON Schema）・sqlp・geocode・fetch | 2 ファイルの結合のウィザード・ファイルの分割・抜き取って表示・スキーマに沿った検証 |
| DuckDB read_csv | sniffer・列ごとの型・dateformat・null_padding・ignore_errors / store_rejects・skip・gzip / zstd の直接読み | 開く前の設定画面（判別の結果を示して直せる）・壊れた行を「隔離のタブ」に・`.gz` をそのまま開く |
| Datasette | ファセットの候補の提示・日付単位のファセット・絞り込みの状態を URL で共有・CSV / JSON の書き出し | 「この列で絞り込めます」の提案・絞り込み条件の保存と共有 |
| Excel から移る人 | 集計行・ピボット・条件付き書式・入力規則・Power Query の型を指定した取り込み・ルールが崩れたら警告 | 列ごとの検証ルールと違反の強調・簡単なピボット・集計行 |
| 差分の道具 | キー列で突き合わせる差分（追加・削除・変更をセル単位で） | 2 ファイルを並べ、変更点だけで絞り込む |

## 4. 日本に特有の要望

- **Shift_JIS / CP932 が前提の周辺システムがまだ多い**（弥生会計・ヤマトの B2 クラウド）。一方、自治体のオープンデータと国の仕様は UTF-8（BOM 付き）。**同じファイルを用途で両方に書き出す**必要
- **Shift_JIS で表せない文字の検出と置換**（㈱・①・はしご高の「髙」・絵文字は `?` に化ける）→ 保存の前に警告し置換を提案
- **全角と半角の統一**（半角カナ・全角英数・全角空白。弥生では「ﾔﾖｲ」と「ヤヨイ」が別物）
- **桁数（バイト数）の制限**（弥生の摘要は半角 64 桁 = 全角 32 文字で切り捨て）→ 全角を 2 と数える桁数の表示
- **0 落ちと日付化けの防止**（郵便番号・電話番号・JAN・番地の「1-2」）— 知恵袋の典型
- **和暦と西暦の変換**・オープンデータの「1 セル 1 データ」の検査
- **全項目を `"` で囲んで保存したい**（取り込み側が要求。Excel ではできず、置換・VBA・PowerShell で回避）
- 国内の競合: Cassava Editor（無料・巨大ファイルは苦手）・SmoothCSV 3（国産・SQL・差分・3 OS・数式なし）・EmEditor（GB 級・Windows 専用・UI が古く価格が高いという声）・CSV 比較名人
- EC: 楽天のコントロールカラム（n / u / d）の誤指定の事故・「空欄で空に上書き」・テンプレートの列の順の固定

## 5. 出典

Hacker News:

- https://news.ycombinator.com/item?id=14919149 ・ https://news.ycombinator.com/item?id=37140159 ・ https://news.ycombinator.com/item?id=44537558 ・ https://news.ycombinator.com/item?id=45040323 ・ https://news.ycombinator.com/item?id=33611431 ・ https://news.ycombinator.com/item?id=9935357

レビュー:

- https://alternativeto.net/software/modern-csv/about ・ https://www.capterra.com/p/210331/EmEditor/reviews/ ・ https://www.g2.com/products/emeditor/reviews?qs=pros-and-cons ・ https://smoothcsv.com/blog/best-csv-editors-in-2026

Excel の自動変換:

- https://support.microsoft.com/en-us/excel/set-automatic-data-conversions ・ https://learn.microsoft.com/en-us/answers/questions/5296142/excel-converts-long-numbers-to-scientific-notation ・ https://www.mrexcel.com/board/threads/losing-upc-code-leading-zeros-when-converting-to-csv.570887/ ・ http://support.sellbrite.com/en/articles/3367331-how-to-prevent-scientific-notation-in-the-upc-column-of-your-product-csv-file ・ https://leaprows.com/en/blog/excel-row-limit-csv-workaround

EC:

- https://community.shopify.com/t/excel-export-file-problem-with-special-characters-shows-garbage-instead/63801 ・ https://cartworks.io/blogs/shopify/why-shopify-csv-import-keeps-failing ・ https://rakurip.com/ec/rakuten-csv-error/ ・ https://united-works.com/rakuten-csv-error/ ・ https://note.com/smile_comfort/n/n2f4007cc0f86

研究:

- https://www.progress.org.uk/human-genes-renamed-as-microsoft-excel-reads-them-as-dates/ ・ https://www.ncbi.nlm.nih.gov/pmc/articles/PMC9325790/ ・ https://www.biostars.org/p/298429/

経理・銀行:

- https://www.statementedge.com/blog/xero-bank-statement-import-csv-format ・ https://snyp.ai/blog/csv-format-bank-statement ・ https://support.yayoi-kk.co.jp/faq_Subcontents.html?page_id=18043 ・ https://support.yayoi-kk.co.jp/subcontents.html?page_id=18545 ・ https://katoh-tax.com/2021/03/25/yayoi-csv-import-error/ ・ https://valmung.com/blog/yayoi-csv-import-error

CRM:

- https://splitforge.app/blog/mailchimp-csv-import-errors-fix ・ https://www.importcsv.com/blog/hubspot-import-contacts-csv

ローカライズ:

- https://www.linkedin.com/pulse/csv-localization-when-why-how-use-place-excel-gergely-vandor ・ https://dev.to/circobit/why-your-exported-table-has-weird-characters-and-how-to-fix-it-2lbk

GIS:

- https://github.com/qgis/QGIS/issues/57156 ・ https://manual.giscloud.com/knowledge-base/how-to-upload-your-csv-xls/

オープンデータ:

- https://github.com/frictionlessdata/frictionless-py/issues/803 ・ https://op.europa.eu/webpub/op/data-quality-guidelines/en/ ・ https://www.city.komae.tokyo.jp/index.cfm/50,125501,c,html/125501/20221112-145333.pdf ・ https://www.soumu.go.jp/main_content/000723626.pdf

国内（文字コード・0 落ち・引用・差分・比較）:

- https://detail.chiebukuro.yahoo.co.jp/qa/question_detail/q12254459552 ・ https://detail.chiebukuro.yahoo.co.jp/qa/question_detail/q14284208304 ・ https://teratail.com/questions/qoeya8n4se00wh ・ https://qiita.com/kourak_utiloop/items/fba35ebf46a1b10714e5 ・ https://b-faq.kuronekoyamato.co.jp/app/answers/detail/a_id/413/ ・ https://forest.watch.impress.co.jp/docs/serial/exceltips/1290315.html
- https://qiita.com/sen-ltd/items/a895cf3526dff5dabbc8 ・ https://zenn.dev/kesu/articles/csvdiff-intro ・ https://nodakoubou.net/csvdiff.html
- https://dev.classmethod.jp/articles/smoothcsv-3/ ・ https://goqsystem.com/blog/18505

パワーツール・ログ・テストデータ:

- https://miller.readthedocs.io/en/latest/reference-verbs/ ・ https://github.com/dathere/qsv ・ https://duckdb.org/docs/2.0/data/csv/overview.html ・ https://openrefine.org/docs/manual/cellediting ・ https://www.visidata.org/docs/ ・ https://simonwillison.net/2018/May/20/datasette-facets/
- https://fast.io/resources/large-log-file-viewer-guide/ ・ https://beantoolbox.com/tools/fake-data-generator
