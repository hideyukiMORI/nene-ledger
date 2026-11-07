# ADR 0030 — 検索はパレットの欄の段で、見えている値（原本 ＋ 差分）を裏で探す。置換は 1 手の束にし、全部の置換は件数を見せてから

- 状態: 受理
- 日付: 2026-11-06（受理 2026-11-06）
- Issue: #122（GitHub）
- 影響する規則: FR-008 / FR-010 / ARC-007 / ARC-011 / ARC-013 / FIND-01・02・03・04・05・06 / UNDO-01 / KEEP-01 / BIG-12 / ADR 0005 / ADR 0008 / ADR 0014 / ADR 0017 / ADR 0024 / ADR 0029

## 文脈

M2 は「探して置き換え、壊れた行を直す」束を含む（ROADMAP・GitHub Issue #8 で業務ペルソナ 5 人が「編集と同じ版に要る」と言った）。フェーズ 1 の検索と置換は次の 5 つ。

- FIND-01 検索（大文字小文字・セル全体の一致・列を限る・選択範囲を限る）
- FIND-02 置換（同じ条件・件数の事前表示・プレビュー）
- FIND-03 正規表現
- FIND-05 巨大なファイルでも裏で検索し、見つかった順に出す
- FIND-06 全角半角・ひらがなカタカナを同一視する検索（選べる）

今の土台は次のとおり。

- 見えている表は「原本 ＋ 差分（行の並び・列の並び・セルの書き換え）」（ADR 0005・0024・0027）。画面は窓の分の行しか持たない
- 裏の一括処理は scan のスレッドで、状態は application の状態機械（ADR 0014）。原本の範囲を読んで畳み込む仕事（`RangeJob`・選んだ範囲の集計で使っている）がある
- パレットには文字を打つ段がある（行へ飛ぶ・名前を付けて保存の道筋）
- 外部のクレートは許可制（ARC-013）。正規表現は「依存を入れるか自前か」を導入の Issue で ADR にする約束（ADR 0008 の 5）

## 決定

### 1. 見え方とキー

- **Ctrl+F でパレットの欄に「検索」の段を開く**（行へ飛ぶ Ctrl+G と同じ作り）。欄の下に条件の印（大文字小文字・セル全体・範囲）と件数を出す
- Enter・F3 で次、Shift+Enter・Shift+F3 で前。欄を閉じても F3 で続けて探せる
- Ctrl+H で「置換」の段（検索の欄 ＋ 置き換える文字の欄）
- 見つかったセルへカーソルを動かし、窓に見えるように動かす。表の終わりまで行けば頭から続け、一周したことを知らせる

### 2. 何を探すか

- **見えている値**（原本のセルの値に、セルの書き換えと行と列の並びを重ねたもの）を探す。原本のバイト列ではない
- 足した行・足した列・書き換えたセルは、application が差分から探す。原本の行は、scan が裏で読んで探し、書き換えたセルの元の値は見つけても捨てる
- 見出しの行は探さない（本体だけ）。前書きの行は探す
- 順は行の順（左から右、上から下・Excel の既定の「行」）。始まりはカーソルの次のセル

### 3. 既定の条件と切り替え

| 条件 | 既定 | 切り替え |
| --- | --- | --- |
| 大文字小文字 | 区別しない | Alt+C |
| 一致の仕方 | 部分一致 | Alt+W でセル全体 |
| 範囲 | 表の全体 | Alt+S で選んだ範囲（列全体を選べば列を限る） |
| 全角半角 | 区別しない（Excel の日本語版の既定） | パレットの切り替え |
| ひらがなカタカナ | 区別する | パレットの切り替え |
| 正規表現 | 使わない | Alt+R |

- 条件は設定に残す（次の起動でも同じ・配色と同じ扱い）
- 値は変えない（KEEP-01）。数や日付として比べることはしない（文字列として探す）

### 4. 巨大なファイル（FIND-05）

- 原本の行の検索は scan の裏の仕事（ADR 0014）。カーソルの位置から先へ塊ごとに読み、**最初に見つかった所で止まって知らせる**（次を求められたら続きから）。進み具合を状態の行に出し、Esc で止める
- 「すべて数える」（件数・FIND-04 はフェーズ 2）は、求められたときだけ別の仕事で数える
- 並列にしても結果の順は行の順（ADR 0014 の決定 5）

### 5. 正規表現（FIND-03）は `regex` クレートを scan に入れる

> 2026-11-06：置き場所は [ADR 0031](0031-regex-in-core.md) で core の `regex-automata`（`no_std`）に直した（照合の規則の正本が `no_std` の core にあるため・施主が受理）。

