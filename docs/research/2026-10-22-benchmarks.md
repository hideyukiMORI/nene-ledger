# 調査: CSV エディターと CSV 処理の道具の速さの比較

> Issue #3 / 2026-10-22。Web の文献調査の記録。※要確認は、検索結果の要約でしか確かめていない数値。
> 自前の測定（QLT-014・ADR 0007）の設計の材料にする。

**結論: GUI の CSV エディターを同じ機械・同じファイルで比べた第三者の測定は見つからなかった。** GUI の数値はほぼすべてメーカー自身の発表か、競合のブログ。CLI とエンジンには、方法を公開した再現できる測定が多い。

## 1. 見つかった測定

### GUI のエディター・ビューアー

| 道具 | 作業 | データ | 結果 | 機械 | 時期 | 出典 |
| --- | --- | --- | --- | --- | --- | --- |
| EmEditor v25.2 | 開く | 1 GB（1,278 万行）/ 50 GB（6.39 億行） | 0.113 秒 / 20.8 秒。読み込み中も閲覧・編集できる | Ryzen 9 9950X・64 GB・SSD・Win11・3 回の平均 | 2025 年頃 | メーカー |
| EmEditor v25.2 | 並べ替え・重複行の削除・検索 | 1 GB | 並べ替え 0.619 秒（v14.8 は 74.7 秒）・重複削除 0.031 秒・全一致の選択 0.688 秒 | 同上 | 同上 | メーカー |
| EmEditor v25.2 | CSV の列の並べ替え・頻出文字列 | CDC の実データ 15.2 GB（1.06 億件） | 並べ替え 7.34 秒・頻出 52.4 秒 | 同上 | 同上 | メーカー |
| EmEditor | 開く・置換・並べ替え・重複削除 | 10 GB（ランダムな ASCII・1.28 億行） | 1.07 / 0.27 / 7.27 / 4.29 秒 | 不明 | 現行のトップ | メーカー ※要確認 |
| EmEditor 対 Excel | 開く・すべて置換・並べ替え | 123 MB（70 万行） | EmEditor 0.185 / 0.031 / 0.173 秒、Excel 7.31 / 2.36 / 2.39 秒 | i9-11900K | 2023 年 | メーカーの PDF ※要確認 |
| EmEditor 対 大容量向けエディター 6 製品 | 開く・検索・置換・並べ替え・保存 | 不明 | 全項目で 2〜187 倍速い | 不明 | 2023 年頃 | メーカー |
| Modern CSV | 開く | 100 MB | Excel より最大 11 倍速い。読み取り専用ではメモリがファイルの数分の一 | 不明 | — | メーカーの主張 |
| Tad（DuckDB）対 Excel | 開く | 230 MB（45 万行） | Tad 5 秒未満・Excel 約 33 秒 | 2019 年の MacBook Pro | 2022 年 | 開発者 ※要確認 |
| SmoothCSV | 開く | 100 万行 | 約 3 秒（別記事で 120 万行・約 10 秒）。Windows で約 640 MB が上限 | 不明 | 2025〜26 年 | メーカー・競合のブログ |
| Tablecruncher | 開く | 100 MB・2 GB（1,500 万行） | 「数秒」。2 GB まで | 不明 | — | プロジェクトの主張 |
| Gigasheet（クラウド） | 表示・SUM・書き出し | 141 GB（7.98 億行） | 表示まで 10 秒以内・SUM 約 8 秒・書き出し約 1.5 時間（取り込みの時間は含まない） | クラウド | 2023 年頃 | 自社 |
| Excel 2021 | 開く | 1.2 GB（850 万行） | 14 分・メモリ 6.8 GB。1,048,576 行を超えた分は捨てる | 16 GB のノート | 2025 年頃 | 第三者のブログ ※要確認 |
| Delimit・Ron's Data Edit | — | — | 上限の主張だけ（Delimit は 20 億行・200 万列） | — | — | メーカー |

### CLI・エンジン・パーサー

