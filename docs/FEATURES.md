# 機能の一覧 — NeNe Ledger

> Status: normative（規範）/ 2026-10-22 初版（Issue #4・[ADR 0008](adr/0008-cover-all-features-in-phases.md)）。Issue #6 で窓・表示言語・配色を足した
> **何を作るかの細目の正本。** 大きな約束は [SPECIFICATION.md](../SPECIFICATION.md) の FR にあり、ここの各機能はどれか 1 つの FR に属する。
> 起こした経緯と出典は [docs/research/](research/2026-10-22-feature-catalog.md)（Issue #3）。
> 🔴 本書の `Issue #N`（#1〜#68）は公開前のローカル履歴の番号（[ADR 0021](adr/0021-public-repository-on-github.md)）。証拠は [gate-proofs](quality/gate-proofs.md) の節と [benchmarks](quality/benchmarks.md) の節で引く。

## 0. 読み方

- **番号**（`OPEN-01` など）は一度付けたら変えない。消すときは行を残して状態を「取り下げ」にする
- **フェーズ**: 1 = 約束の芯（無いと SPECIFICATION の約束が成り立たない）/ 2 = 初版に続けて入れる / 3 = 後の版 / — = やらない（第 25 節）
- **状態**: `未着手` / `実装済み` / `確認済み` / `取り下げ` / `やらない`。取り下げた行は、機能の文に行き先を書く
  - 🔴 **`確認済み` は、テストか実測の記録（gate-proofs・性能の記録・実機の確認）を名指しできるものだけ。** 書いてあるから動く、とは書かない（QLT-013 / QLT-014）
  - 「全部カバーした」は、フェーズ 1〜3 の全行が `確認済み` になったときだけ言う
- **競合**: Issue #3 の調査で、競合で持つものの多さ（多 / 一部 / 少 / 無）。無は差別化の候補
- **根拠**: research の文書の項目（文献・速さ・競合・要望・討議）か ADR
- FR-022（Windows と X11 で同じ振る舞い）は全機能にかかる横断の約束で、専属の行を持たない

| フェーズ | 件数 |
| --- | --- |
| 1 | 124 |
| 2 | 70 |
| 3 | 26 |
| — | 1 |
| 合計（取り下げ 2 を除く） | 221 |

## 1. 開く・方言・文字コード（OPEN）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| OPEN-01 | 区切り文字の判別（`,` タブ `;` `\|` `^` ほか）と手動の指定 | 1 | 確認済み | FR-001 | 多 | 文献・落とし穴 1・判別（`,` タブ `;` `\|` `^`）は Issue #10・#23、Pollock で正しく読めた率は benchmarks 第 1 節（Issue #24・#25）。手動の指定はパレットの「区切りを選んで開き直す…」（Issue #52・gate-proofs 第 39 節。カンマ・タブ・セミコロン・縦棒・空白・`^`・コロン）。Windows は Issue #59（gate-proofs 第 46 節）。任意の字は、区切りの段で英数字でない字を 1 つ打つ（Issue #67・gate-proofs 第 49 節）。施主の Windows の実機で `~` を確かめた（gate-proofs 第 99 節） |
| OPEN-02 | 判別の候補を確信度とプレビュー付きで示し、すぐ選び直せる | 1 | 確認済み | FR-001 | 少 | 文献・CleverCSV・候補（点数の順）と確信の有無は core の判別（Issue #10・#23）。読み方の候補をその場で試して選び直すのは Issue #52（gate-proofs 第 39 節）。候補を点数と列の数つきで並べて試し、開き直す段（パレットの「判別の候補から選んで開き直す…」）と、確信が無いときの状態の行の知らせは GitHub Issue #3（gate-proofs 第 52 節）。施主の Windows の実機で確かめた（gate-proofs 第 96 節）。確信の判じ方は GitHub Issue #24 で直した（引用の逃がし方だけが違う候補は競わない・点の下限 200・空や 1 列のファイルでは知らせない） |
| OPEN-03 | 任意の文字・複数文字の区切り | 2 | 未着手 | FR-001 | 一部 | 競合・Delimit / EmEditor |
| OPEN-04 | 固定幅のテキストを開く | 3 | 未着手 | FR-001 | 一部 | 競合・LibreOffice / Delimit |
| OPEN-05 | 引用符の流派（`""` の二重・`\` のエスケープ・引用符なし・シングルクォート）の判別と指定 | 1 | 確認済み | FR-001 | 少 | 文献・落とし穴 3・判別の候補は `""`・引用の中の `\`・引用なしの `\` のエスケープ・引用符なし・シングルクォート（Issue #9・#10・#23）。指定はパレットの「引用符の扱いを選んで開き直す…」（Issue #52・gate-proofs 第 39 節）。Windows は Issue #59（gate-proofs 第 46 節）。引用符の字（シングルクォート）を選ぶことと、状態の行に引用の扱いを出すこと（既定でないときだけ）は Issue #68（gate-proofs 第 50 節・施主の Windows の実機で確かめた・gate-proofs 第 96 節）。引用の中に区切りが無くても `\` のエスケープを選ぶのは Issue #60（閉じた引用符の後ろに文字が続いたセルを減点・core のテスト・Pollock の点は変わらず） |
| OPEN-06 | TSV の 3 流派（CSV と同じ引用・バックスラッシュのエスケープ・エスケープなし） | 1 | 確認済み | FR-001 | 少 | 文献・IANA TSV・3 流派の切り方は core のテスト（Issue #9 の `tsv_with_csv_style_quotes` など 3 つ）で、判別の候補にも入っている（Issue #10）。窓では UTF-16LE のタブ区切り（gate-proofs 第 27 節）。Windows は Issue #59（gate-proofs 第 46 節）。流派を見分ける判別は Issue #66 のテスト（core の `tsv_with_csv_style_quotes_is_read_with_doubled_quotes`・`tsv_with_escaped_tabs_is_read_with_backslash_escapes`・`tsv_with_stray_quotes_is_read_without_quoting`。逃がした区切りの無い `\t` の字は値を変えずに読む: `backslash_letters_without_an_escaped_delimiter_keep_their_bytes`）。指定は OPEN-05 |
| OPEN-07 | 引用符の中の改行・区切り・`""` を正しく読む（RFC 4180） | 1 | 確認済み | FR-001 | 多 | 文献・RFC 4180・切り方の正本は core の参照実装（Issue #9）。scan の索引と範囲の取り出しは、塊をまたぐ引用の中の改行を乱数の入力で参照実装と突き合わせた（Issue #11・#13）。窓では引用の中の区切り・`""`・改行（¶）を見た（gate-proofs 第 13 節）。Pollock は benchmarks 第 1 節。Windows は Issue #59（gate-proofs 第 46 節） |
| OPEN-08 | 改行（CRLF / LF / CR）の判別と、混在の検出・報告 | 1 | 確認済み | FR-002 | 一部 | 文献・落とし穴 5・要望・経理・改行の内訳と混在は判別の標本（先頭 64 KiB）で数え（Issue #10）、状態の行に「改行が混在（CRLF n · LF n · CR n）」と出す（Issue #29・status_line のテスト）。ファイル全体での混在は GitHub Issue #128 で、索引を作る仕事が引用の外のレコードの改行を数え（参照実装の `Record::terminator` と同じことを scan の無作為のテストで固定）、作り終えたら状態の行をその数に替える（Xvfb・gate-proofs 第 72 節）。施主の Windows の実機で混在の知らせを確かめた（gate-proofs 第 96 節）。混ざった所へ飛ぶことと改行をそろえて書き出すこと（SAVE-03）はまだ |
| OPEN-09 | 文字コードの判別（UTF-8・BOM・UTF-16LE/BE・CP932・JIS の Shift_JIS・EUC-JP・Latin-1 / CP1252） | 1 | 未着手 | FR-002 | 一部 | 文献・落とし穴 11〜15・判別は Issue #17、復号は #18、CP932 の 2 バイト目と重なる区切りは #35（gate-proofs 第 26 節）、UTF-16 は #36（第 27 節）。JIS の Shift_JIS の区別（JP-01）と判別の精度の測定はまだ |
| OPEN-10 | 別の文字コード・方言で開き直す（一瞬で） | 1 | 確認済み | FR-001 | 一部 | 討議 F・読めないバイトを返す復号はある（Issue #18）。文字コード・区切り・引用符の扱い・見出しの行を選び、先頭をその場で試してから開き直す（索引を作り直す）のは Issue #52（gate-proofs 第 39 節）。Windows は Issue #59（gate-proofs 第 46 節） |
| OPEN-11 | 見出しの有無の判別・見出しの行数・読み飛ばす行数の指定 | 1 | 確認済み | FR-001 | 一部 | 文献・落とし穴 9・見出しの有無の推定（Issue #10）、前書きの行の判別（Issue #25・benchmarks の「Issue #25 の後」。実データの表題の行・右の空の列・段の見出しは GitHub Issue #15・benchmarks 第 2 節）、見出しの行の固定（Issue #26）、見出しの行の指定はパレットの「見出しの行を選ぶ…」（Issue #52）。見出しの行より上の前書きの行は、字と行番号を薄く描いて区別し、開いたときのカーソルはデータの 1 行目に置く（施主の判断 2026-11-08・GitHub Issue #273・application のテスト `preamble_rows_are_faded_and_the_cursor_starts_on_data`・Xvfb で確かめた）。読み飛ばす行は、見出しの行を選べばその上が前書きになる。見出しの行の数は「見出しの行を選ぶ…」で 1〜3 行を選べ、全部上に固定し、列の名前は各行の値を「 / 」でつなぐ（施主の判断 2026-11-08・ADR 0034・GitHub Issue #277・application のテスト `every_header_row_is_frozen_and_left_out_of_the_body`・`column_names_join_the_header_rows_with_a_slash`、app のテスト `every_header_row_is_left_out_of_the_count`・Xvfb で確かめ、施主の Windows の実機で 2 行の見出しを確かめた・gate-proofs 第 98 節）。前書きを薄く見せることも施主の Windows の実機で確かめた（第 99 節） |
| OPEN-12 | コメント行（`#` など）の指定。既定は無効 | 2 | 未着手 | FR-001 | 少 | 文献・落とし穴 10 |
| OPEN-13 | Excel の `sep=;` の行を方言の情報として読む | 2 | 未着手 | FR-001 | 少 | 文献・落とし穴 23 |
| OPEN-14 | 列数のそろわない行（不足・超過）を壊さず読む | 1 | 確認済み | FR-003 | 一部 | 文献・落とし穴 6・行の形は Issue #9。足りないセルは「セルが無い」の色で空のセルと分け（Issue #19・gate-proofs 第 13 節）、列の多い行が来たらその列を足す（Issue #26 のテスト）。Windows は Issue #59（gate-proofs 第 46 節）。2026-11-09：列の数が見出しと違う行の番号に注意の色と帯・詳細ペインに数（GitHub Issue #317）・範囲の集計に「列の数が違う行 n を含む」（#329）。Xvfb で確かめた・Windows の実機はまだ |
| OPEN-15 | 壊れた行（閉じていない引用符など）を元のバイト列で隔離して示し、そこへ飛べる | 1 | 未着手 | FR-023 | 少 | 討議 G・要望・DuckDB・ADR 0018・閉じない引用符の位置を覚えて（Issue #15）状態の行で知らせる（Issue #21・#23、gate-proofs 第 14・16 節）。元のバイト列での隔離・そこへ飛ぶこと・読み直しを尋ねる画面はまだ |
| OPEN-16 | 1 ファイルの中の複数の表（空行のブロック）の検出 | 2 | 未着手 | FR-001 | 無 | 文献・Pollock |
| OPEN-17 | 判別に読む量とセルの長さに上限（病的な入力で固まらない） | 1 | 確認済み | FR-001 | 無 | 文献・セキュリティ・判別は先頭 64 KiB だけを読む（Issue #10）。範囲の取り出しは 1 レコード 1 MiB で切り詰め、巨大なレコードの所で止まる（Issue #13・#23。約 300 MB の閉じない引用符でも最初の画面がすぐ出た: gate-proofs 第 16 節）。Windows は Issue #59（gate-proofs 第 46 節）・約 300 MB。切り詰めたことは詳細ペインの見出しの行（「1 MiB で切り詰め（行の全体 2.0 MiB）」）・コピーの知らせ・`--dump` の標準エラーで示す（ADR 0018 の決定 2・GitHub Issue #36・gate-proofs 第 56 節） |
| OPEN-18 | gzip / zstd の圧縮ファイルをそのまま開く | 3 | 未着手 | FR-001 | 少 | 要望・DuckDB |
| OPEN-19 | 方言の設定をファイルごとに覚える・規則で自動適用 | 2 | 未着手 | FR-001 | 少 | 競合・SmoothCSV / EmEditor |
| OPEN-20 | 方言を CSVW / Frictionless のメタデータとして読み書き（ADR 0008） | 2 | 未着手 | FR-001 | 無 | 文献・CSVW / Frictionless |
| OPEN-21 | 小数点の記号・桁区切りの地域差の設定（数として見るときだけ） | 2 | 未着手 | FR-001 | 少 | 文献・落とし穴 2・要望・GIS |