- **`regex` を依存に足す**（scan だけ・ARC-013 の手順で `deny.toml` と表を実測で更新する）。入力の長さに比例する時間で動き、逆戻りで止まらない（利用者が打った式で窓が固まらない・SEC-04）。ライセンスは Apache-2.0 か MIT
- 正規表現の置換の `$1` などは `regex` の書き方に合わせる

### 6. 置換（FIND-02）

- 1 つずつの置換（Enter で置き換えて次へ）と、全部の置換
- **全部の置換は、先に件数を数えて「n 件を置き換えます」と見せ、Enter で置き換える**（プレビューの表はフェーズ 2）
- 置き換えはセルの書き換え（ADR 0024 の層 2）。全部の置換は **1 手の束**（ADR 0029 の決定 3 と同じ形）で、Ctrl+Z 1 回で戻る
- **全部の置換は 100 万セルまで**（ADR 0029 の決定 4 と同じ）。超えれば置き換えずに知らせる。それより大きな置換は、M3 の「変換の規則」（ADR 0005 の「原本 ＋ 差分 ＋ **変換の規則**」・セルごとに持たない層）で扱う
- 置き換えた結果は文字列のまま。表せない字（文字コード）は保存のときに知らせる（今と同じ・SAVE-04）

### 7. 当たりの見え方（2026-11-07 追記・**提案**・GitHub Issue #189）

> 施主の声（2026-11-06・0.1.0-preview.1 を実機で触って）：「選択されている文字が色変わるとか背景色が変わるとかしないと何をやっているのか、どの文字を変えるのかがわからないかも」。施主が外出中に作ったので、この節は**提案**。下の表の推しで作ってあり、施主の判断を待ってからマージする。

- **印を付ける間**：検索の欄と置換の欄が開いている間（打っている途中の問いで、Enter を待たない）と、全部の置換の件数を見せている間。欄を閉じれば消す（F3 で続けて探すときは印を付けない）
- **印を付ける所**：見えている行の本体のセルだけ（見出しの行は探さないので付けない）。画面に描く分だけ照らすので、1 GB の表でも全体を走らせない。**64 KiB を超える値には印を付けない**（描くたびに値の全体を照らすと、描く手間が値の長さに比例するため・独立のレビューの指摘）
- **当たったセルの背景**：選んでいないセル（と帯）を「当たったセル」の薄い黄にする。選んだセルとカーソルのセルは選択の色のまま
- **当たった字**：明るい黄（蛍光ペン）の背景に、濃い字で書き直す（Vim の検索の強調と同じ）。暗い配色でも同じ。`…` で切った後ろに当たりがあれば `…` に印を付ける。改行などの印（`¶`・`→`・`□`）の後ろの位置も合わせる
- **色**：配色の役割を 3 つ足す（当たったセル・当たった字の背景・その上の字）。配色の強調の色に依らない印の黄（`#FFC400`）を背景から混ぜて導く。125 配色すべてで、当たったセルの上の字と当たった字が 4.5 の比で読め、当たった字の背景がほかの面（背景・面・選択・帯・当たったセル）から離れていることをテストで確かめる
- **字の幅**：字の幅は UI が字体で測る（比例の字体では半角 1・全角 2 の近似が合わない）。描く命令に「字の一部に印を付ける」を足し、X11（cairo）と Windows（DirectWrite）の両方で描く
- **表を眺めること**：数えている間と件数を見せている間も、表のスクロール・ドラッグはできる（数えも確かめもやめない）。欄に打つ・条件を替える・Esc はやめる。やめたら、やめる前に数え終えた結果が後から届いても使わない
- **全部の置換の件数と確かめ**：Alt+A で欄を閉じず、置換の欄の 1 行目のキーの案内の代わりに「n セルを置き換えます · Enter 置き換える · Esc やめる」を強調の色で出す。Enter で置き換え、Esc でやめる（欄は開いたまま）。欄に打てば置き換えをやめて打った字が欄に入る。置き換えた後は「n セルを置き換えました · 表で Ctrl+Z で戻す」。長い知らせ（1 MiB で切り詰めた行を飛ばした数など）は今までどおり状態の行

#### 判断の前に並べた選択肢（施主に諮る）

