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
| 全角半角 | 区別しない（Excel の日本語版の既定） | パレットの切り替え（Alt+Z・GitHub Issue #199・2026-11-08 施主が受理） |
| ひらがなカタカナ | 区別する | パレットの切り替え（Alt+K・同上） |
| 正規表現 | 使わない | Alt+R |

- 条件は設定に残す（次の起動でも同じ・配色と同じ扱い）。2026-11-07 に GitHub Issue #213 で入れた：設定のファイルの版 2 の `find = <区別する条件の並び>`（`case`・`whole`・`width`・`kana`・`regex`）。条件を切り替えたときに書き、その窓で初めて検索と置換の欄を開くときに使う。版 1 のファイルも読む（既定の条件）。強制は application のテスト `find_options_round_trip_and_version_one_files_still_read`
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

### 7. 当たりの見え方（2026-11-07 追記・2026-11-08 施主が受理・GitHub Issue #189）

> 施主の声（2026-11-06・0.1.0-preview.1 を実機で触って）：「選択されている文字が色変わるとか背景色が変わるとかしないと何をやっているのか、どの文字を変えるのかがわからないかも」。施主が外出中に下の表の推しで作り、2026-11-08 に施主が推しのとおり受理した（「全部推しで」）。

- **印を付ける間**：検索の欄と置換の欄が開いている間（打っている途中の問いで、Enter を待たない）と、全部の置換の件数を見せている間。欄を閉じれば消す（F3 で続けて探すときは印を付けない）
- **印を付ける所**：見えている行の本体のセルだけ（見出しの行は探さないので付けない）。画面に描く分だけ照らすので、1 GB の表でも全体を走らせない。**64 KiB を超える値には印を付けない**（描くたびに値の全体を照らすと、描く手間が値の長さに比例するため・独立のレビューの指摘）
- **当たったセルの背景**：選んでいないセル（と帯）を「当たったセル」の薄い黄にする。選んだセルとカーソルのセルは選択の色のまま
- **当たった字**：明るい黄（蛍光ペン）の背景に、濃い字で書き直す（Vim の検索の強調と同じ）。暗い配色でも同じ。`…` で切った後ろに当たりがあれば `…` に印を付ける。改行などの印（`¶`・`→`・`□`）の後ろの位置も合わせる
- **色**：配色の役割を 3 つ足す（当たったセル・当たった字の背景・その上の字）。配色の強調の色に依らない印の黄（`#FFC400`）を背景から混ぜて導く。125 配色すべてで、当たったセルの上の字と当たった字が 4.5 の比で読め、当たった字の背景がほかの面（背景・面・選択・帯・当たったセル）から離れていることをテストで確かめる
- **字の幅**：字の幅は UI が字体で測る（比例の字体では半角 1・全角 2 の近似が合わない）。描く命令に「字の一部に印を付ける」を足し、X11（cairo）と Windows（DirectWrite）の両方で描く
- **表を眺めること**：数えている間と件数を見せている間も、表のスクロール・ドラッグはできる（数えも確かめもやめない）。欄に打つ・条件を替える・Esc はやめる。やめたら、やめる前に数え終えた結果が後から届いても使わない
- **全部の置換の件数と確かめ**：Alt+A で欄を閉じず、置換の欄の 1 行目のキーの案内の代わりに「n セルを置き換えます · Enter 置き換える · Esc やめる」を強調の色で出す。Enter で置き換え、Esc でやめる（欄は開いたまま）。欄に打てば置き換えをやめて打った字が欄に入る。置き換えた後は「n セルを置き換えました · 表で Ctrl+Z で戻す」。長い知らせ（1 MiB で切り詰めた行を飛ばした数など）は今までどおり状態の行

#### 判断の前に並べた選択肢（施主は推しを選んだ）

| # | 問い | 推し（作った形） | ほかの案 |
| --- | --- | --- | --- |
| 1 | 印を付ける間 | 欄が開いている間（打つたびに）＋ 件数を見せている間 | Enter で探した後も、次の検索まで残す（VS Code の欄を閉じた後と同じ）／ Enter を押してから |
| 2 | 当たった字の見え方 | 明るい黄の背景 ＋ 濃い字（どの配色でも同じ） | 配色の強調の色の背景（配色に馴染むが、選択の色と紛れる・暗い配色で字が読めなくなる）／ 下線だけ |
| 3 | 当たったセルの背景 | 選んでいないセルを薄い黄 | 付けない（字の印だけ）／ セルの枠 |
| 4 | 件数と確かめの置き場 | パレットの中（置換の欄の 1 行目） | 今までどおり状態の行（欄を閉じる）／ 両方 |
| 5 | 色の役割を足す | 3 つ（配色の数は変えない・導出で作る） | 2 つ（字の色は配色の字のまま・暗い配色で比 4.5 に届かず薄くなる。試して捨てた） |