| 道具 | 作業 | データ | 結果 | 機械 | 時期 | 出典 |
| --- | --- | --- | --- | --- | --- | --- |
| xsv | count・index・sort・stats・search | worldcitiespop 100 万行（約 45 MB） | count 0.11 秒（414 MB/s）・sort 2.18 秒・stats 1.09 秒。索引で slice が 0.08 → 0.01 秒 | i7-6900K | 2017 年頃 | 公式 BENCHMARKS.md |
| qsv 0.20 対 xsv 0.13 | 46 以上のコマンド | NYC 311 の 512 MB（100 万行 × 41 列） | count 0.09 秒・索引つき 0.01 秒。stats は xsv より遅い（計算する項目が多い） | Ryzen 7 4800H の VM | 2021 年頃 | 公式 |
| qsv | 索引の作成 | NYC 311 の 15 GB（2,800 万行） | 14 秒 | 不明 | — | 二次情報 ※要確認 |
| zsv・xsv/qsv・xan・DuckDB・Polars | count・select | worldcitiespop × 3 の 433 MB | メモリ: zsv 1.5 MB・xsv 4 MB・DuckDB 76 MB・Polars 475 MB。速さは zsv の並列が最速 | M3 Pro | 2026-03 | zsv 自身 |
| tsv-utils・xsv・csvtk・awk ほか | 絞り込み・列の選択・結合・統計 | 4.8 GB・2.7 GB・1.7 GB | 正規表現の絞り込みは xsv 7.03 秒が最速、ほかの多くは tsv-utils | Mac mini・Linux 16 vCPU | 2018 年 | eBay |
| csvkit 対 xsv | stats | 3 万行 × 400 列 | 4 分 16 秒 対 1.8 秒 | 不明 | — | Faraday 社 |
| DuckDB の並列リーダー | 走査 | 1.1 GB | 約 0.5 秒 | M3 Max | 2025 年頃 | MotherDuck ※要確認 |
| DuckDB の sniffer | 方言の判別と読み込み | NYC のデータ | 判別 0.11 秒・読み込み 2.43 秒（判別は約 4%） | 不明 | 2023 年 | DuckDB 公式 |
| DuckDB・Polars | 1BRC（10 億行・約 14 GB の集計） | 約 14 GB | DuckDB 23 秒・Polars 27 秒（M1 Max）。別の検証で Polars のストリーミング 9.8 秒・DuckDB 12.7 秒 | 検証で違う | 2024 年 | コミュニティ ※要確認 |
| db-benchmark | group by 10 問・join 5 問 | 0.5 / 5 / 50 GB | 50 GB の join を完走したのは 2 製品。1 回目コールド・2 回目ホット | AWS c6id.metal | 2023 年〜 | 公式 |
| fread・readr・read.csv（R） | 読み込み | 5.34 GB | 1.4 分・3.4 分・8.1 分 | 不明 | 2019 年頃 | 個人ブログ |
| clickhouse-local 対 DuckDB | 集計 | 20 億行（gzip の CSV） | 11 秒 対 30 秒。ただし事前に Parquet に変換しており同条件ではない | MacBook Pro | 2023 年頃 | Vantage 社 |
| simdcsv（C++） | 解析 | nfl.csv | 約 5.5 GB/s（AVX2）。MSVC では 0.65 GB/s | x86 | 2019〜20 年 | GitHub ※要確認 |
| minio/simdcsv（Go） | 解析 | 3 種 | 832 / 563 / 1008 MB/s（標準の 4〜5 倍） | c5.12xlarge | 2020 年頃 | 公式 |
| Sep（.NET） | 行の列挙 | NuGet の資産データ | 最大 21 GB/s（多スレッド） | Ryzen 9 9950X | 2024〜25 年 | Sep 自身 |
| uDSV（JS） | 解析 | 合成と実データ | PapaParse の 1〜5 倍（数値だけの合成で 251 対 74 MiB/s） | Ryzen 7 | 2023 年 | uDSV 自身 |

## 2. 測るべき指標と、測り方

GUI のエディター:

- **最初の画面までの時間**と**全体の読み込みが終わるまでの時間**を分ける（EmEditor は読み込み中も編集でき、Gigasheet の「10 秒」は取り込みを含まない。分けないと有利に見える）
- ピークのメモリ（RSS）と、ファイルの大きさに対する倍率
- 並べ替え・絞り込み・検索・置換・重複削除・**保存**の時間
- 操作中の応答（スクロールの引っかかり・入力から反応までの遅れ）
- 扱える最大の大きさ（開けたかどうかを打ち切りの時間で判定。EmEditor の 2009 年の比較は 10 分）

CLI・エンジン: 処理の速さ（MB/s・行/秒）・実時間・最大 RSS（`/usr/bin/time`）・スレッド数による伸び・コールドとホットの区別。

繰り返し: EmEditor は 3 回の平均、eBay は 25〜35 回の 10 パーセンタイル、db-benchmark はコールドとホットの 2 回。

データ: 合成（ランダムな ASCII・数だけ）と実データ（CDC・NYC 311・タクシー・worldcitiespop）の両方。列の幅・引用符・埋め込みの改行・文字コード・列の数で結果が大きく変わる（ファイルの形で勝者が変わると eBay が明記）。

公平さ: 設定と出力をそろえる（PapaParse は header の有無で約 3 倍）。機能の差を明記する（qsv の stats は項目が多い・Excel は 100 万行を超えた分を捨てる）。**正しく読めるか**も測る（Pollock で DuckDB は 99.61%・自動判別で約 91%）。

よく使われるデータ: worldcitiespop・NYC タクシー / NYC 311・1BRC の measurements.txt・db-benchmark の G1（生成スクリプトあり）・HEPMASS・nfl.csv・CDC の COVID（15 GB）・ClickBench の hits。

## 3. 道具が挙げる速さの手段