| # | 問い | 推し（作った形） | ほかの案 |
| --- | --- | --- | --- |
| 1 | 印を付ける間 | 欄が開いている間（打つたびに）＋ 件数を見せている間 | Enter で探した後も、次の検索まで残す（VS Code の欄を閉じた後と同じ）／ Enter を押してから |
| 2 | 当たった字の見え方 | 明るい黄の背景 ＋ 濃い字（どの配色でも同じ） | 配色の強調の色の背景（配色に馴染むが、選択の色と紛れる・暗い配色で字が読めなくなる）／ 下線だけ |
| 3 | 当たったセルの背景 | 選んでいないセルを薄い黄 | 付けない（字の印だけ）／ セルの枠 |
| 4 | 件数と確かめの置き場 | パレットの中（置換の欄の 1 行目） | 今までどおり状態の行（欄を閉じる）／ 両方 |
| 5 | 色の役割を足す | 3 つ（配色の数は変えない・導出で作る） | 2 つ（字の色は配色の字のまま・暗い配色で比 4.5 に届かず薄くなる。試して捨てた） |

試して捨てた形（2026-11-07）：当たった字の背景を「背景から黄へ寄せ、字が読めるまで背景へ戻す」で作ると、暗い配色（solarized-dark・abyss など）では当たったセルの色とほぼ同じになり（チャンネルの差の和が 1）、見分けられなかった。字の色を濃くして書き直す形に変えると、全配色で差の和 107 以上・字の比 4.5 以上になった。

## 強制

- **active**（GitHub Issue #151 で入れた）:
  - 書き換えたセルは書き換えた値で探し、元の値では見つからない。並びから消した行は探さない。足した列は書き換えた値で探す（application のテスト `rows_are_searched_in_the_shown_order_with_edits_over_the_source`・app のテスト `edited_values_and_removed_rows_are_respected`）
  - 裏の検索の結果は塊の大きさに依らない（app のテスト `the_first_match_after_the_bound_is_found_in_shown_values` が 1・2・100 レコードの塊で同じ答えを見る。今は 1 つのスレッド）。見つけた所で残りを読まずに止まる（scan のテスト `a_fold_that_is_done_ends_without_reading_the_rest`）
  - 既定の条件と切り替え（core のテスト `the_standard_ignores_case_and_width_and_keeps_kana_and_toggles_flip_one_each`・`options_make_the_match_stricter_or_looser`）。ASCII の速い道は畳んで比べる遅い道と同じ答え（`the_ascii_shortcut_agrees_with_folding`・`a_record_is_ruled_out_only_when_no_cell_can_match`）
- **active**（GitHub Issue #152 で入れた）:
  - 足した行の中も探し、前へ探しても足した行から原本の行へ戻る（application のテスト `finding_uses_edited_values_and_added_rows`）
  - 次を探し、F3 で続け、端から反対の端へ回って知らせる。前へも探せる（application のテスト `finding_moves_to_the_next_match_and_wraps_around`）。F3 は問いが無ければ知らせ、探している間の Esc は止める（`find_again_needs_a_query_and_escape_stops_a_search`）
  - 探す段のキーと条件の切り替え（application のテスト `the_find_step_searches_both_ways_and_toggles_its_options`・ui-x11 と ui-win32 の `find_keys`）
- **active**（GitHub Issue #157 で入れた）:
  - 裏の検索を索引の目印で分けて並列にしても、結果はスレッドの数と分け方に依らない（scan のテスト `a_split_fold_merges_to_the_same_result_for_any_number_of_parts`・`a_split_search_finds_the_earliest_match_for_any_number_of_parts`・1・2・3・4・8 個に分けて同じ答え）
- **active**（GitHub Issue #161・#163 で入れた）:
  - 置き換えは検索と同じ条件で当たった所だけを替え、当たらない所の字は元のまま残す。置き換えの有無は照合と同じ答え（core のテスト `replacing_keeps_the_unmatched_characters_as_they_were`・`replacing_agrees_with_matching`・`origins_point_back_to_the_source_bytes`）
  - 全部の置換は件数を見せてから Enter で 1 手の束として書き、Ctrl+Z 1 回ですべて戻る（application のテスト `replacing_all_shows_the_count_first_and_undoes_in_one_step`）。ほかの意図・Esc でやめ、数えている間に表が変われば書かない・100 万セルを越えれば書かない（`replacing_all_can_be_declined_stopped_or_outdated`）。足した行も置き換え、1 MiB で切り詰めた行は飛ばして数を知らせる（`replacing_all_reaches_added_rows_and_reports_nothing_found`）。1 つずつの置換は 1 手ずつ戻る（`replacing_one_rewrites_the_cursor_cell_then_finds_the_next`）
  - 裏の数えは塊の大きさと分け方に依らず、見出し・書き換えた値・消した行の扱いが検索と同じ（app のテスト `matching_cells_are_collected_in_record_order_over_shown_values`・`edits_removed_rows_and_the_limit_are_respected`）