試して捨てた形（2026-11-07）：当たった字の背景を「背景から黄へ寄せ、字が読めるまで背景へ戻す」で作ると、暗い配色（solarized-dark・abyss など）では当たったセルの色とほぼ同じになり（チャンネルの差の和が 1）、見分けられなかった。字の色を濃くして書き直す形に変えると、全配色で差の和 107 以上・字の比 4.5 以上になった。

### 8. 当たったセルを箱の下に隠さない（2026-11-09 追記・GitHub Issue #286）

> 施主の外出中に、専門家ペルソナ 3 人の推しで作った（[見直しの記録](../research/2026-11-09-find-replace-ux-review.md)）。施主の確認を待つ。

- 検索と置換の箱が開いている間、見つけたセルが箱の下に入るなら、**箱の下へ出るまで表を送る**（箱と横に重ならない列は送らない）
- 表の端で送れない（箱が上で表の先頭の行・箱が下で表の末尾の行）ときは、**箱を反対の端へ寄せる**（上なら窓の下・下なら上）。寄せた位置は閉じるまでそのまま
- 試して選ばなかった形：VS Code の `editor.find.addExtraSpaceOnTop`（1 行目の上に空きを足す）。表は見出しの行と行の番号を固定して描くので、空きを足すと見出しの固定の作りに手が入る。GNOME Text Editor は同じ問題が未解決（Issue #346）

### 9. 置換の欄は 2 つを縦に並べ、名札を付ける（2026-11-09 追記・GitHub Issue #287）

> 第 8 節と同じく、専門家ペルソナ 3 人の推しで作った。施主の確認を待つ。

- 決定 1 の「Ctrl+H で置換の段（検索の欄 ＋ 置き換える文字の欄）」を、**入力の欄を 2 つ縦に並べる**形にする（前は箱が 1 つで、Tab で中身が入れ替わり、もう一方は灰色の 1 行だった）。左に名札「探す」「置換後」。打っている欄だけ下辺を強調の色にして入力の印を置き、名札も濃くする。欄を押しても打つ先が移る
- **表から Ctrl+H で開くときは、いつも探す欄から**（前は、前に探した文字があると置き換える欄から始まり、どちらに打っているか取り違えた）。検索の欄で打ってから Ctrl+H で移るときは、今までどおり置き換える欄から（GitHub Issue #187・施主が実機で確かめた流れ。2 つの欄と名札で打つ先が見える）
- Ctrl+F と Ctrl+H で前に探した文字を入れて開くときは、**その文字を全部選んでおく**（選択の色で塗る）。打てば置き換わり、Backspace で全部消える。条件の切り替えでは選んだまま、ほかのキーで選びを外す（Excel・VS Code と同じ）
- 置換の段の 1 行目のキーの案内は「Enter 置き換えて次へ · Alt+A 全部 · Tab 欄を替える」（Shift+Enter で前を探すのは探す段と同じなので省いた）

### 10. 条件は名前を固定した札にし、オンとオフを塗りと印で見せる（2026-11-09 追記・GitHub Issue #288）

> 第 8 節と同じく、専門家ペルソナ 3 人の推しで作った。施主の確認を待つ。

- 決定 1 の「欄の下に条件の印」を、文の 2 行（「Alt+C 大小を区別しない · Alt+W 部分一致 …」）から **5 つの札**にする。前は文が「今の状態」か「押すと起きること」か読めず、「区別しない」と「区別する」が混ざっていた
- 札の名前は「区別する」側にそろえて固定する：「大小を区別」「セル全体」「正規表現」「全角半角を区別」「ひら/カタを区別」（2026-11-09 #299 で「かなを区別」から改めた）。右にキー（Alt+C・W・R・Z・K）
- **オン**は強調の色の縁・選択の色の塗り・✓・濃い字。**オフ**は縁の色の枠と薄い字。色だけに頼らず ✓ でも見せる。上の行に 3 つ、下の行に 2 つ（行の数は前の 2 行と同じで、箱の高さは変わらない）
- 札を押しても切り替わる（キーと同じ・設定に残す）
- 正規表現の式が読めなければ、欄の右端に強調の色で「読めない正規表現」（前は条件の文の中）
- 選ばなかった形：VS Code の `Aa` `ab` `.*` の記号だけの札（日本の事務の利用者に記号だけでは伝わりにくい）

