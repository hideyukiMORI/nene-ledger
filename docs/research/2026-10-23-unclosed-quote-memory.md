# 閉じない引用符と記憶 — 文献・他の製品・専門家の討議

> Issue #13 / 2026-10-23。Issue #11 の残るリスク（閉じない引用符があると、索引を作るときにそのレコードの全体を記憶に持つ）への解き方を、文献と他の製品から探した記録。**決定ではない。** 実装は Issue #13、見せ方は [ADR 0018（提案）](../adr/0018-unclosed-quote-presentation.md)。

## 1. 問題は 2 つに分かれる

| | 何が起きるか | 性質 |
| --- | --- | --- |
| A. 索引の記憶 | 確定しないレコードのバイト列を持ち越して切り直すので、閉じない引用符が 2 行目にあると、ファイルの残り全部を記憶に持つ | 実装の問題。規則を変えずに直せる |
| B. 見せ方 | 規則（RFC 4180）では、閉じない引用符から入力の終わりまでが 1 つのセルになる。範囲の取り出しもそのセル全部を読む | 仕様の問題。決める必要がある |

## 2. 文献と他の製品

| 出典 | 要点 |
| --- | --- |
| csv-core（A. Gallant） | 切り出しを表で動く状態機械（DFA）にし、使う記憶はスタックの数百バイト。どこで中断しても再開でき、塊の境目では状態だけを持ち越す |
| simdjson（Langdale・Lemire, VLDB Journal 2019） | 64 バイトの塊ごとに「引用符の中か」の 1 ビットと「直前がエスケープか」の 1 ビットだけを次へ渡す。引用の範囲は carry-less multiply の prefix XOR で求める |
| ParPaRaw（Stehle・Jacobsen, PVLDB 2020） | 核は DFA。塊ごとに全部の始まりの状態について終わりの状態を求め、prefix scan でつないで正しい始まりの状態を決める |
| Ge ほか（SIGMOD 2019） | 引用符の前後の文字の並び（`x"` か `"x` か）から、塊の始まりが引用の中か外かを局所的に推し量る。失敗すれば 2 パスに落ちる |
| Ellmann・Neumann（PVLDB Vol.19） | 推し量りを「前提の集合」に一般化し、外れたら塊だけを切り直す。各塊は自分が始めたレコードを、境目を越えて読み切る |
| Python の `csv` | セルの長さの上限 131,072。閉じない引用符がファイルの残りを飲み込むのを防ぐ役も持つ（上限で守る対症療法） |
| DuckDB | 既定は厳格。閉じない引用符は「Value with unterminated quote found」と位置付きで報告。緩める・飛ばす・捨てた行を記録するのは明示の設定。行の長さの上限（`max_line_size`）もある |
| univocity | 閉じない引用符の扱いを選べる（区切りまでを引用の無い値とみなす・誤りにする・ほか）。セルの文字数の上限 |
| EmEditor | 「引用符内の改行を許可する」と「セル内の最大改行数」で、閉じない引用符の被害の範囲を絞る |

## 3. 専門家ペルソナの討議

| ペルソナ | 主張 |
| --- | --- |
| ① パーサーの実装者（csv-core の流儀） | 持ち越すのはバイト列でなく状態。索引に要るのはレコードの始まりだけで、セルの中身は要らない。各バイトを 1 回だけ見れば切り直しも消える |
| ② DB エンジンの開発者（DuckDB・Pollock） | 規則の正本は厳格に保つ。閉じない引用符は位置付きで報告する。緩めた読み方は明示の設定にし、黙って直さない |
| ③ 並列・SIMD の研究者 | 状態機械を今入れれば、SIMD（引用符の 1 ビットの持ち越し）にも並列（状態の遷移をつなぐ）にもそのまま伸ばせる |
| ④ 業務の利用者（経理・EmEditor の利用者） | 1 GB のセルを見せられても困る。どこで壊れたかを示し、その引用符を文字として読み直して開けるようにしてほしい。元のバイト列は変えないこと |
| ⑤ セキュリティ担当 | 上限の設定で守るのでなく、作りとして上限の無い記憶を持たない。作り込まれたファイル 1 つで落ちる経路を残さない |

②と④が対立した（厳格に／とにかく開きたい）。まとまった形: **正本の規則は厳格なまま。読み直しは利用者が選ぶ別の規則として core に置き、版を付ける。どちらの規則でも記憶が一定になるよう、①⑤の作り（状態とバイト位置だけを持つ）を土台にする。**

## 4. 推奨（施主の了承 2026-10-23）

1. 索引の持ち越しを状態機械にする（規則は変えない・Issue #13）
2. 範囲の取り出しに、1 レコードの読む量の上限を置く（Issue #13）
3. 閉じない引用符の見せ方は ADR で決める（[ADR 0018（提案）](../adr/0018-unclosed-quote-presentation.md)）
4. SIMD・並列は 1 の状態機械を伸ばす（後で）

## 出典

- [Rust and CSV parsing — Andrew Gallant](https://burntsushi.net/csv/)・[csv_core::Reader — docs.rs](https://docs.rs/csv-core/latest/csv_core/struct.Reader.html)
- [Parsing Gigabytes of JSON per Second — branchfree.org](https://branchfree.org/2019/02/25/paper-parsing-gigabytes-of-json-per-second/)・[Fast CSV processing with SIMD — nullprogram](https://nullprogram.com/blog/2021/12/04/)
- [ParPaRaw（arXiv 1905.13415）](https://arxiv.org/abs/1905.13415)
- [Speculative Distributed CSV Data Parsing for Big Data Analytics（SIGMOD 2019）](https://dl.acm.org/doi/abs/10.1145/3299869.3319898)
- [One Pass to Parse Them All: Fused Parallel CSV Processing（PVLDB Vol.19）](https://www.vldb.org/pvldb/vol19/p3579-ellmann.pdf)
- [DuckDB's CSV Reader and the Pollock Robustness Benchmark](https://duckdb.org/2025/04/16/duckdb-csv-pollock-benchmark)・[max_line_size の議論](https://github.com/duckdb/duckdb/discussions/14199)
- [Handling CSV files with wide columns in Python — Simon Willison](https://til.simonwillison.net/python/csv-error-column-too-large)
- [UnescapedQuoteHandling — univocity-parsers](http://docs.univocity.com/parsers/2.7.3/com/univocity/parsers/csv/UnescapedQuoteHandling.html)
- [CSV Formats — EmEditor Help](https://help.emeditor.com/en/dlg/customize/csv/index.html)