## 2. 勝手に変えない・往復の保全（KEEP）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| KEEP-01 | セルは文字列のまま。型を推し量らない（先頭の 0・日付に見える値・長い数字・指数表記） | 1 | 確認済み | FR-003 | 一部 | 要望 1・文献・遺伝子名・値は文字列のまま見せる（gate-proofs 第 13 節の `00123`・`1.20E+03`、第 27 節の `0012`）。コピー（Issue #30）と集計（Issue #42）も値を変えない。Windows は Issue #59（gate-proofs 第 46 節）。日付に見える値・遺伝子名・16 進・前後の空白などの見本は Issue #65 のテスト（app の `values_that_look_like_dates_numbers_or_genes_are_kept_as_text`・`…_written_back_unchanged`、application の `…_drawn_as_they_are`） |
| KEEP-02 | 読んで何も変えずに保存すると、バイト単位で同じファイル | 1 | 実装済み | FR-003 | 少 | 文献・全体の原則。保存の経路（store）と `--resave` の往復の実測は GitHub Issue #58（benchmarks 第 4 節）。画面からの保存（Ctrl+S）は GitHub Issue #71 から入り、2026-11-06 の #130 で書き方を一時ファイルに直に流す形に替えた後も、実データ 382 ファイルの写しの `--resave` は元と違う 0（PR #131 のコメント）。Windows の実機での往復はまだなので `確認済み` にしない |
| KEEP-03 | 触っていない行（目標はセル）は、保存で元のバイト列のまま | 1 | 実装済み | FR-003 | 少 | 要望・SmoothCSV / 討議 B・書き換えた行の触っていないセル・触っていない行・足し消しした行の前後は原本のバイト列を写す（application の `random_edits_change_only_the_edited_cells`・`random_row_insertions_and_removals_keep_the_other_rows_byte_for_byte`・`random_column_changes_rebuild_rows_with_the_same_values`・`untouched_cells_keep_bom_quotes_blanks_spaces_and_the_missing_end`・GitHub Issue #138）。1 GB で列を足した保存のバイト列は期待と `cmp` で一致（benchmarks 第 5 節） |
| KEEP-04 | BOM の有無・最終行の改行の有無・末尾の区切りを保つ | 1 | 実装済み | FR-003 | 少 | 文献・落とし穴 7 / 11・BOM・最後の行の改行の無さ・末尾の区切りは、書き換えた行でも列を足して組み立て直しても保つ（application の `untouched_cells_keep_bom_quotes_blanks_spaces_and_the_missing_end`・最初の行を消しても BOM は残るテスト・`rows_added_after_an_unterminated_last_row_start_on_a_new_line`・GitHub Issue #138） |
| KEEP-05 | 「空のセル」と「セルが無い」、引用符つきの空と無しの空を区別して保つ | 1 | 実装済み | FR-003 | 無 | 文献・落とし穴 6 / 26。例外：列の数が足りない行のセルの無い所に書くと、間の列は空のセルで埋める（ADR 0024 の施主の判断 4・5・GitHub Issue #83）・保存では引用符つきの空（`""`）と無しの空・短い行のセルの無い所を保つ（application の `untouched_cells_keep_bom_quotes_blanks_spaces_and_the_missing_end`・GitHub Issue #138）。表の中で「空のセル」と「セルが無い」を区別するのは core の `CellValue`（RS-004） |
| KEEP-06 | 見出しの重複・空を許し、勝手に改名しない（列は番号で指す） | 1 | 確認済み | FR-003 | 無 | 文献・落とし穴 8・見出しは原本のレコードのまま出し、列は位置で扱う（Issue #26・#48）。重複・空の見出しは Issue #61 のテスト（app の `duplicate_and_empty_headers_are_kept_as_they_are`・application の `duplicate_and_empty_headers_are_drawn_as_they_are_and_pointed_by_position`。2 つめの同じ名前の見出しを押すとその位置の列が選ばれ、コピーもその列） |
| KEEP-07 | 前後の空白を保つ（削るのは明示の操作） | 1 | 実装済み | FR-003 | 一部 | 文献・落とし穴 25・前後の空白は見せるときも保存でも削らない（app の `values_that_look_like_dates_numbers_or_genes_are_kept_as_text` の ` 7 `・application の `untouched_cells_keep_bom_quotes_blanks_spaces_and_the_missing_end` の ` a `・`" q "`・GitHub Issue #138）。空白を削る操作はまだ無い |
| KEEP-08 | 列ごとの「見方」（文字列 / 数 / 日付）は比較・統計・表示だけに効き、保存に効かない。読める率を示して提案するが自動で適用しない | 1 | 未着手 | FR-009 | 無 | 討議 A・文献・CSVW |

## 3. 巨大なファイルと速さ（BIG）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| BIG-01 | 大きさに依らず最初の画面がすぐ出る（先頭を切るだけ） | 1 | 確認済み | FR-004 | 一部 | 速さ・EmEditor / CSView・索引を待たずに先頭を出す（Issue #20）。巨大なレコードの所で止まる（Issue #23・gate-proofs 第 16 節の約 300 MB）。止まるのは索引がその後ろを数えていないときだけで、作り終えた索引があれば 1 MiB を超えるレコードは切り詰めて後ろも返す（GitHub Issue #13・scan の `the_records_after_a_long_closed_cell_are_returned`・`without_the_index_a_long_record_stops_the_window_at_the_limit`・`an_unclosed_quote_to_the_end_stops_at_the_limit_even_with_the_index`）。100 万行でも動く（第 36 節）。Windows は Issue #59（gate-proofs 第 46 節）・100 万行と約 300 MB。最初の画面は 91 MB でも 954 MB でも、窓が 0.04〜0.05 s・表の字が 0.10〜0.11 s で出る（release・Xvfb・GitHub Issue #258・benchmarks 第 19 節）。比較相手との比べ（BIG-11）はまだ |
| BIG-02 | 行の索引を裏で作り、進み具合を見せる。作り終わる前から閲覧できる（編集は BIG-12・Issue #51） | 1 | 確認済み | FR-004 | 少 | 速さ・EmEditor・裏の索引と進み具合は Issue #21（gate-proofs 第 14 節）、作り終える前の閲覧は Issue #22。3.8 GB の索引を作っている途中に状態の行が `索引を作っています 8%` → `18%` と進み、その間も 1 画面下へが 9〜13 ms で応える（release・Xvfb・GitHub Issue #253・benchmarks 第 17 節）。作り終える前の編集は BIG-12 に分けた（Issue #51）。Windows の実機の時間は測っていない |
| BIG-03 | 見えている行と列だけを描く（行も列も） | 1 | 確認済み | FR-004 | 多 | 文献・落とし穴 24・見えている範囲だけを索引の目印から読み（Issue #22・gate-proofs 第 15 節）、窓の外の列は描かない（Issue #19・#26）。横の移動は Issue #28（第 19 節）。Windows は Issue #59（gate-proofs 第 46 節） |
| BIG-04 | メモリはファイルの大きさでなく索引に比例（閲覧。編集の差分は BIG-13・Issue #51） | 1 | 確認済み | FR-004 | 少 | 速さ・Modern の読み取り専用・まばらな索引（Issue #11）と、持ち越すのは状態だけ（Issue #13。256 MiB の閉じない引用符で最大常駐 3,784 KB を 1 回実測）。91 MB・954 MB・3.8 GB を開いて索引を作り終えた後の記憶は 17.6〜20.4 MB で、ファイルの大きさに依らない（release・Xvfb・GitHub Issue #253・benchmarks 第 17 節）。比較相手との記憶の比べ（BIG-11）はまだ。窓の行は見えている列の範囲だけを持ち、1 行に数百万セルのファイル（2 億セル）を開いた最大の常駐は 11.7 GB → 83 MB（GitHub Issue #22・#230・benchmarks 第 13・14 節）。差分は BIG-13 |
| BIG-05 | 巨大なファイルでも編集して保存できる（読み取り専用に落とさない） | 1 | 未着手 | FR-005 | 少 | 討議 B・速さ・空白 3 |
| BIG-06 | 重い処理はすべて裏で・進み具合・中止・画面は止めない | 1 | 未着手 | FR-010 | 一部 | 要望・ETL |
| BIG-07 | 記憶に入らない並べ替え・重複削除・集計は一時ファイルに逃がす | 1 | 未着手 | FR-013 | 少 | 速さ・qsv / EmEditor |
| BIG-08 | 数万列の横長のファイル | 1 | 確認済み | FR-004 | 少 | 文献・落とし穴 24・横は見えている列だけを描く（Issue #28）。窓の行も見えている列の範囲（＋余白）だけを持ち、横に動いて外に出たら読み直す（GitHub Issue #22・ADR 0033・benchmarks 第 14 節・gate-proofs 第 90 節）。20,000 列 × 200 行を開いた最大の常駐は 383 MB → 34 MB、行の終わりへ飛んでも値が出る（Xvfb）。行の終わりへ飛ぶキー（End）は、見せる最初の列を列の数の 2 乗の手間で探していたのを直し、20,000 列で 1.92 s → 0.12 s・50,000 列で 11.47 s → 0.27 s（GitHub Issue #236・benchmarks 第 15 節）。施主の Windows の実機で、20,000 列の行の終わりへ飛んで左右に動いても値が出ることを確かめた（gate-proofs 第 91 節・メモリの使用量は実機では見ていない） |
| BIG-09 | ファイルの一部（先頭 / 末尾 / 範囲 / 抜き取り）だけを開く・見る | 2 | 未着手 | FR-004 | 少 | 要望・研究 / qsv sample |
| BIG-10 | 操作ごとの応答 500 ms 以内を目標（対話の閾値） | 1 | 確認済み | FR-004 | 無 | 文献・Rahman 2020・1 GB・10 列の表で、移動・選択・パレット・詳細ペイン・直し始め・検索の欄の 17 の操作が、キーを押してから画面が変わるまで 7〜26 ms（Xvfb・GitHub Issue #251・benchmarks 第 16 節）。列全体を裏で数えている間も 12〜19 ms で応える。重い仕事は裏で動き進み具合を見せる（ADR 0014）。Windows の実機の時間は測っていない |
| BIG-11 | 速さの測定を公表（最初の画面・索引・集計・メモリの倍率・正しく読めた率。保存の往復は BIG-14、並べ替えと絞り込みは BIG-15・Issue #51） | 1 | 未着手 | FR-004 | 無 | 速さ・空白 1〜9・正しく読めた率は Pollock で記録した（Issue #24・#25、benchmarks 第 1 節）。最初の画面・索引・集計・メモリの倍率と、比較相手（DuckDB・qsv・xsv）との比較はまだ |
| BIG-12 | 行の索引を作り終わる前から編集できる（BIG-02 から分けた） | 1 | 未着手 | FR-004 | 少 | 速さ・EmEditor・Issue #51 |
| BIG-13 | 編集してもメモリは差分に比例（全体を記憶に読み込まない・BIG-04 から分けた） | 1 | 未着手 | FR-004 | 少 | 速さ・ADR 0005・Issue #51 |
| BIG-14 | 保存の往復の時間を測って公表（BIG-11 から分けた） | 1 | 実装済み | FR-004 | 無 | 速さ・空白 3・Issue #51。保存の経路（store）と `--resave` の往復の実測は GitHub Issue #58（benchmarks 第 4 節）。画面からの保存は benchmarks 第 5 節（2026-11-06・1 GB でセル 1.04 s・行 0.99 s・列 18.23 s と記憶・GitHub Issue #130）。比較相手との同じファイルでの比べと Windows の実機はまだ |
| BIG-15 | 並べ替えと絞り込みの時間を測って公表（BIG-11 から分けた） | 1 | 未着手 | FR-004 | 無 | 速さ・Issue #51 |