- **active**（GitHub Issue #165 で入れた）:
  - 置換の段のキー（Ctrl+H・Tab で欄を替える・Enter で 1 つ・Alt+A で全部・Shift+Enter と条件は探す段と同じ）（application のテスト `the_replace_step_types_into_two_fields`・`the_replace_step_replaces_one_or_all_and_finds_like_the_find_step`・`the_replace_step_shows_both_fields_and_its_keys`・ui-x11 と ui-win32 の `replace_keys`）
- **active**（GitHub Issue #189 で入れた・決定 7 は提案）:
  - 当たった所の元の字の範囲は、置き換える所と同じで、当たるときだけ空でない（core のテスト `spans_are_the_parts_that_replacing_rewrites`・`spans_are_the_matched_parts_of_the_original_cell`）
  - 字の一部の印は字体で測った幅で塗り、帯の外と字の境目に無い範囲は塗らない（ui-x11 のテスト `a_text_span_fills_behind_the_measured_characters_inside_its_band`。Windows は `make lint-windows` まで・実機はまだ）
  - 125 配色すべてで、当たったセルの上の字と当たった字が読め、当たった字の背景がほかの面から離れている（core のテスト `every_builtin_theme_marks_found_cells_and_characters_readably`）
  - 当たった字の印は字を書いた後に置き、表示の 1 行（`¶` など）と `…` で切った後ろに合わせる。見出しの行には付けない（application のテスト `found_cells_and_characters_are_marked_after_their_text`・`a_match_past_the_cut_marks_the_ellipsis`・`spans_in_the_value_map_onto_the_single_line`・`spans_past_the_cut_mark_the_ellipsis`）
  - 印を付ける問いは探す段と置換の段だけ（`only_the_find_and_replace_steps_mark_the_table`）。件数と確かめは置換の欄の案内に出し、Enter / Esc / 欄に打つ の扱いが決まっている（`counting_inside_the_open_field_waits_for_the_count`・`replacing_all_is_confirmed_inside_the_open_field`・`replacing_all_inside_the_open_field_can_be_declined`・`the_replace_step_shows_the_count_and_the_confirmation_in_place_of_its_keys`）
- **planned（実装の Issue で active にする）**:
  - 正規表現の逆戻りで止まらない（病的な式と入力の時間を測る・gate-proofs）
  - 1 GB の表で、最後の行にだけある値を見つける時間を DuckDB / qsv / EmEditor と同じファイルで記録（QLT-014。`grep` と `rg` との比べは benchmarks 第 8 節）

## 結果

得られるもの:

- 見えているとおりに探せる（書き換えた値・足した行も）。巨大でも画面が止まらない
- 正規表現が速く安全に使える
- 全部の置換は件数を見てから決められ、1 回で戻せる

失うもの・残る穴（**正直に**）:

- 依存が 1 つ増える（`regex` とそれが連れてくるもの。組み立ての時間と exe の大きさが増える。量は導入の Issue で測る）
- 100 万セルを越える置換は M3 まで無い
- 一致の件数・全一致の選択・一致した行だけを見る（FIND-04）はフェーズ 2

## 施主の判断（2026-11-06・GitHub Issue #122）

下の表の推しを、すべて受理した（施主「うん、全部推しで」）。`regex` を scan の依存に足すこと（決定 5）も、この受理に含む。

### 判断の前に並べた選択肢

| # | 問い | 推し | ほかの案 |
| --- | --- | --- | --- |
| 1 | 見え方 | パレットの欄の「検索」の段（Ctrl+F・Ctrl+H） | 窓の上に残る検索の帯（ブラウザと同じ・表が 1 行狭くなる）／ Excel と同じ別の窓の画面 |
| 2 | 何を探すか | 見えている値（原本 ＋ 差分） | 原本のバイト列（速いが、書き換えた値が見つからない） |
| 3 | 既定の条件 | 大文字小文字を区別しない・部分一致・全角半角を区別しない・かなは区別する | すべて区別する（エディターの既定）／ かなも区別しない |
| 4 | 巨大なファイル | 次の 1 件で止まる（続きは求められたら） | 全部を数えてから最初へ動く（件数がすぐ分かるが、1 GB で待つ） |
| 5 | 正規表現 | `regex` クレートを scan に入れる | 自前の小さな正規表現（依存は増えないが、作る量と逆戻りの危うさ）／ 正規表現はフェーズ 2 へ |
| 6 | 大きな置換 | 100 万セルまで（セルの書き換え）。それより大きいのは M3 の変換の規則 | 上限なし（記憶が置き換えた数に比例）／ 初めから変換の規則で持つ（作る量が大きい） |