### 11. 全部の置換の確かめに、何が何に変わるかの内訳を出す（2026-11-09 追記・GitHub Issue #289）

> 第 8 節と同じく、専門家ペルソナ 3 人の推しで作った（3 人とも最重要に挙げた）。施主の確認を待つ。

- 件数を見せている間、条件の札の行に**内訳**を出す。1 行目：セルの値の「前 → 後」の組を、セルの数の多い順に 2 つ（例：`“980”→“990” 74 セル · “29800”→“29900” 61 セル`。値は 8 桁で切る）。2 行目：残りの組の数と注意（強調の色）
- 注意は 2 つ：置き換える文字が空なら「置換後が空なので、当たった所を消します」。部分一致で値の一部だけが変わるセルがあれば「一部だけ変わるセルあり · Alt+W で全体一致」（正規表現は `$1` などで後の値が変わるので見ない）
- 見つけた事故：980 → 990 の部分一致の全部の置換は、1980 と 29800 も変える（#285 の f4-33count）。内訳で 29800→29900 が 2 番目に見えた
- 選ばなかった形：「セル全体の一致 n・部分一致 m」の数だけ（操作設計の推し）。値が見える方が気づきやすい。プレビューの表（決定 6 でフェーズ 2）は残す

#### 第 11 節の改め（2026-11-09・GitHub Issue #299・専門家ペルソナの 2 巡目）

- 多い順の 2 組では、事故の本命（1980→1990）が「ほか 1 通り」に埋もれた（3 人とも高）。組を **セル全体が変わる組**（後の値が置き換える文字そのもの）と **一部だけ変わる組** に分ける
- 1 行目：セル全体の組のいちばん多い組と残りの組の数（`“980”→“990” 74 セル`）。2 行目：一部だけの組のセルの数と例（強調の色・`一部だけ変わる 112 セル: “29800”→“29900” など`）。空の置換は「置換後が空なので、当たった所を消します」（強調）。3 行になれば 1 行目を落とす。値は 7 桁で切る
- 確かめの間に条件の札（Alt+C・W・R・Z・K）を替えたら、確かめをやめるだけでなく**その場で数え直す**（前はやめるだけで、もう一度 Alt+A が要った）
- 札の名前「かなを区別」を「ひら/カタを区別」に（ひらがなとカタカナの区別だと読み取れなかった）

### 12. 見つからない・置き換えなかったことは、欄の中でその場に言う（2026-11-09 追記・GitHub Issue #291）

> 第 8 節と同じく、専門家ペルソナの推しで作った。施主の確認を待つ。

- 探す段は欄の右端に、置換の段は 1 行目に、強調の色で短く言う：「見つかりません」「当たるセルがありません」「表の端から続けて見つけました」「置き換えをやめました」など。前は「置き換えませんでした（訳は下の状態の行）」と目を状態の行へ送っていた（GNOME HIG の「No results をその場で」）
- 見つからないときは、絞り込む条件のうちオンのもの（大小を区別・セル全体・正規表現・全角半角を区別。既定でオンのかなの区別は除く）を添える：「見つかりません（オン: 大小を区別・セル全体）」
- パレットを開き直せば（Ctrl+F・Ctrl+H・Ctrl+G・Ctrl+Shift+P）、前の検索と置換の知らせを消す（前は行へ飛ぶ段にも「見つかりませんでした」が状態の行に残った）。長い知らせは今までどおり状態の行にも出す

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
- **active**（GitHub Issue #189 で入れた）:
  - 当たった所の元の字の範囲は、置き換える所と同じで、当たるときだけ空でない（core のテスト `spans_are_the_parts_that_replacing_rewrites`・`spans_are_the_matched_parts_of_the_original_cell`）
  - 字の一部の印は字体で測った幅で塗り、帯の外と字の境目に無い範囲は塗らない（ui-x11 のテスト `a_text_span_fills_behind_the_measured_characters_inside_its_band`。Windows は `make lint-windows` まで・実機はまだ）
  - 125 配色すべてで、当たったセルの上の字と当たった字が読め、当たった字の背景がほかの面から離れている（core のテスト `every_builtin_theme_marks_found_cells_and_characters_readably`）
  - 当たった字の印は字を書いた後に置き、表示の 1 行（`¶` など）と `…` で切った後ろに合わせる。見出しの行には付けない（application のテスト `found_cells_and_characters_are_marked_after_their_text`・`a_match_past_the_cut_marks_the_ellipsis`・`spans_in_the_value_map_onto_the_single_line`・`spans_past_the_cut_mark_the_ellipsis`）
  - 印を付ける問いは探す段と置換の段だけ（`only_the_find_and_replace_steps_mark_the_table`）。件数と確かめは置換の欄の案内に出し、Enter / Esc / 欄に打つ の扱いが決まっている（`counting_inside_the_open_field_waits_for_the_count`・`replacing_all_is_confirmed_inside_the_open_field`・`replacing_all_inside_the_open_field_can_be_declined`・`the_replace_step_shows_the_count_and_the_confirmation_in_place_of_its_keys`）