## 4. 表示（VIEW）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| VIEW-01 | 見出しの行と左の列の固定 | 1 | 確認済み | FR-026 | 多 | 競合・見出しの行の固定は Issue #26（gate-proofs 第 18 節）、見出しの無い表の列の番号の行は Issue #48（第 37 節）。左の列の固定（パレットの「カーソルの列まで左に固定する」「列の固定を外す」・配置と当たり判定は core の `TableLayout::with_frozen`・固定した列の幅は本体の 2/3 まで）は GitHub Issue #4（gate-proofs 第 53 節）。施主の Windows の実機で固定した列が残ること（gate-proofs 第 96 節）と、固定した後に狭い窓で右へ動いてから広げると間の列が戻ること（GitHub Issue #265 の直し・gate-proofs 第 97 節）を確かめた。見出しの扱いを利用者が選ぶこと（Issue #48 の残るリスク 1）はまだ |
| VIEW-02 | 列の幅のドラッグ・内容に合わせた自動の幅・列の順の入れ替え（表示だけ） | 1 | 確認済み | FR-024 | 多 | 競合・開いたときに決めてスクロールで変えない自動の幅は Issue #26。列の順の入れ替え（選んだ見出しのドラッグ）は Issue #49（gate-proofs 第 38 節）。幅のドラッグ（見出しの境目・3〜200 字・Esc で戻す・列を入れ替えても付いていく・パレットで元に戻す）は Issue #64（gate-proofs 第 48 節）。施主の Windows の実機で幅と順のドラッグを確かめた（gate-proofs 第 96 節） |
| VIEW-03 | 行と列の非表示 | 2 | 未着手 | FR-026 | 一部 | 競合・Modern / EmEditor |
| VIEW-04 | セルの折り返しの切り替えと、行の高さ | 2 | 未着手 | FR-026 | 一部 | 競合・SmoothCSV |
| VIEW-05 | セルの中の改行・タブ・見えない文字（NBSP・ゼロ幅・全角空白・NUL）の可視化 | 1 | 確認済み | FR-026 | 少 | 要望・翻訳・文献・落とし穴 17・改行（¶）・タブ（→）・NUL を含む制御文字（·）は Issue #19（gate-proofs 第 13・27 節）、全角空白（□）・NBSP（␣）・ゼロ幅（¦）・行と段落の区切り（¶）は Issue #53（第 40 節）。表と詳細ペインの両方で、値は変えない。Windows の字体での印は Issue #59（gate-proofs 第 46 節）。印は字の色とセルの背景の中間の灰色で描く（背景はセルのまま・データの中の本物の `¶`・`→` は薄くしない・GitHub Issue #261・#269・application のテスト `mark_spans_cover_only_replaced_characters`・`invisible_character_marks_are_muted`・Xvfb で確かめた）。最初の行番号の字の色は施主の Windows の実機で薄さが分かりにくく（gate-proofs 第 96 節）、施主の判断（2026-11-08）で中間の灰色に替えた。施主の Windows の実機で、印が薄く本物の字と見分けられることを確かめた（gate-proofs 第 97 節） |
| VIEW-06 | 引用符の有無の表示（元のファイルでどう書かれていたか） | 2 | 未着手 | FR-026 | 少 | 競合・SmoothCSV |
| VIEW-07 | セルの詳細ペイン（長い値・改行・JSON の整形表示・文字数とバイト数） | 1 | 確認済み | FR-026 | 少 | 討議 K・要望・GIS・全文と文字数は Issue #31（gate-proofs 第 22 節）、拡大に合わせた行の数は Issue #38・#39（第 28 節）、ファイルの文字コードでのバイト数は Issue #59（core の `encoded_length`・`encoding_rs` との総当たりのテスト・gate-proofs 第 46 節）。JSON のオブジェクトか配列のセルは 2 字下げで整形して見せ、補助の行に「JSON（整形して表示）」と添える（値は変えない・切り詰めた値は整形しない・深さ 128・1 MiB まで・GitHub Issue #247・core の `json_layout`・application のテスト `the_pane_lays_out_a_json_cell`）。本文は 5 行なので、長い JSON は「〈あと N 行〉」。施主の Windows の実機で確かめた（gate-proofs 第 96 節） |
| VIEW-08 | 1 行をフォームで縦に見る・編集する（レコード表示） | 2 | 未着手 | FR-026 | 一部 | 競合・Ron's / SmoothCSV / Easy |
| VIEW-09 | 行の交互の色・列の色分け | 2 | 未着手 | FR-026 | 一部 | 競合・Rainbow / Modern |
| VIEW-10 | 数として見る列の右寄せ | 2 | 未着手 | FR-026 | 少 | 競合・Tad |
| VIEW-11 | テーマ（ライト / ダーク / 高コントラスト）・OS の明暗に従う → THEME-01〜04 と A11Y-02 に分けた | 1 | 取り下げ | FR-035 | 一部 | 要望 11・討議 I |
| VIEW-12 | ズーム（文字の大きさ） | 1 | 確認済み | FR-021 | 少 | 討議 I・Issue #37（表と詳細ペインの字・Ctrl+ホイール / Ctrl+0・ADR 0020。Ctrl++ / Ctrl+- は 2026-11-06 に行と列の足し消しへ譲った・GitHub Issue #103）。設定に残すのは Issue #40（第 29 節）。Windows は Issue #59（gate-proofs 第 46 節） |
| VIEW-13 | ステータスバーに方言（文字コード・BOM・区切り・改行・列数・行数）を常に表示 | 1 | 確認済み | FR-001 | 一部 | 要望・経理・Issue #29（gate-proofs 第 20 節。改行は判別の標本の内訳）。Windows は Issue #59（gate-proofs 第 46 節） |
| VIEW-14 | 選んだ範囲の件数・合計・平均をステータスバーに | 2 | 実装済み | FR-026 | 一部 | 競合・Excel 由来・Issue #42（gate-proofs 第 31 節・10 万行まではその場で）・Issue #43（第 32 節・超える範囲は裏で・進み具合・Esc で止める）・日本の業務の数の書き方（`1,234`・`▲100`・`(100)`・`¥1,000`・`１２３`・指数）を数として読む（GitHub Issue #41・core の `number_reading` の試験）。`%` は数でない。2026-11-09：書き方を読み替えて数にしたセルの数を添える「記号や指数を読んだ n」（GitHub Issue #319） |
| VIEW-15 | 条件付きの強調（値の規則で色を付ける） | 3 | 未着手 | FR-026 | 少 | 要望・Excel から移る人 |
| VIEW-16 | 行番号は「表示の行」と「原本の物理的な行」の両方を出せる | 2 | 未着手 | FR-026 | 無 | 文献・落とし穴 4 |
| VIEW-17 | 列の幅に収まらない値は三点リーダー（`…`）で省略して描く（値は変えない・全文は詳細ペイン） | 1 | 確認済み | FR-026 | 多 | 施主 2026-11-05・GitHub Issue #7。ペルソナ EC は必須（商品説明が 1 行に並ぶと読めない）。折り返し（VIEW-04）とは別。application の `cell_display::fitted`（頭の部分だけを数える）と `values_wider_than_the_column_end_with_an_ellipsis`・gate-proofs 第 55 節（Xvfb・列の幅のドラッグで消える）。施主の Windows の実機で確かめた（gate-proofs 第 96 節） |

