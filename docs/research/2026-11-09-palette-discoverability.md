# コマンドパレットで「どんな機能があるか」を分かるようにする — /design・専門家ペルソナ・文献（2026-11-09）

> 施主の依頼（2026-11-09）：「Ctrl+P でコマンドパレット出るじゃん？でもこれ、どんな機能があるのか触り始めた人やコマンドパレットに慣れてない人はわからないよね。改善方法無いかな。/design と専門家ペルソナに相談してみて」「web で文献を調べたりもして」。
> 起点は `main` `9b300c2`（0.1.0-preview.2）。画面は Xvfb（1024×640・既定の配色）。

## 1. 問題（相談の前に確かめたこと）

- **メニューバーも右クリックのメニューも無い**。機能を見つける道は、パレット（Ctrl+Shift+P・F1）と、状態の行の右端の灰色の「Ctrl+Shift+P でコマンド」だけ
- 何も打っていない一覧は **分類なしの 47 個**。開いた直後の 10 行は移動・選択・文字の大きさで、**保存・開く・探す・置換は 25〜33 番目**
- スクロールバーが無く、「47 / 47」だけでは下に続くことが分からない
- 施主は「Ctrl+P」と覚えていたが、開くキーは Ctrl+Shift+P（FEATURES の AUTO-03：「Ctrl+P は印刷 PRNT-01 に空ける」施主 2026-10-26）

## 2. 誰に聞いたか・答えの重なり

| 改善 | /design の評価 | UX（見つけやすさ） | Excel 派の事務 | キーボード道具の設計 | 文献 | どうしたか |
| --- | --- | --- | --- | --- | --- | --- |
| 空の一覧を「よく使う」＋分類の見出しの目録に | 1 位 | 1 位 | 2 位 | 1・2 位 | 推し A（Superhuman「既定の点数」） | #420 |
| 案内を押せる・目立たせる | 2 位 | 2 位 | 3 位 | 4 位 | — | #422 |
| Ctrl+P でも開く | 2 位 | 3 位 | — | 4 位 | — | 🔴 割り当てない（施主の過去の決定・確かめる） |
| スクロールバー・下に続く印 | — | 指摘 | — | 3 位 | Superhuman | #424 |
| Excel の言葉で引ける・無い機能は無いと言う | — | 5 位 | — | — | 推し B（別名・Office の Tell me） | #426 |
| 欄の例を初心者の言葉に | — | — | 指摘 | — | — | #428 |
| 見える入口（タイトルバーに「≡ 操作」） | — | 2 位 | 1 位（タブ） | — | — | 🔴 まだ（施主に確かめる） |
| 初回だけの 1 行の案内 | 3 位 | 4 位 | — | ✕（吹き出しはやめる） | 推し C（流れの中で教える） | まだ |

**全員が「やらないで」と言ったこと**：分類を折りたたみの木にする・打って絞る間も見出しを残す・使った回数で並びを毎回変える（動くのは「最近」の 5 つだけ）・開くたびに出るツアー・リボンや大きなツールバーを足す。

## 3. 直したこと（すべて main に入れた）

| Issue | 中身 | PR |
| --- | --- | --- |
| #420 | 何も打っていない一覧を「最近使った操作」→「よく使う」（開く・保存・探す・置換・元に戻す）→ 8 つの分類（ファイル・探す・置き換える・編集・行と列・移動と選択・読み方・文字コード（文字化けしたとき）・表示・ヘルプ）の見出し付きに。見出しは選べず ↑↓ は飛ばす。打って絞る間は見出しを出さない | #421 |
| #422 | 状態の行の案内を「F1 · Ctrl+Shift+P で操作の一覧」（F1 は 1 キーで Windows のヘルプの慣習）にし、本文の字の色に。押せば一覧を開く | #423 |
| #424 | パレットの候補が見せている数より多ければ、行の右にスクロールバー | #425 |
| #426 | 「並べ替え」を列の移動の言い換えから外し（ソートと取り違える）、並べ替え・絞り込み・重複の削除・印刷を打てば「まだありません（今後の版で足す予定です）」 | #427 |
| #428 | 欄の例を「したいことを打って探す（例: 保存・置換・文字化け）」に | #429 |

確かめ方は各 PR（`make lint`・`make conformance`・`make doc-check`・`make test` と Xvfb の画面）。**Windows の実機では見ていない**。

## 4. 文献（調査の要約）

- Superhuman の設計記事：何も打っていないときは操作ごとの「既定の点数」で主な機能を先に、別名で引ける、最後の行を途中で切って下に続くことを見せる、今の画面に合う操作を上げる
- NN/g：キーはメニューの右に目立たない書式で添える（上級者は見つけ、初心者は無視できる）。パレットそのものの記事は見つからなかった
- Office の検索（Tell me）：言い換えで当たる・最近の操作・今使えない操作は灰色。Slack・GitHub・VS Code：分類したキーの一覧・`?` で使い方
- 研究：ExposeHK（CHI 2013・修飾キーでキーの印を重ねる）、Grossman ら（CHI 2007・キーの学習）、Krisler & Alterman（2008・能動的利用者のパラドックス）、CommunityCommands（UIST 2009・ネットワークを使うので当てはまらない）
- 日本の事務職についての研究は見つからなかった

出典：
- https://www.nngroup.com/articles/ui-accelerators/
- https://www.nngroup.com/articles/flexibility-efficiency-heuristic/
- https://blog.superhuman.com/how-to-build-a-remarkable-command-palette/
- https://help.superhuman.com/article/478-superhuman-command
- https://code.visualstudio.com/docs/getstarted/userinterface
- https://docs.github.com/en/get-started/accessibility/github-command-palette
- https://slack.com/help/articles/115003340723
- https://support.microsoft.com/en-gb/office/do-things-quickly-with-tell-me
- https://www.eweek.com/enterprise-apps/microsoft-updates-powerpoint-excel-online-with-tell-me-command-search/
- https://hal.archives-ouvertes.fr/hal-01894253 （ExposeHK）
- https://www.dgp.toronto.edu/~ravin/papers/chi2007_learninghotkeys.pdf
- https://research.autodesk.com/app/uploads/2023/03/communitycommands-command-recommendations-for.pdf_recZiyrKCEIuGQLZh.pdf
- https://researchr.org/publication/KrislerA08

## 5. 施主に決めてもらうこと

| # | 問い | 推し | ほかの案 |
| --- | --- | --- | --- |
| 1 | Ctrl+P でもパレットを開くか | 印刷（PRNT-01）を作るまでは Ctrl+P で開き、印刷を作るときに見直す（3 人とも推した。Excel の人が印刷と思って押しても、一覧から印刷が「まだ」と分かる） | 今までどおり空ける（2026-10-26 の決定） |
| 2 | タイトルバーに見える入口（「≡ 操作」）を描くか | 描く（マウスで探す人の入口・Excel の「ファイル」の位置） | 状態の行の案内（#422）だけで足りるか、実機で様子を見る |
| 3 | 初回だけの 1 行の案内 | 2 を入れるなら不要 | 初めてファイルを開いたとき 1 回だけ、× で消せる 1 行 |