- **active**（GitHub Issue #199 で入れた）:
  - 検索と置換の欄で、全角半角（Alt+Z）とひらがなカタカナ（Alt+K）を 1 つずつ切り替え、案内の行に今の条件を出す（application のテスト `width_and_kana_toggle_one_each`・`the_find_steps_show_the_width_and_kana_conditions`・案内の幅は 32 の条件の組み合わせで `find_and_replace_guides_fit_the_narrowest_palette`・ui-x11 と ui-win32 の `find_keys`）
- **active**（GitHub Issue #286 で入れた）:
  - 箱の下に入る行は箱のすぐ下（下の箱なら上）へ送り、箱と横に重ならない列は送らない。表の端で送れないことが分かる（application のテスト `a_found_cell_is_not_left_under_the_cover`・`only_columns_beside_the_cover_are_covered`・`a_row_under_the_cover_comes_out_just_below_it`）
  - 下へ寄せた箱は、下の端との空きが上と同じで、大きさと行の数は変えない（`a_lowered_panel_sits_at_the_bottom_with_the_same_rows`）
- **active**（GitHub Issue #287 で入れた）:
  - 置換の段は 2 つの欄と名札を描き、打っている欄だけ強調し、Tab と欄を押すことで替わる（application のテスト `the_replace_step_shows_both_fields_and_its_keys`・`only_the_typed_field_is_accented`）
  - Ctrl+H は探す欄から、前の文字は全部選んで打てば置き換わる（`the_replace_step_types_into_two_fields`・`a_previous_query_is_selected_and_typing_replaces_it`・貼り付けと 1 字ずつが同じ `inserting_text_at_once_equals_typing_it`）
- **active**（GitHub Issue #288 で入れた）:
  - 5 つの札を出し、オンの札だけに ✓ と塗りを付ける（32 の条件の組み合わせ・探す段と置換の段・application のテスト `the_find_steps_mark_only_the_conditions_that_are_on`）
  - 札はいちばん狭いパレットでも欄の幅に収まり、押した札の切り替えが分かる。低い窓とほかの段には無い（`the_condition_chips_fit_and_answer_clicks`）
- **active**（GitHub Issue #289 で入れた）:
  - 内訳はセルの数の多い順で、書き換えの並びに依らない。部分一致と空の置換の注意、長い値の切り詰めと行の幅（application のテスト `a_partial_match_shows_the_unintended_pair_and_a_warning`・`an_empty_replacement_warns_and_the_rest_are_counted`・`long_values_are_clipped_to_fit_the_guide`）
  - 件数を見せている間だけ内訳があり、置き換えれば消える（`replacing_all_shows_what_changes_into_what`）。札の行に出し、注意は強調の色（`the_breakdown_takes_the_place_of_the_chips`）
- **active**（GitHub Issue #291 で入れた）:
  - 見つからないことをオンの条件を添えて言い、開き直せば消える（application のテスト `nothing_found_is_said_in_place_with_the_narrowing_conditions`）。探す段は欄の右端に出す（`the_replace_step_shows_the_count_and_the_confirmation_in_place_of_its_keys`）
- **active**（GitHub Issue #299 で入れた）:
  - 一部だけ変わる組は、少なくても必ず強調の行に出る。同じ数の組は前の値の順、答えは並びに依らない。いちばん長い行も案内の幅に収まる（application のテスト `partial_pairs_are_always_shown_apart_from_whole_cell_pairs`・`an_empty_replacement_warns_and_ties_are_stable`・`long_values_are_clipped_to_fit_the_guide`）。確かめの間の条件の切り替えでの数え直しは合成ルート（Xvfb で確かめた・自動の試験は無い）
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