## 5. 移動・選択（NAV）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| NAV-01 | 行番号・セルへ飛ぶ・先頭 / 末尾へ一瞬で | 1 | 確認済み | FR-026 | 多 | 競合・先頭 / 末尾へ（Ctrl+Home / Ctrl+End）は Issue #22・#28。スクロールバー（つまみのドラッグ・溝の押し続け）は Issue #44（gate-proofs 第 33 節）。行の番号とセル（行,列）へ飛ぶのは Ctrl+G とパレットの「行へ飛ぶ…」（Issue #54・第 41 節）。Windows は Issue #59（gate-proofs 第 46 節）。2026-11-09：Ctrl+G の欄に表の範囲と、範囲の外の番号の注意（GitHub Issue #307） |
| NAV-02 | 矩形の選択・行全体・列全体・複数範囲の選択 | 1 | 確認済み | FR-026 | 多 | 競合・矩形の選択（キーと Shift+クリック）は Issue #28（gate-proofs 第 19 節）、ドラッグは Issue #45（第 34 節）、離れた範囲は Issue #46（第 35 節）、行全体・列全体は Issue #47・#48（第 36・37 節）。Windows は Issue #59（gate-proofs 第 46 節） |
| NAV-03 | ブックマーク（行に印を付けて行き来する・印の行だけ書き出す） | 2 | 未着手 | FR-026 | 少 | 競合・Delimit / Tablecruncher |
| NAV-04 | 列の一覧から列へ飛ぶ（数万列で） | 2 | 未着手 | FR-026 | 無 | 討議 |
| NAV-05 | 2 つのペインで同じファイルの別の場所を見る（分割表示） | 3 | 未着手 | FR-026 | 少 | 要望・ログ |
| NAV-06 | 取り込み先が返した失敗の一覧（DuckDB の `reject_errors`・BigQuery の errors など）の行番号を読んで、その行へ飛ぶ | 2 | 未着手 | FR-023 | 無 | GitHub Issue #8・ペルソナ ETL |

## 6. 編集（EDIT）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| EDIT-01 | セルの書き換え（セルの中で・詳細ペインで） | 1 | 未着手 | FR-005 | 多 | ADR 0026。表の上で直すこと（打ち始め・F2・確定して動く・やめる・書き換えを表と詳細ペインとコピーに重ねる）は GitHub Issue #69（application のテスト・gate-proofs 第 57 節）。画面からの保存は GitHub Issue #71。詳細ペインで直すことはまだなので状態は変えない。ダブルクリックで直し始めるのは GitHub Issue #120（gate-proofs 第 71 節）。施主の Windows の実機で、IME の確定・F2・Tab・Delete を確かめた（gate-proofs 第 60 節）。2026-11-09：編集中の長い文字列はセルを右へ広げる（#321）・書き換えたセルの左に保存まで緑の帯（#323・#331） |
| EDIT-02 | 複数セルへの一括の入力 | 1 | 確認済み | FR-005 | 一部 | 競合・Modern / SmoothCSV。範囲を選んで打ち、Ctrl+Enter で範囲のすべてのセルに書く（GitHub Issue #203・ADR 0029 の決定 6・貼り付けの「1 つの値は範囲を埋める」と同じ道で 1 手で戻る・X11 は Xvfb で確かめた）。施主の Windows の実機で、範囲に書けて 1 手で戻ることを確かめた（gate-proofs 第 91 節） |
| EDIT-03 | 行と列の挿入・削除・複製・移動（ドラッグも） | 1 | 未着手 | FR-005 | 多 | 競合・見え方と操作は ADR 0027。行の並び（GitHub Issue #87）・保存の計画（#91）・画面の読み出し（#93）・行を足す・消す・戻す（#95・パレットから・gate-proofs 第 63 節）。列の並びと保存（#97）・列を足す・消す・戻す（#99・パレットから・gate-proofs 第 64 節）。キーは Ctrl++ と Ctrl+-（#103・ADR 0020 の施主の判断）。複製・移動・Windows の実機はまだなので状態は変えない |
| EDIT-04 | 見出しの名前の変更 | 1 | 未着手 | FR-005 | 多 | 要望 10 |
| EDIT-05 | 下へ埋める（fill down）・空欄を上の値で埋める・連番 | 2 | 未着手 | FR-005 | 一部 | 要望・Miller / OpenRefine |
| EDIT-06 | 例から推し量る入力（Flash Fill） | 3 | 未着手 | FR-005 | 少 | 競合・EmEditor |
| EDIT-07 | 列の追加（式・連番・固定値・ID） | 2 | 未着手 | FR-005 | 一部 | 競合・Delimit / Modern |
| EDIT-08 | 列の結合と分割（区切り・正規表現・固定幅） | 1 | 未着手 | FR-011 | 多 | 競合 |
| EDIT-09 | 複数の値を含むセルを行に分ける・行をまとめる（explode / implode） | 2 | 未着手 | FR-005 | 少 | 要望・qsv / OpenRefine |
| EDIT-10 | セルの中の改行を安全に入力・編集 | 1 | 確認済み | FR-005 | 一部 | 要望・翻訳・Alt+Enter で欄の中に改行（GitHub Issue #69・gate-proofs 第 57 節）。保存でその行の終わり方に揃える規則は core（#56）。画面からの保存は GitHub Issue #71（core のテスト `line_breaks_follow_the_row`・欄の規則は `a_newline_character_is_a_line_break_inside_the_cell`）。Alt+Enter は施主の Windows の実機でも確かめた（gate-proofs 第 60 節） |
| EDIT-11 | IME の確定前の文字列をセルの中に出す | 1 | 未着手 | FR-020 | 一部 | 討議・方式は ADR 0026（Windows は IMM32、X11 は当面の貼り付け）。欄の規則（確定前の文字列の差し込みと下線の範囲）は core の `text_edit`（GitHub Issue #65・core のテスト）。Windows の IMM32 の経路（組み立ての窓を消す・確定前の文字列をセルに描く・確定を欄に挿す・候補の窓をセルの左下に置く・表の上で打ち始めて編集に入る）は GitHub Issue #81（application のテスト）。施主の Windows の実機で 6 項目を確かめた（gate-proofs 第 61 節）。X11 の XIM はまだなので状態は変えない |
| EDIT-12 | 入力の候補（同じ列の既存の値から） | 3 | 未着手 | FR-005 | 少 | 競合・Easy |
| EDIT-13 | 読み取り専用で開く（誤って変えない） | 1 | 確認済み | FR-005 | 一部 | 競合・パレットの「読み取り専用を切り替える」で読み取り専用にする / 外す。読み取り専用の間は表の値・行・列を変える操作（打つ・直す・消す・貼る・切り取る・Ctrl+Enter・置換・行と列の足し消し・Undo / Redo・保存・ダブルクリック・IME の確定）を止めて状態の行で知らせ、タイトルバーのファイル名の後ろに「（読み取り専用）」を出す（GitHub Issue #238・application のテスト `a_read_only_table_refuses_every_change_until_released`・Xvfb は gate-proofs 第 92 節）。Excel が開いているファイルを読み取り専用で開くことは FILE-04。施主の Windows の実機で確かめた（gate-proofs 第 94 節）。開くときに読み取り専用で開く指定（コマンドラインなど）はまだ |
| EDIT-14 | 隔離した壊れた行（OPEN-15）を元のバイト列のまま手で直し、読み直して列数が合うことを確かめる | 1 | 未着手 | FR-023 | 無 | GitHub Issue #8・ペルソナ ETL / EC（ローダーの reject を GUI で直す。「見せる」だけでは CLI の `store_rejects` と同じ） |
| EDIT-15 | 新規作成（空の表から始め、保存のときに名前と方言を決める） | 1 | 実装済み | FR-005 | 多 | GitHub Issue #85（施主 2026-11-05「行や列を追加できないと、新規で作れない」・一覧の漏れ）。行と列を足す EDIT-03 と、名前を付けて保存 SAVE-11 と組。GitHub Issue #109 で `--new` から 1 行 × 3 列の新しい表を開けるようにし、#111 で Ctrl+S が名前を付けて保存になり、#113 で Ctrl+N とパレットから新しい窓で開けるようになった（ADR 0028。X11 は Xvfb で確かめた・gate-proofs 第 67〜69 節。起動の入口はまだ。Ctrl+N は施主の Windows の実機で確かめた・gate-proofs 第 86 節） |
| EDIT-16 | 切り取りと貼り付け（Ctrl+X → Ctrl+V で、セル・行全体・列全体を動かす。貼るまでは元を消さず、点線の枠で示す） | 1 | 実装済み | FR-005 | 多 | GitHub Issue #105（施主 2026-11-06「カットペーストが欲しくなる」・一覧の漏れ）。GitHub Issue #149（ADR 0029 の決定 5・6）：Ctrl+X とパレットで点線の枠、この窓で貼ったら元を空のセルにして 1 手で戻る。行全体・列全体も値を動かす（並びは変えない）。Esc・直し始め・表を変える操作で枠は消える（gate-proofs 第 76 節）。行と列そのものを動かす（切り取ったセルの挿入・見出しのドラッグ）はまだ（EDIT-03）。施主の Windows の実機で切り取りと貼り付けを確かめた（gate-proofs 第 86 節） |
| EDIT-17 | 選んだ範囲の枠をドラッグして動かす・Ctrl+ドラッグで写す（セル・行全体・列全体） | 1 | 未着手 | FR-005 | 多 | GitHub Issue #105（施主 2026-11-06「ドラッグドロップが欲しくなる」・一覧の漏れ）。Excel の枠のドラッグ。行と列の見出しのドラッグでの移動は EDIT-03 |

## 7. 元に戻す・履歴・安全網（UNDO）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| UNDO-01 | 無制限の元に戻す / やり直す（一括の操作も 1 手で） | 1 | 確認済み | FR-006 | 多 | セルの書き換えの元に戻す・やり直す（Ctrl+Z・Ctrl+Y・Ctrl+Shift+Z・パレット）は GitHub Issue #75（application の `edit_history`・無作為の 500 列で全部戻すと差分が空・全部やり直すと元の差分）。保存の後も戻せる（触ったセルのファイルの値を保存した値に進める）。貼り付けの束（書き換え・足した行・足した列）を 1 手で戻すのは GitHub Issue #146（`EditStep::Batch`・ADR 0029 の決定 3）。置換の束は GitHub Issue #163（application のテスト `replacing_all_shows_the_count_first_and_undoes_in_one_step`）。施主の Windows の実機で、保存の後に戻すことも確かめた（gate-proofs 第 60 節） |
| UNDO-02 | 操作の履歴の一覧（何をしたか・どこまで戻すか選べる） | 2 | 未着手 | FR-006 | 少 | 文献・Wrangler |
| UNDO-03 | 未保存の変更の自動の退避と、落ちた後の復元（hot exit） | 2 | 未着手 | FR-006 | 少 | 競合・SmoothCSV |
| UNDO-04 | 保存の前の差分の確認（何行変わるか） | 2 | 未着手 | FR-006 | 無 | 討議 B |
| UNDO-05 | 保存のときに元のファイルのバックアップを残す（選べる） | 2 | 未着手 | FR-006 | 少 | 競合・SmoothCSV |