- **SIMD の解析**: 引用符の内外を分岐なしの prefix-XOR で判定（simdcsv・zsv・Sep・EmEditor）。AVX2 / SSE2 / NEON
- **並列**: ファイルを塊に切って並列に読む（DuckDB は 32 MB 単位）。塊の境目が引用符の中にある場合の扱いが難所
- **索引**: qsv / xsv は行の位置を `.idx` に保存し、count と飛び先の読みがほぼ一瞬
- **遅延の解析**: vroom は要る所だけを後で解析。EmEditor は読み込み中も編集でき、Modern CSV の読み取り専用はメモリを抑える
- **見えている所だけ描く**: Tad
- **ディスクへの退避**: EmEditor は 248 GB まで一時ファイルで扱う。qsv の extsort / extdedup。DuckDB の集計もメモリを超えるとディスクへ
- **組み込みの DB に取り込む**: Tad（DuckDB）・Gigasheet / Row Zero（サーバー側）
- **mmap**: 明記した情報源は見つからなかった ※要確認
- **列指向への変換**: Parquet にすると CSV のままより 22〜60 倍速いという報告

## 4. 空白地帯: GUI のエディターについて誰も測っていないこと

1. 同じ機械・同じファイルでの GUI エディター同士の第三者の比較（SmoothCSV の比較記事自身が「同一条件ではない」と明記）
2. 読み込み中・操作中の応答（スクロール・飛ぶ・セルの編集の遅れ）の数値
3. **巨大なファイルを編集して保存する往復の時間**（1 セルを変えて保存に何秒か。Modern CSV は大きなファイルを読み取り専用で開くので直接は保存できない）
4. 正しく読めるか（引用符・埋め込みの改行・Shift_JIS・UTF-16・BOM・改行の混在）を GUI で Pollock のように検証したもの
5. ファイルの大きさを変えながらのメモリの倍率
6. Mac・Linux の GUI エディターの実測（EmEditor と Delimit は Windows 専用）
7. 並べ替え・絞り込み・集計を、CLI・エンジン（DuckDB・qsv）と GUI で同じデータで比べたもの
8. GUI のコールドとウォームの区別
9. ファイルの形（列が多い・行が長い・引用符が多い・巨大なセル）の影響

→ 本製品の測定は、ここを埋める形で設計すると比較の価値が出る。

## 5. 出典

GUI:

- https://www.emeditor.com/text-editor-features/large-file-support/optimized-sort/
- https://www.emeditor.com/journal/text-editor-speed-comparison-for-large-files/
- https://www.emeditor.com/journal/text-editor-speed-comparison/
- https://download.emeditor.info/doc/working-with-csv.pdf
- https://www.emeditor.com/
- https://www.moderncsv.com/
- https://smoothcsv.com/blog/best-csv-editors-in-2026
- https://www.typo.monster/blog/best-large-csv-editor-comparison
- https://leaprows.com/en/blog/csv-editor-tools-comparison
- https://rowzero.com/blog/best-csv-editor
- https://csv-x.com/blog/large-csv-files-handling/
- https://github.com/duckdb/duckdb/discussions/3566
- https://www.tadviewer.com/
- https://www.gigasheet.com/post/big-data-spreadsheet
- https://delimitsoftware.com/
- https://www.ronsplace.ca/products/ronsdataedit/history

CLI・エンジン:

- https://github.com/BurntSushi/xsv/blob/master/BENCHMARKS.md
- https://github.com/dathere/qsv/blob/master/docs/BENCHMARKS.md
- https://qsv.dathere.com/benchmarks
- https://github.com/liquidaty/zsv/blob/main/app/benchmark/README.md
- https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2018.md
- https://faraday.ai/blog/how-we-made-our-csv-processing-142x-faster
- https://news.ycombinator.com/item?id=31354538
- https://duckdb.org/2023/10/27/csv-sniffer
- https://github.com/duckdb/duckdb/pull/5194
- https://duckdb.org/2024/12/05/csv-files-dethroning-parquet-or-not
- https://duckdb.org/2025/04/16/duckdb-csv-pollock-benchmark
- https://motherduck.com/learn/csv-vs-parquet-benchmark/
- https://duckdb.org/2023/11/03/db-benchmark-update
- https://duckdblabs.github.io/db-benchmark/
- https://github.com/joseph-cavarretta/billion-rows-challenge
- https://www.vantage.sh/blog/clickhouse-local-vs-duckdb
- https://cran.r-project.org/web/packages/vroom/vignettes/benchmarks.html
- https://jozef.io/r917-fread-comparisons/

パーサー:

- https://github.com/geofflangdale/simdcsv
- https://github.com/minio/simdcsv
- https://github.com/joelverhagen/NCsvPerf
- https://nietras.com/Sep/
- https://github.com/leeoniya/uDSV/tree/main/bench
- https://github.com/uniVocity/csv-parsers-comparison
