# ADR 0031 — 正規表現の照合は core に置き、`regex-automata`（`no_std`）を使う（ADR 0030 の決定 5 を直す）

- 状態: 受理
- 日付: 2026-11-06（受理 2026-11-06）
- Issue: #169（GitHub）
- 影響する規則: FR-008 / FIND-03 / ARC-001 / ARC-003 / ARC-013 / CNF-007 / CNF-013 / SEC-04 / ADR 0008 / ADR 0030

## 文脈

ADR 0030 の決定 5 は「`regex` クレートを scan に入れる」と決めた（施主が 2026-11-06 に受理）。置換まで入れて（GitHub Issue #161・#163・#165）、正規表現に入ろうとして次に当たった。

- 照合の規則の正本は core の `FindQuery`（`matches` と `replaced`）。core は `#![no_std]`（ARC-003・CNF-007）
- 原本の行は scan の裏の仕事が照らすが、**足した行・書き換えたセル・1 つずつの置換のカーソルのセルは application が照らす**（ADR 0030 の決定 2「見えている値を探す」）。application も `#![no_std]`
- `regex` クレートは `std` を要る。scan にだけ置くと、application からは呼べない

決定 5 のままでは、次のどちらかになる。

1. 照合を 2 か所に持つ（scan は `regex`、application は別の何か）。答えが食い違う道で、ARC-001（1 つのことに 1 つの経路）に反する
2. core に「照合の口」（トレイト）を置き、合成ルートが scan の `regex` を差し込む

調べたこと（2026-11-06）:

- `regex` の中身の `regex-automata` 0.4.14 は `#![no_std]`（`src/lib.rs` の 545 行目）で、`alloc` と `syntax` と `meta` の機能だけで、`regex` と同じ正規表現（`meta::Regex`・逆戻りしない・入力の長さに比例する時間）が使える。ライセンスは `MIT OR Apache-2.0`
- 使い捨てのクレートで測った（release・この機械）:

| 形 | 連れてくる依存 | exe の増え | 組み立て |
| --- | --- | --- | --- |
| `regex-automata`（`alloc`・`syntax`・`meta`・`unicode`） | `regex-syntax` | +1.54 MB | 3.4 s |
| 上 ＋ 文字列の先探しの速い道（`perf-literal-*`） | ＋ `aho-corasick`・`memchr` | +1.83 MB | 3.6 s |

- `#![no_std]` ＋ `extern crate alloc` のライブラリから `meta::Regex::new` と `is_match` を呼んで、組み立てが通ることを確かめた

## 決定

### 1. 照合の規則は core の 1 か所のまま、正規表現も `FindQuery` が持つ

- `FindOptions` に「正規表現」の印を足す（Alt+R・ADR 0030 の決定 3）。`FindQuery::new` が式を組み立て、組み立てられなければ失敗を返す（欄の下に「式が読めません」と出す）
- `matches` と `replaced` は、正規表現でも今と同じ口のまま。scan の裏の仕事も application も同じ `FindQuery` を使う（今と同じ）

### 2. 依存は `regex-automata` を core に（`default-features = false`）

- 機能は `alloc`・`syntax`・`meta`・`unicode`。速い道（`perf-literal-*` と、それが連れてくる `aho-corasick`・`memchr`）は入れない。入れるかは、1 GB で測ってから別に決める
- ARC-013 の手順：`deny.toml` の許可・CODING_RULES 第 5 節の表・CNF-013 の表を、導入の Issue で `cargo tree` と `cargo deny` の実測で更新する

### 3. 全角半角・大文字小文字・かなの畳みと正規表現

- 正規表現は**畳んだ写し**に当てる（`fold_with_origins` の写し・GitHub Issue #161）。式も同じ畳みで畳む（字の並びだけ。`\d` や `[...]` の中の字も同じ畳み）。大文字小文字を区別しないなら、式に `(?i)` を足すのではなく、写しと式の両方を小文字にする（今の照合と同じ答えにする）
- 置換の `$1` などは `regex` の書き方。当たった所は写しの位置を元の範囲に戻す（今の `replaced` と同じ）

### 4. 速さ

- 原本の行の ASCII の速い道（`rules_out_ascii`）は、正規表現では使わない（式の字の並びがレコードに無いとは言えない）。1 GB の時間は導入の Issue で測って benchmarks に残す

## 強制

- **active**（GitHub Issue #171 で入れた）:
  - core の外部クレートは `regex-automata` だけ（CNF-013 の許可表・許可を書く前に `regex-automata への依存は crates/core/ に許されていない` で落ちることを確かめた・gate-proofs 第 80 節）。core が `std` に届かない（CNF-007 のまま）
  - 正規表現でも `matches` と `replaced` が同じ答え、ASCII の速い道と畳んだ写しの道が同じ答え（core のテスト `a_regex_replacement_agrees_with_matching`）。`\D` などを小文字に畳まない・全角を半角に畳む（`a_regex_matches_folded_cells_and_keeps_its_escapes`）。`$1`・`${name}` は元の字で埋める（`a_regex_replacement_fills_groups_with_the_original_characters`）。読めない式はどこにも当たらない（`a_broken_regex_matches_nothing_and_says_so`）
  - 病的な式（`(a*)*b`・`(x+x+)+y`）が 20 万字のセルで終わる（`pathological_patterns_finish_on_long_cells`・SEC-04。時間の数は画面の Issue で 1 GB と一緒に測る）
- **active**（GitHub Issue #173 で入れた）:
  - Alt+R の切り替え（application のテスト `the_find_line_shows_the_regex_state`・ui-x11 の `find_keys`・ui-win32 の `find_keys`）。読めない式では探さず、置き換えず、知らせる（`a_broken_regex_neither_finds_nor_replaces`）。1 GB の時間（benchmarks 第 10 節・検索 4.2〜4.4 s・全部の置換の数え 6.9 s）

## 結果

得られるもの:

- 照合の規則は core の 1 か所のまま（ARC-001）。足した行も原本の行も同じ答え
- core と application の `no_std` を保つ

失うもの・残る穴（**正直に**）:

- core に初めて外の依存が入る（今まで core は依存なし）。exe は 1.5 MB ほど増える
- 速い道を入れないので、正規表現の検索は普通の検索より遅いかもしれない（測ってから決める）
- 決定 3 の「式も畳む」は、式の書き方（`Ａ` のような書き方）までは畳まない

## 判断の前に並べた選択肢

| # | 案 | 良い所 | 悪い所 |
| --- | --- | --- | --- |
| A（推し） | `regex-automata`（`no_std`）を core に | 照合が 1 か所・`no_std` のまま・口を作らない | core に初めて依存が入る |
| B | `regex` を scan に入れ、core に照合の口（トレイト）を置いて合成ルートが差し込む | ADR 0030 の決定 5 の字面どおり・core に依存が入らない | `FindQuery` が差し込んだ物を持つ（等しさ・写し・Debug を手で書く）。口の向こうとこちらで答えが食い違わないことをテストで縛る手間 |
| C | 正規表現は原本の行だけ（scan）で、足した行と書き換えたセルは探さない | 小さい | ADR 0030 の決定 2（見えている値を探す）に反する |

## 施主の判断（2026-11-06・GitHub Issue #169）

推しの A を受理した（施主「Aでいいよ」）。`regex-automata` を core の依存に足すこと（決定 2）も、この受理に含む。導入の Issue で `deny.toml`・CODING_RULES 第 5 節の表・CNF-013 の表を実測で更新する。