## 8. 検索・置換（FIND）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| FIND-01 | 検索（大文字小文字・セル全体の一致・列を限る・選択範囲を限る） | 1 | 未着手 | FR-008 | 多 | 競合。照合の規則（大文字小文字・セル全体・全角半角・ひらがなカタカナ・ADR 0030 の決定 3）と見えている値で照らすことは GitHub Issue #151（core `find_query`・application `find_scope`）。画面は GitHub Issue #152（Ctrl+F の探す段・Enter / F3 で次・Shift+Enter / Shift+F3 で前・端から回る知らせ・Alt+C / Alt+W・足した行も・gate-proofs 第 77 節）。欄が開いている間は見えている当たったセルと字に印を付ける（GitHub Issue #189・ADR 0030 の決定 7・gate-proofs 第 87 節）。条件は設定に残し、次の起動でも同じ条件で欄を開く（GitHub Issue #213）。当たった所の印は施主の Windows の実機で確かめた（gate-proofs 第 91 節）。列と選択範囲を限るのはまだ。2026-11-09（GitHub Issue #285 の見直し・専門家ペルソナの推し・施主の確認待ち）：当たりを箱の下に隠さない（#286・#300）・条件は名前を固定した札（#288）・見つからないことを欄の中で（#291）・知らせの色（#305） |
| FIND-02 | 置換（同じ条件・件数の事前表示・プレビュー） | 1 | 未着手 | FR-008 | 一部 | 競合・Modern。当たった所だけを元の字のまま置き換える規則は GitHub Issue #161（core `FindQuery::replaced`）。1 つずつと、件数を見せてから 1 手の束で書く全部の置換は GitHub Issue #163（Ctrl+Z 1 回で戻る・100 万セルまで）。画面は GitHub Issue #165（Ctrl+H の置換の段・Tab で欄を替える・Enter で 1 つ・Alt+A で全部・gate-proofs 第 78 節・benchmarks 第 9 節）。全部の置換の件数と確かめはパレットの中に出し、数えた当たりに印を付ける（GitHub Issue #189・gate-proofs 第 87 節）。プレビューの表（ADR 0030 でフェーズ 2）はまだ。2026-11-09（#285 の見直し・施主の確認待ち）：置換は名札付きの 2 欄（#287）・全部の置換の確かめに「前→後」の内訳と一部だけ変わる組（#289・#299） |
| FIND-03 | 正規表現 | 1 | 確認済み | FR-008 | 多 | 要望 5・競合。GitHub Issue #8（業務ペルソナ 5 人の洗い出し・`_work/reports/2026-11-05-nene-ledger-persona-needs.md`）でフェーズ 1 へ。照合と置換（`$1`・`${name}` は元の字で埋める）は GitHub Issue #171（core の `regex-automata`・ADR 0031）。画面の切り替え（Alt+R・読めない式の知らせ）は GitHub Issue #173（gate-proofs 第 81 節・1 GB の時間は benchmarks 第 10 節・普通の検索の 2.5 倍ほど）。施主の Windows の実機で検索と置換を確かめた（gate-proofs 第 86 節）。速い道は依存を足さない先探し（式から当たりの頭の字の並びを取り出す・GitHub Issue #223）で、1 GB の画面の検索が 4.2〜4.6 s → 1.6 s・全部の置換の数えが 6.8〜7.0 s → 1.7 s（benchmarks 第 12 節）。正規表現の検索も施主の Windows の実機で確かめた（954 MB で `R1222485\d$` が「１秒くらい？」〔伝聞〕・gate-proofs 第 91 節） |
| FIND-04 | 一致の件数・全一致の選択・一致した行だけを見る | 2 | 未着手 | FR-008 | 一部 | 競合・Modern / CSView |
| FIND-05 | 巨大なファイルでも裏で検索し、見つかった順に出す | 1 | 実装済み | FR-010 | 少 | 速さ。原本を塊ごとに照らして最初に当たった所で止まる畳み込みは GitHub Issue #151（`--find` で 1 GB の最後の行まで 9.1〜9.8 s・benchmarks 第 8 節）。画面からの裏の検索（進み具合・Esc で止める）は #152（1 GB の最後の行まで 9.2〜9.3 s）。GitHub Issue #157 で索引の目印で分けて並列に（8 スレッド・1.6〜1.8 s・benchmarks 第 8 節）。施主の Windows の実機で検索を確かめた（gate-proofs 第 86 節・小さなファイル）。Windows の実機の 1 GB はまだ |
| FIND-06 | 全角半角・ひらがなカタカナを同一視する検索（選べる） | 1 | 確認済み | FR-008 | 無 | 要望・日本。GitHub Issue #8（業務ペルソナ 5 人の洗い出し・`_work/reports/2026-11-05-nene-ledger-persona-needs.md`）でフェーズ 1 へ。照合の規則（全角半角は既定で区別しない・ひらがなカタカナは既定で区別する・ADR 0030 の決定 3）は core の `find_folding`（GitHub Issue #151・テスト `the_standard_ignores_case_and_width_and_keeps_kana_and_toggles_flip_one_each`）。画面で選ぶ切り替え（決定 3 の「パレットの切り替え」）は検索の欄の Alt+Z・Alt+K（GitHub Issue #199・gate-proofs 第 88 節・X11 は Xvfb で確かめた）。施主の Windows の実機で確かめた（gate-proofs 第 91 節） |
| FIND-07 | 複数のファイルをまたいだ検索 | 3 | 未着手 | FR-008 | 少 | — |

## 9. 並べ替え（SORT）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| SORT-01 | 複数キー・昇順 / 降順・安定 | 1 | 未着手 | FR-009 | 多 | 競合 |
| SORT-02 | 比較の方法を列ごとに: 文字列 / 数 / 日付 / 長さ / 自然順（`a2` < `a10`） | 1 | 未着手 | FR-009 | 一部（自然順は無） | 討議 A・競合 |
| SORT-03 | 数や日付として読めない値の置き場（先頭 / 末尾）と件数 | 1 | 未着手 | FR-009 | 無 | 討議 A |
| SORT-04 | 見えている順で保存するか、元の順で保存するか | 1 | 未着手 | FR-009 | 無 | ADR 0005 |
| SORT-05 | 範囲だけの並べ替え | 3 | 未着手 | FR-009 | 少 | 競合・Modern / Easy |
| SORT-06 | 日本語の並び（読みではなく文字コード順 / Unicode 照合順の選択） | 2 | 未着手 | FR-009 | 無 | 要望・日本 |

## 10. 絞り込み（FILT）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| FILT-01 | 見出しからの絞り込み（一意値のチェックボックス・件数付き） | 1 | 未着手 | FR-009 | 一部 | 要望・分析 |
| FILT-02 | 条件の組み合わせ（AND / OR / NOT・含む・一致・範囲・空・正規表現） | 1 | 未着手 | FR-009 | 多 | 競合 |
| FILT-03 | 絞り込んだ行だけを保存 / 書き出し / 削除 | 1 | 未着手 | FR-009 | 一部 | 競合・Modern / CSVed。GitHub Issue #8（業務ペルソナ 5 人の洗い出し・`_work/reports/2026-11-05-nene-ledger-persona-needs.md`）でフェーズ 1 へ |
| FILT-04 | 条件の保存と再利用 | 2 | 未着手 | FR-009 | 少 | 競合・Easy |
| FILT-05 | ファセット（サイドバーに列ごとの値の分布・数の範囲・空欄） | 2 | 未着手 | FR-009 | 少 | 要望・OpenRefine / Datasette |
| FILT-06 | 重複している行・一意の行だけを見る | 2 | 未着手 | FR-009 | 一部 | 要望 9 |
| FILT-07 | 式による絞り込み（列の値の比較・関数） | 3 | 未着手 | FR-009 | 一部 | 競合・Delimit / SmoothCSV |

## 11. 変換・整形（XFRM）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| XFRM-01 | 変換は列にかける規則として持ち、適用の前にプレビューと影響の件数 | 1 | 未着手 | FR-011 | 少 | ADR 0005・文献・Wrangler |
| XFRM-02 | 大文字 / 小文字 / 先頭だけ大文字 | 1 | 未着手 | FR-011 | 多 | 競合 |
| XFRM-03 | 前後の空白・連続する空白・見えない文字の除去 | 1 | 未着手 | FR-011 | 一部 | 要望・CRM / 翻訳 |
| XFRM-04 | 置換（文字列・正規表現）を列に | 1 | 未着手 | FR-011 | 多 | 競合 |
| XFRM-05 | 日付の形式の変換（曖昧さの解釈を選ぶ・読めない件数を先に） | 1 | 未着手 | FR-011 | 一部 | 文献・落とし穴 21 |
| XFRM-06 | 数の形式（小数点・桁区切り・負の数 `(30.00)`・0 埋め・先頭の 0 の除去） | 2 | 未着手 | FR-011 | 一部 | 要望・経理 |
| XFRM-07 | 全角 ⇔ 半角・NFKC の正規化（明示の変換） | 1 | 未着手 | FR-011 | 無 | 文献・落とし穴 16・要望・日本 |
| XFRM-08 | 和暦 ⇔ 西暦（明示の変換。元号の表はデータファイル・ADR 0008） | 2 | 未着手 | FR-011 | 無 | 要望・日本 |
| XFRM-09 | 重複の削除（キー列を選ぶ・最初 / 最後を残す） | 1 | 未着手 | FR-011 | 多 | 要望 9 |
| XFRM-10 | 空行・空列の削除 | 2 | 未着手 | FR-011 | 一部 | 競合 |
| XFRM-11 | 表記ゆれのクラスタリングと一括の統合（fingerprint・n-gram・距離） | 2 | 未着手 | FR-011 | 無（GUI のエディター） | 要望・OpenRefine |
| XFRM-12 | 転置（範囲・上限つきで全体） | 2 | 未着手 | FR-015 | 一部 | 要望・研究 |
| XFRM-13 | 縦持ち ⇔ 横持ち（melt / pivot・unpivot） | 2 | 未着手 | FR-015 | 一部 | 競合 |
| XFRM-14 | 列の値から列を作る（前の行との差・累計・構成比・連結・抜き出し） | 2 | 未着手 | FR-011 | 少 | 要望・Miller。GitHub Issue #8（業務ペルソナ 5 人の洗い出し・`_work/reports/2026-11-05-nene-ledger-persona-needs.md`）でフェーズ 2 へ（列の演算 XFRM-17 と対） |
| XFRM-15 | 正規表現で抜き出して新しい列に | 2 | 未着手 | FR-011 | 少 | 要望・VisiData |
| XFRM-16 | 列の並べ替えと選択をテンプレート（列の名前と順）に合わせる | 1 | 未着手 | FR-011 | 無 | 要望・EC / CRM。GitHub Issue #8（業務ペルソナ 5 人の洗い出し・`_work/reports/2026-11-05-nene-ledger-persona-needs.md`）でフェーズ 1 へ |
| XFRM-17 | 列の演算（列 × 定数・列 ± 列・丸め方を選んで上書きか新しい列。税込・掛け率） | 2 | 未着手 | FR-011 | 一部 | GitHub Issue #8・ペルソナ EC / 経理（計算パレットの「列を作り直す」側。結果は値・ADR 0022） |

## 12. 統計・要約（STAT）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| STAT-01 | 列の要約: 件数・空・一意の数・最小 / 最大・平均・中央値・分位点・長さ | 1 | 未着手 | FR-012 | 一部 | 要望 12。GitHub Issue #8（業務ペルソナ 5 人の洗い出し・`_work/reports/2026-11-05-nene-ledger-persona-needs.md`）でフェーズ 1 へ |
| STAT-02 | 値の頻度表（上位・件数・割合）とヒストグラム | 1 | 未着手 | FR-012 | 一部 | 要望・VisiData / Modern。GitHub Issue #8（業務ペルソナ 5 人の洗い出し・`_work/reports/2026-11-05-nene-ledger-persona-needs.md`）でフェーズ 1 へ |
| STAT-03 | ファイル全体のプロファイル（列ごとの読める率・空の率・異常の件数） | 2 | 未着手 | FR-012 | 少 | 文献・Wrangler の品質メーター |
| STAT-04 | ピボット・グループ集計 | 2 | 未着手 | FR-013 | 一部 | 競合 |
| STAT-05 | 集計行を下に固定表示 | 3 | 未着手 | FR-012 | 少 | 競合・SmoothCSV |
| STAT-06 | 時刻ごとの件数（時系列のヒストグラム） | 3 | 未着手 | FR-012 | 無 | 要望・ログ |
| STAT-07 | 簡単なグラフ | 3 | 未着手 | FR-012 | 少 | 競合・Ron's |
| STAT-08 | 概算（HyperLogLog・t-digest）を使うときは概算と明示 | 1 | 未着手 | FR-012 | 無 | ADR 0007 |
| STAT-09 | 集計パレット: 選んだ範囲か列に基本関数（SUM・AVERAGE・COUNT・COUNTA・COUNTBLANK・MIN・MAX・MEDIAN・一意の数・ROUND〈四捨五入／切り捨て／切り上げ〉）を当て、結果を値として見せる・貼る。数として読めなかった件数も出す。セルに式は保存しない（ADR 0022） | 1 | 未着手 | FR-012 | 一部 | 施主 2026-11-05・GitHub Issue #8。5 人とも SEC-01 と両立すると判断。VIEW-14 と同じ経路の別の入口 |

## 13. 結合・参照・ファイルの分割と連結（JOIN）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| JOIN-01 | 列の参照（VLOOKUP 相当） | 2 | 未着手 | FR-014 | 少 | 競合・Modern |
| JOIN-02 | 2 ファイルの結合（INNER / LEFT / RIGHT / OUTER・複数キー） | 2 | 未着手 | FR-014 | 少 | 競合・EmEditor |
| JOIN-03 | 別のファイルとの一致 / 不一致の抽出 | 2 | 未着手 | FR-014 | 少 | 競合・Delimit |
| JOIN-04 | 複数のファイルの縦の連結（列の名前で合わせる） | 2 | 未着手 | FR-014 | 一部 | 競合 |
| JOIN-05 | ファイルの分割（行数ごと・大きさごと・列の値ごと） | 2 | 未着手 | FR-014 | 一部 | 競合・EmEditor / Delimit |

## 14. 検証・規則（VALD）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| VALD-01 | 構造の検査（列数の不一致・不正な引用符・壊れた文字）を一覧し、該当の行へ飛ぶ | 1 | 未着手 | FR-023 | 少 | 競合・EmEditor / Rainbow |
| VALD-02 | 書式の規則（列の名前と順・必須・見方・長さ・**全角を 2 と数えるバイト数**・選択肢・正規表現・一意）を作って保存・共有 | 1 | 未着手 | FR-027 | 無 | 討議 D。GitHub Issue #8（業務ペルソナ 5 人の洗い出し・`_work/reports/2026-11-05-nene-ledger-persona-needs.md`）でフェーズ 1 へ |
| VALD-03 | 規則に違反したセルの強調・一覧・違反した行だけを取り出して直す | 1 | 未着手 | FR-027 | 無 | 要望 6。GitHub Issue #8（業務ペルソナ 5 人の洗い出し・`_work/reports/2026-11-05-nene-ledger-persona-needs.md`）でフェーズ 1 へ |
| VALD-04 | 規則と方言を Frictionless Table Schema / CSVW の語彙で読み書き（ADR 0008） | 2 | 未着手 | FR-027 | 無 | 文献・Frictionless |
| VALD-05 | 取り込み先のプリセット（楽天・弥生・Shopify など）を規則ファイルとして同梱する。コードに焼き込まない。確かめた日付と出典を持つ（ADR 0008） | 1 | 未着手 | FR-027 | 無 | 討議 D（施主の判断）。GitHub Issue #8（業務ペルソナ 5 人の洗い出し・`_work/reports/2026-11-05-nene-ledger-persona-needs.md`）でフェーズ 1 へ |
| VALD-06 | オープンデータの作成ルールの検査（1 セル 1 データ・機種依存文字・和暦だけの表記） | 3 | 未着手 | FR-027 | 無 | 要望・公開 |
| VALD-07 | 文字集合の規則（許される文字の範囲: JIS X 0201・半角カナ禁止・CP932 で表せる字だけ・絵文字禁止）で検査し、違反のセルへ飛ぶ | 1 | 未着手 | FR-027 | 無 | GitHub Issue #8・ペルソナ 人事（e-Gov・健保・自治体は許される文字が決まっている。桁数だけでは足りない） |

## 15. 比較（DIFF）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| DIFF-01 | 2 つのファイルをキー列で突き合わせる差分（追加・削除・変更をセル単位） | 2 | 未着手 | FR-028 | 少 | 要望 7・討議 H |
| DIFF-02 | 差分だけを絞り込んで見る・差分を書き出す | 2 | 未着手 | FR-028 | 少 | 要望 7 |
| DIFF-03 | 差分の取り込み（マージ） | 3 | 未着手 | FR-028 | 少 | 競合・SmoothCSV |
| DIFF-04 | 開いている内容とディスクの原本の差分 | 2 | 未着手 | FR-028 | 無 | 討議 B |

## 16. 保存・書き出し（SAVE）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| SAVE-01 | 一時ファイルに書いて置き換える保存（途中で落ちても元は残る） | 1 | 確認済み | FR-007 | 無（明記なし） | ADR 0005。保存の経路（store）と `--resave` の往復の実測は GitHub Issue #58（benchmarks 第 4 節）。画面から Ctrl+S（とパレットの「直したセルを保存する」）で、計画を作って裏の `SaveJob` で書いて置き換え、開き直すのは GitHub Issue #71（gate-proofs 第 58 節・X11 で直した 2 つのセルだけが変わり、引用と CRLF は元のまま）。進み具合と中止は SAVE-02、別名で保存は SAVE-11。未保存のまま閉じるときに尋ねる（保存して閉じる・保存しないで閉じる・やめる）のは GitHub Issue #73（gate-proofs 第 59 節）。施主の Windows の実機で、保存（`ReplaceFileW`）・Excel が開いているときに止まること・閉じるときに尋ねて保存することを確かめた（gate-proofs 第 60 節） |
| SAVE-02 | 保存を裏で・進み具合・中止・空きの事前の確認 | 1 | 未着手 | FR-007 | 無 | 討議 B・失敗のときに一時ファイルを消す経路（`discard`）は store にある。計画を作ることも裏で行い（GitHub Issue #130）、進み具合を状態の行に出して Esc でやめられる（GitHub Issue #132・store のテスト・1 GB で Esc・gate-proofs 第 73 節）。施主の Windows の実機で保存の Esc を確かめた（gate-proofs 第 86 節）。空きの事前の確認はまだなので状態は変えない。空きの事前の確認は依存を入れずに進め、`rustix` を足すかは M2 の終わりに決める（ADR 0025 の施主の判断 6）。裏で回すことは ADR 0026 の決定 6 |
| SAVE-03 | 方言を変えて書き出す（文字コード・BOM・区切り・改行・引用の方針: 必要なときだけ / 全セル / 文字列だけ / なし） | 1 | 未着手 | FR-007 | 一部 | 要望 4・日本 |
| SAVE-04 | 保存の前の文字コードの検査（表せない文字・往復で変わる文字の一覧と置換の候補） | 1 | 未着手 | FR-023 | 無 | 討議 F |
| SAVE-05 | Excel 向けの書き出し（BOM・全セルの引用・数式の無害化を選べる）と「Excel で壊れる値」の警告 | 2 | 未着手 | FR-007 | 無 | 討議 E |
| SAVE-06 | 選んだ範囲・絞り込んだ行・選んだ列だけを書き出す | 1 | 未着手 | FR-007 | 一部 | 競合。GitHub Issue #8（業務ペルソナ 5 人の洗い出し・`_work/reports/2026-11-05-nene-ledger-persona-needs.md`）でフェーズ 1 へ |
| SAVE-07 | 他の形式への書き出し（JSON / JSON Lines / Markdown / HTML / SQL の INSERT） | 3 | 未着手 | FR-030 | 一部 | 競合 |
| SAVE-08 | xlsx の書き出し（全セルを文字列として保存し、相手の Excel で壊れない） | 2 | 未着手 | FR-030 | 無 | ADR 0008 |
| SAVE-09 | xlsx の読み込み（値と表示の書式のどちらを取るかを選ぶ） | 3 | 未着手 | FR-030 | 一部 | ADR 0008 |
| SAVE-10 | Parquet の読み書き（依存は導入の Issue で ADR） | 3 | 未着手 | FR-030 | 少 | ADR 0008 |
| SAVE-11 | 名前を付けて保存（別の名前・場所に、同じ方言で書く。元のファイルは変えない） | 1 | 確認済み | FR-007 | 多 | FR-007 の本文にあるのに行が無かった（GitHub Issue #85・一覧の漏れ）。書き方は保存の置き換え（ADR 0025）と同じ経路で、置き換え先だけが違う。GitHub Issue #111 で入れた：Ctrl+Shift+S・パレット。Windows は OS の画面（`GetSaveFileNameW`）、X11 はパレットの欄。まだ無いファイルは store の `save_absent` で作る（ADR 0028）。施主の Windows の実機で確かめた（gate-proofs 第 86 節） |

## 17. クリップボード・他のアプリ（CLIP）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| CLIP-01 | Excel とのコピー / 貼り付け（タブ区切り・HTML）。貼り付けでも型を推し量らない | 1 | 未着手 | FR-016 | 一部 | 要望・X11 のタブ区切りのコピーは Issue #30（gate-proofs 第 21 節）。貼り付けは GitHub Issue #115 で 1 つのセルと編集中の欄・パレットの欄まで（両方の OS・Excel の引用を解く `parse_tsv`・gate-proofs 第 70 節）。表への複数のセルの貼り付けは GitHub Issue #146（ADR 0029 の決定 1〜4・左上から束のまま・表の端を越えれば行と列を足す・1 つの値は範囲を埋める・1 手で戻す・100 万セルまで・gate-proofs 第 75 節・benchmarks 第 7 節）。HTML はまだ。施主の Windows の実機で貼り付けを確かめた（gate-proofs 第 86 節）。X11 の大きな中身（`INCR`）は GitHub Issue #140 で受け取る（64 MiB まで・gate-proofs 第 74 節） |
| CLIP-02 | 見出し付きでコピー・形式を選んでコピー（CSV / Markdown / JSON / SQL） | 2 | 未着手 | FR-016 | 一部 | 競合・SmoothCSV |
| CLIP-03 | 外から貼った表の大きさが選択と違うときの扱い（はみ出し・行の追加）を確かめる | 1 | 未着手 | FR-016 | 少 | — |
| CLIP-04 | ドラッグ＆ドロップでファイルを開く | 1 | 確認済み | FR-016 | 多 | 窓に落としたファイルは、どれも新しい窓で開く（施主の判断 2026-11-08・自分の exe をその道筋を付けて起こす・ADR 0028 の決定 1 と同じ作り・GitHub Issue #243）。X11 は XDND の版 5（`XdndAware`・`XdndStatus` で写して受け取る・`XdndSelection` を `text/uri-list` で読む・`XdndFinished`）。`file:` の URI の読み方は ui-x11 のテスト（`file_uris_become_paths`）。Xvfb で XDND の送り手を真似る道具から 2 つ落として 2 つの窓が開いた（gate-proofs 第 93 節）。Windows は `DragAcceptFiles`・`WM_DROPFILES`。施主の Windows の実機で、エクスプローラーから 2 つ落として 2 つの新しい窓が開き、元の窓が残ることを確かめた（gate-proofs 第 94 節） |
| CLIP-05 | 「既定のアプリ」としての関連付け・コマンドラインから開く（行や列を指定して） | 2 | 未着手 | FR-016 | 一部 | 競合・SmoothCSV |

## 18. 自動化・繰り返し（AUTO）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| AUTO-01 | 操作の手順（レシピ）の記録・名前を付けて保存・別のファイルへ再生。形式は独自の版付きテキスト（ADR 0008） | 2 | 未着手 | FR-029 | 少 | 討議 C・文献・Wrangler / OpenRefine |
| AUTO-02 | 手順を読める形で書き出す・編集する | 2 | 未着手 | FR-029 | 少 | 討議 C |
| AUTO-03 | コマンドパレット（全操作を名前で探して実行） | 2 | 実装済み | FR-029 | 一部 | 競合・Modern / SmoothCSV・呼び出しは Ctrl+Shift+P（施主 2026-10-26。Ctrl+P は印刷 PRNT-01 に空ける）・Issue #41（gate-proofs 第 30 節。検索は英語とローマ字・IME はまだ）。2026-11-09：漢字とかなで引く（#303）・最近使った操作（#309）・開いている間の別の段のキーで段を替える（#290）・試す段は箱を下へ（#311）・2 段目の注と印（#312）・見出しの行を 2-3 で絞る（#315） |
| AUTO-04 | キー割り当ての変更 | 2 | 未着手 | FR-029 | 一部 | 競合 |
| AUTO-05 | 手順をコマンドラインから流す（GUI なしの一括処理） | 3 | 未着手 | FR-029 | 少 | 競合・SmoothCSV / EmEditor |
| AUTO-06 | SQL での問い合わせ（SELECT・WHERE・GROUP BY・ORDER BY・JOIN。読み取り専用。GUI と同じ自前のエンジンへの入口・ADR 0008） | 3 | 未着手 | FR-029 | 少 | 討議 C（施主の判断） |
| AUTO-07 | スクリプト言語の組み込み（JavaScript / Python など） | — | やらない | FR-029 | 少 | 依存が大きい・ADR 0007 の趣旨 |
| AUTO-08 | OpenRefine の操作履歴（JSON）の取り込み | 3 | 未着手 | FR-029 | 無 | ADR 0008 |

## 19. 日本語・地域（JP）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| JP-01 | CP932 と JIS の Shift_JIS を区別して選ぶ。日本語の既定は CP932 | 1 | 未着手 | FR-002 | 少 | 文献・落とし穴 13・CP932 の判別と復号は Issue #17・#18、2 バイト目と重なる区切りは Issue #35（gate-proofs 第 26 節）。JIS の Shift_JIS との区別と選ぶ手段はまだ（ADR 0015 の決定 5） |
| JP-02 | 波ダッシュなど、往復で変わる文字の警告 | 1 | 未着手 | FR-023 | 無 | 文献・落とし穴 14 |
| JP-03 | 機種依存文字・絵文字（Shift_JIS で表せない文字）の検出と置換の候補 | 1 | 未着手 | FR-023 | 無 | 要望・日本 |
| JP-04 | 全角を 2 と数える桁数（バイト数）の表示 | 1 | 未着手 | FR-002 | 無 | 要望・経理（弥生）。GitHub Issue #8（業務ペルソナ 5 人の洗い出し・`_work/reports/2026-11-05-nene-ledger-persona-needs.md`）でフェーズ 1 へ |
| JP-05 | 全項目を `"` で囲んで保存 | 1 | 未着手 | FR-007 | 一部 | 要望・日本（Excel でできない） |
| JP-06 | 画面の表示言語（日本語・英語ほか）→ LANG-01〜04 に分けた | 1 | 取り下げ | FR-034 | 少 | 競合・SmoothCSV |
| JP-07 | 半角カナの検出と変換 | 2 | 未着手 | FR-002 | 無 | 要望・経理 |

## 20. 安全（SEC）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| SEC-01 | 数式を一切評価しない・外部のプログラムを起動しない | 1 | 確認済み | FR-031 | 多 | 文献・OWASP・数式を評価する処理も外部のプログラムを起動する処理も無い。数式に見えるセルが表とコピーで字のまま残ることは application のテスト `formula_like_cells_stay_as_text`。外部のプログラムを起こすのは、自分の exe を起こし直して新しい窓を開くアダプターの 2 ファイルだけで、ほかはどのクレートでも CNF-016 が拒む（GitHub Issue #249・gate-proofs 第 95 節） |
| SEC-02 | 数式に見えるセル（`=` `+` `-` `@` タブ CR・全角の `＝＋－＠`）の強調と件数・DDE の形の警告 | 2 | 未着手 | FR-031 | 無 | 文献・OWASP |
| SEC-03 | Excel 向けの書き出しで無害化を選べる（既定は変えない） | 2 | 未着手 | FR-031 | 無 | 討議 E |
| SEC-04 | 病的な入力（巨大なセル・引用の地獄・不正なバイト列）で落ちない・固まらない | 1 | 確認済み | FR-031 | 無 | 文献・セキュリティ・巨大なセルと閉じない引用符は Issue #13・#15・#23（gate-proofs 第 16 節）、読めないバイトは復号で知らせる（Issue #18）。任意のバイト列で開く道と全部を書き出す道が落ちないことは app のテスト `arbitrary_bytes_open_and_dump_without_failing`（ファズに近い 400 通り）。約 100 MB の引用符だけ・行ごとに閉じない引用符・組み違いの引用符・無作為のバイトは、どれも 0.5〜1.1 s で索引を作り終え、末尾へ 16〜24 ms、記憶 20〜34 MB（release・Xvfb・GitHub Issue #255・benchmarks 第 18 節）。本物のファザー（cargo-fuzz）は使っていない |
| SEC-05 | データを外に送らない（クラウド・AI・テレメトリ・Web からの取り込みなし） | 1 | 確認済み | FR-031 | 一部 | 討議 J・外へ送る処理は無い。外へ送る API（`std::net`・Windows の `WinHttp` など）と `Win32_Networking` の機能は、どのクレートでも CNF-016 が拒む（GitHub Issue #249・gate-proofs 第 95 節）。外部クレートは CNF-013 の許可表と cargo-deny が守る |

## 21. 操作性・アクセシビリティ（A11Y）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| A11Y-01 | キーボードだけで全操作（ショートカットの一覧つき） | 1 | 確認済み | FR-032 | 一部 | 討議 I・要望 11・キーでの移動・選択・コピー・拡大・配色・コマンドパレット（Issue #22・#28・#30・#34・#37・#41）。行全体・列全体の選択（Shift+Space・Ctrl+Space）と列を左右へ移すキー（Ctrl+Shift+PageUp / PageDown）は Issue #62（パレットにも・両方の OS のキーのテスト）。キーの一覧はパレットの「キーの一覧を見る…」（Issue #63・gate-proofs 第 47 節・一覧の各行の割り当てを両方の OS のキーのテストが確かめる）。施主の Windows の実機で行全体・列全体の選択・列を移すキー・キーの一覧を確かめた（gate-proofs 第 96 節） |
| A11Y-02 | フォーカスがいつも見える・高コントラストのテーマ | 1 | 確認済み | FR-032 | 無 | 討議 I・カーソルは色に加えて枠でも見せる（Issue #32・gate-proofs 第 23 節）。全配色でカーソルの枠と背景の 3.0 以上をテストで求め、高コントラストの組み込み配色もある（Issue #34・第 25 節）。OS の高コントラストが有効なら起動時にその明暗の高コントラストの配色で開き、保存した配色より優先する（GitHub Issue #275・core の `ThemeChoice::resolve` のテスト。X11 は GTK の `HighContrast` / `HighContrastInverse` を Xvfb で、Windows は施主の実機で確かめた・gate-proofs 第 98 節） |
| A11Y-03 | OS の文字の大きさ・拡大率に従う | 1 | 確認済み | FR-021 | 少 | 討議 I・Windows の拡大率に従う（Issue #58・gate-proofs 第 45 節）。OS の文字の大きさは起動時に読み、表と詳細ペインの文字の大きさ（利用者の段）に掛けていちばん近い段で描く（施主の判断 2026-11-08・Windows は「テキストを大きくする」の `TextScaleFactor`・X11 は `Xft.dpi`・GitHub Issue #263・application の `os_text_scale_picks_the_nearest_step`・Xvfb で `Xft.dpi: 144` のとき表が 150%）。X11 の窓の倍率（画面の dpi）は当面 100%。施主の Windows の実機で「テキストのサイズ」を大きくして表の字が大きくなることを確かめた（gate-proofs 第 96 節）。アプリの中の拡大は VIEW-12 |
| A11Y-04 | スクリーンリーダー（Windows の UI Automation の Grid） | 2 | 未着手 | FR-032 | 無 | 討議 I・ADR 0002 の残る穴 |
| A11Y-05 | 複数のファイルをタブで開く・ウィンドウを分ける | 2 | 未着手 | FR-032 | 一部 | 競合 |
| A11Y-06 | 最近のファイル・作業状態（列の幅・絞り込み）の復元 | 2 | 未着手 | FR-024 | 一部 | 競合 |
| A11Y-07 | スクリーンリーダー（Linux の AT-SPI） | 3 | 未着手 | FR-032 | 無 | 討議 I |

## 22. ファイル・監視（FILE）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| FILE-01 | 外部の変更の検出（原本が変わったら示し、保存を止めて選ばせる） | 1 | 未着手 | FR-019 | 一部 | ADR 0005。保存の前と置き換えの直前に、開く前に取った印（大きさ・更新日時・Unix の dev と inode）と比べて止める経路は GitHub Issue #60（store の `stamping`・テスト `a_file_changed_after_opening_is_not_overwritten`・`a_change_while_writing_stops_before_the_replacement`）。開いている間に示すこと（FR-019）と選ばせる画面はまだ。窓の保存では、開く前に取った印を保存に渡し、計画を作る前にも比べる（GitHub Issue #71・gate-proofs 第 58 節） |
| FILE-02 | 外部の変更で再読み込み（未編集なら自動・選べる） | 2 | 未着手 | FR-019 | 一部 | 競合・Modern / SmoothCSV |
| FILE-03 | 追記の追従（tail） | 2 | 未着手 | FR-019 | 少 | 討議 K・要望・ログ |
| FILE-04 | ロックされたファイル（Excel が開いている）を読み取り専用で開く・保存先のロックで再試行 | 1 | 未着手 | FR-019 | 少 | ADR 0002・scan は Windows で、他のアプリの書き込みと削除を妨げない共有の指定で開く（Issue #11。型検査だけ）。開くときに原本を保存で置き換えられるかを store の `write_access::write_blocker` で見て（Windows は保存の前と同じ `lock_probe`・共有違反ならほかのアプリが開いている。Unix は書き込みの権限）、書けなければ読み取り専用（EDIT-13）で開き、状態の行で理由を知らせる（GitHub Issue #240・store のテスト `a_file_opened_without_sharing_is_locked_until_closed` は `make test-windows` で Windows で走る）。施主の Windows の実機で、Excel が開いているファイルを落とすと読み取り専用で開き理由が出ることを確かめた（gate-proofs 第 94 節）。保存先のロックでの再試行（M2）はまだ。保存先のロック（Excel が開いている）で書き始めないことは、施主の Windows の実機で確かめた（gate-proofs 第 60 節） |
| FILE-05 | ファイルの複製・名前の変更・ごみ箱へ | 3 | 未着手 | FR-018 | 少 | 競合・Modern |
| FILE-06 | ネットワーク共有の上のファイル（遅い・切れる）で固まらない | 2 | 未着手 | FR-019 | 無 | NeNeDocket の経験 |

## 23. 印刷（PRNT）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| PRNT-01 | 印刷・PDF（選んだ範囲・ページ数の上限・見出しの繰り返し。OS の印刷の仕組みに乗る・ADR 0008） | 3 | 未着手 | FR-017 | 少 | 討議・残った論点 6 |

## 24. 生成（GEN）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| GEN-01 | テストデータの生成（種を固定・空や重複を一定の割合で・日本語） | 3 | 未着手 | FR-033 | 無（GUI のエディター） | 要望・QA |
| GEN-02 | 境界値の行を足す（空・巨大・特殊文字・数式に見える値） | 3 | 未着手 | FR-033 | 無 | 要望・QA |

## 25. 窓（WIN）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| WIN-01 | 自前のタイトルバー（アプリ名・ファイル名・未保存と読み取り専用の印） | 1 | 確認済み | FR-025 | 無 | ADR 0013・X11 は Issue #27（アプリ名とファイル名まで。未保存と読み取り専用の印は編集を入れてから・Windows は Issue #55 で施主の実機で確かめた（gate-proofs 第 42 節））。未保存の印 `●` は GitHub Issue #69（保存すると消える・#71）。OS の窓の名前（タスクバー・Alt+Tab）にも同じ印と、名前を付けて保存した後の名前を出すのは GitHub Issue #167（X11 は gate-proofs 第 79 節・Windows の実機は gate-proofs 第 86 節）。読み取り専用の印（ファイル名の後ろに「（読み取り専用）」）は GitHub Issue #238（#239）で入れ、施主の Windows の実機で確かめた（gate-proofs 第 94 節） |
| WIN-02 | 最小化・最大化 / 元に戻す・閉じるのボタン（自前で描く） | 1 | 確認済み | FR-025 | 無 | ADR 0013・X11 は Issue #27・WSLg で施主確認（gate-proofs 第 17 節）。Windows は Issue #55 で施主の実機で確かめた（gate-proofs 第 42 節） |
| WIN-03 | タイトルバーのドラッグで移動・ダブルクリックで最大化 / 元に戻す | 1 | 確認済み | FR-025 | 無 | ADR 0013・X11 は Issue #27・WSLg で施主確認（gate-proofs 第 17 節）。Windows は Issue #55 で施主の実機で確かめた（gate-proofs 第 42 節） |
| WIN-04 | 枠と角のドラッグでリサイズ（見えない縁の当たり判定とカーソル） | 1 | 確認済み | FR-025 | 無 | ADR 0013・X11 は Issue #27・WSLg で施主確認（gate-proofs 第 17 節）。Windows は Issue #55 で施主の実機で確かめた（gate-proofs 第 42 節） |
| WIN-05 | Windows のスナップ（端へのドラッグ・Win+矢印・Windows 11 の Snap Layouts） | 1 | 確認済み | FR-025 | 無 | ADR 0013・Windows は Issue #55 で `HTCAPTION`・縁・`HTMAXBUTTON` を OS に渡す形にした（OS が行う）。施主の実機でスナップを確かめた（gate-proofs 第 42 節。Snap Layouts は最初は出ず、DWM の枠を中身へ広げるのをやめて出た・第 99・100 節・GitHub Issue #282） |
| WIN-06 | Windows の影と角丸（DWM） | 1 | 確認済み | FR-025 | 無 | ADR 0013・Windows は Issue #55 で DWM に影（`DwmExtendFrameIntoClientArea`）と角丸（`DWMWA_WINDOW_CORNER_PREFERENCE`）を頼む形にした。施主の Windows の実機で影と角丸を確かめた（gate-proofs 第 99 節）。#282 で `DwmExtendFrameIntoClientArea` をやめた後も、影と角丸は施主の実機で残った（第 100 節・影は Windows 11 が角丸の窓に付けるもの。Windows 10 は確かめていない） |
| WIN-07 | X11 の移動・リサイズ・最大化をウィンドウマネージャーに頼む（`_NET_WM_MOVERESIZE`・`_NET_WM_STATE`・装飾を外す `_MOTIF_WM_HINTS`） | 1 | 確認済み | FR-025 | 無 | ADR 0013・Issue #27・WSLg で施主確認（gate-proofs 第 17 節）。他のウィンドウマネージャーはまだ |
| WIN-08 | システムメニュー（Alt+Space・タイトルバーの右クリック）とキーボードでの最小化 / 最大化 / 閉じる | 1 | 未着手 | FR-025 | 無 | ADR 0013・X11 でも、システムメニューとキーボードでの最小化 / 最大化 / 閉じるはまだ（Issue #27 の残るリスク 4） |
| WIN-09 | 倍率の変更（Per-Monitor v2）でタイトルバー・ボタン・縁の当たり判定が崩れない | 1 | 確認済み | FR-025 | 無 | ADR 0013・固定の画素は倍率の 1 つの経路（`GlyphMetrics::px`）で掛け、`WM_DPICHANGED` で測り直して置き直す（Issue #58）。125% と 150% の画面とその間の移動を施主の実機で確かめた（gate-proofs 第 45 節）。150% の配置はテスト |
| WIN-10 | 全画面（F11） | 2 | 未着手 | FR-025 | 一部 | ADR 0013 |
| WIN-11 | タイトルバーのボタンをスクリーンリーダーが読める（UI Automation） | 2 | 未着手 | FR-025 | 無 | ADR 0013・A11Y-04 |

## 26. 表示言語（LANG）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| LANG-01 | 6 言語の画面（日本語・English・简体中文・繁體中文・한국어・Español）。文言は core の 1 表 | 1 | 未着手 | FR-034 | 少 | ADR 0011・文言は仮の日本語を各所に直に書いている（Issue #21・#41 など）。core の文言の表と 6 言語はまだ |
| LANG-02 | 既定は OS の言語に従う（起動時に観測・分からなければ English） | 1 | 未着手 | FR-034 | 少 | ADR 0011・OS の言語はまだ読まない |
| LANG-03 | 切り替えがその場で全部の語に効き、版付きの設定に残る | 1 | 未着手 | FR-034 | 少 | ADR 0011・切り替えも設定への保存もまだ（設定の v1 に残すのは配色・拡大・詳細ペインだけ・Issue #40） |
| LANG-04 | 言語ごとの字形の選び直し（X11）・地域名の受け渡し（DirectWrite） | 1 | 未着手 | FR-034 | 無 | ADR 0011・字体は Noto Sans CJK JP の固定（Issue #20）。言語ごとの字形の選び直しと DirectWrite はまだ |

## 27. 配色テーマ（THEME）

| 番号 | 機能 | フェーズ | 状態 | FR | 競合 | 根拠 |
| --- | --- | --- | --- | --- | --- | --- |
| THEME-01 | nene-nib の組み込み 125 配色。画面の色はすべて 3 色から導く | 1 | 確認済み | FR-035 | 無 | ADR 0012・Issue #34（gate-proofs 第 25 節・全配色のコントラストのテスト）。Windows は Issue #59（gate-proofs 第 46 節） |
| THEME-02 | 既定は OS の明暗に従う（Windows の設定・X11 の GTK の慣習） | 1 | 確認済み | FR-035 | 一部 | ADR 0012・X11 は Issue #40（gate-proofs 第 29 節）。Windows は Issue #59（gate-proofs 第 46 節）（アプリ モードをダークにして開いた） |
| THEME-03 | 切り替え（名前を打つ・一覧で巡る）がその場で効き、版付きの設定に残る | 1 | 確認済み | FR-035 | 一部 | ADR 0012・起動の `--theme` と一覧で巡るキー（Ctrl+Shift+T）は Issue #34、設定に残すのは Issue #40、名前から選んでその場で試すのは Issue #41。Windows は Issue #59（gate-proofs 第 46 節） |
| THEME-04 | 全配色で字と背景・面・交互の行・選択のコントラスト 4.5 以上（テストで要求）。色だけで意味を伝えない | 1 | 確認済み | FR-035 | 無 | ADR 0012・A11Y-02・Issue #34（core の `theme_derivation` の全配色のテスト。交互の行はまだ無い） |
| THEME-05 | 利用者のテーマファイル（nene-nib の `.v1.theme`） | 3 | 未着手 | FR-035 | 少 | ADR 0012 |

## 28. やらないこと

| 機能 | 理由 |
| --- | --- |
| 型の自動の推し量りと自動の変換 | 製品の約束（FR-003）の否定 |
| 数式・セルの書式・複数のシート | 表計算ではない（SPECIFICATION 第 3 節） |
| データを外に送る機能（クラウド・共同編集・AI・テレメトリ・Web / API からの取り込み） | 経理や顧客のデータを扱う職場で使えなくなる（SEC-05・ADR 0008） |
| 外部の分析エンジン（DuckDB 等）の組み込み | ADR 0007 |
| スクリプト言語の組み込み（AUTO-07） | 依存が大きい。手順（AUTO-01）・SQL（AUTO-06）・コマンドライン（AUTO-05）で繰り返しの要望に応える（ADR 0008） |
| 地図のプレビュー | 範囲外。座標は文字列のまま保つ（KEEP-01） |
