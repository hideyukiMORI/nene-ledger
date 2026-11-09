# ADR 0035 — Microsoft Store には、Win32 の exe をそのまま MSIX に包んで出す。署名はストアに任せ、版は 4 桁に写す

- 状態: 提案
- 日付: 2026-11-09
- Issue: #342（GitHub）
- 影響する規則: ADR 0002 / ADR 0009 / ADR 0032（決定 5「先の道は Windows Store」）/ ADR 0025 / QLT-013 / ARC-013 / DEVELOPMENT_WORKFLOW 第 10 節

## 文脈

ADR 0032 はプレビュー版を GitHub Releases の zip で出し、決定 5 で「コード署名は買わない。先の道は Windows Store（ストアの側で署名される）。包み方（MSIX）は別に決める」とした。施主の依頼（2026-11-09）：「Microsoft Store リリースに向けて足りないところを対応して」。

2026-11-09 に調べた、足りなかった物〔コードと施主の機械を実測〕:

- 包み（MSIX）の宣言・ロゴ・作る入口が無い。アプリのアイコンそのものが無い（タイトルバーは自前で描き、exe に資源の .ico も無い）
- 版が SemVer の `0.1.0-preview.1` だけで、MSIX の 4 桁の版（4 桁目はストアの決まりで 0）が無い
- ストアの掲載の文・スクリーンショット・個人情報の方針・提出の手順が無い
- 道具はある：施主の機械に Windows SDK 10.0.26100（`makeappx.exe`・`makepri.exe`・`signtool.exe`）と App Certification Kit（`appcert.exe`）。exe の依存は Windows の DLL だけ（`objdump -p` で KERNEL32・USER32・d2d1・dwrite・imm32 などだけ）
- 開発者モードが有効（`AllowDevelopmentWithoutDevLicense = 1`）なので、包みの中身を登録して試せる

## 決定

### 1. Win32 の exe をそのまま MSIX に包む（desktop bridge・`runFullTrust`）

- 宣言は `packaging/msix/AppxManifest.xml.in`。`EntryPoint="Windows.FullTrustApplication"`、`rescap:runFullTrust`。x64 だけ。対象は Windows 10 1809（10.0.17763）以降
- 中身は `nene-ledger.exe`・`LICENSE`・`THIRD-PARTY-NOTICES.txt`・ロゴ・`resources.pri`（大きさ違いのロゴを選ぶ索引）
- **ストアの側で署名する**ので、手元の包みは署名しない（ADR 0032 の決定 5 と同じ考え）
- 設定は今までどおり `%APPDATA%` に書く。MSIX が包みごとに分ける（アンインストールで消える）。保存は利用者のファイルの隣の一時ファイルから置き換え（ADR 0025）で、包みのフォルダ（読み取り専用）には書かない

### 2. CSV と TSV の「プログラムから開く」に出す

- `windows.fileTypeAssociation` で `.csv`・`.tsv`・`.tab`。開くファイルは引数で渡る（今の起動の入口のまま）。既定のアプリにするかは利用者が選ぶ（ストアのアプリが勝手に既定を奪わない）

### 3. 版は Cargo.toml の SemVer から 4 桁に写す

- `X.Y.Z-preview.N` → `X.Y.(Z×100+N).0`、正式の `X.Y.Z` → `X.Y.(Z×100+99).0`（`0.1.0-preview.1` → `0.1.1.0`・`0.1.0` → `0.1.99.0`・`0.1.1` → `0.1.199.0`）
- SemVer の順とストアの版の順が同じになる（ストアは前より大きい版しか受けない）。先行版の番号は 1〜98。ほかの先行版の名前（`beta` など）は写さずに止める
- 版の入力は今までどおり Cargo.toml だけ（DEVELOPMENT_WORKFLOW 第 10 節）。写すのは `tools/msix.sh` の 1 か所

### 4. 作る入口は `make msix`（WSL から Windows SDK を呼ぶ）

- `tools/msix.sh` が exe を作り、Windows の一時フォルダに並べ、`makepri.exe` と `makeappx.exe` を WSL の相互運用で呼ぶ（Linux に同じ道具が無い）。出力は `out/msix/NeNeLedger_<版>_x64.msix`
- 認定の検査（WACK）は管理者の PowerShell で `tools/wack.ps1`（`appcert.exe` は昇格が要る）
- **今は手元で作る**。タグから CI で作るのは、Partner Center の Identity が決まり、windows の runner で `makeappx` を回せることを確かめてから（下の planned）

### 5. Identity（名前と発行者）は Partner Center の値

- `packaging/msix/identity.env` に `IDENTITY_NAME`・`PUBLISHER`・`PUBLISHER_DISPLAY_NAME`。今は手元で試す仮の値（`NeNeLedger.LocalTest`）。**施主が Partner Center でアプリの名前を予約したら、「製品 ID」の値に替える**

### 6. 個人情報の方針は「集めない・通信しない」

- `docs/store/privacy-policy.md`（日本語と英語）。通信しないことは CNF-016（外へ送る API はどこでも書けない・ほかのプログラムを起こすのは新しい窓だけ）が機械で守っている

### 7. アイコン

- `packaging/msix/icon.svg`（唯一の元・viewBox 0 0 256 256・角丸の外は透明）。**NeNe の兄弟（Clock・Folio・Ruler）と同じ作法**で描く：Clock の 3 色（ほぼ黒 `#1A1917`・生成り `#F5F2EB`・琥珀 `#D08C3F`）だけ、タイルは余白なしの角丸 `rx=24`、輪郭 3px（Folio と同じ）、中は太い幾何学形を数個
- 図柄：**生成りのタイルに、黒い値の箱 1 つと琥珀の大きなカンマ（「値,」＝CSV の区切り）**。兄弟と同じく、画面の絵でなく記号を 1 つ取り出し、形は 2 つ（Clock＝丸 2 つ・Ruler＝L 字 2 つ）。16 px でも「四角とカンマ」が残る。カンマは文字でなく丸と尾の幾何学形で起こす
- タイルの色は兄弟と被らない（Clock・Ruler は黒、Folio は琥珀、Ledger は生成り）
- ロゴの PNG と窓のアイコンの BGRA は `tools/icons.sh` で `icon.svg` から作る（`rsvg-convert`・ImageMagick）
- 選んだ経緯（2026-11-09・施主「アプリのアイコンをもう少し練って。/design で。NeNeClock や NeNeFolio, NeNeRuler と並べて違和感ないように」）：
  1. 1 巡目：兄弟の作法で「表の格子」の 6 案を描いた（`docs/store/icon/round1-grid.png`）→ 施主「うーん」
  2. web の資料（Microsoft のアイコンの指針「比喩は 1 つ、多くて 2 つ」「16 px で見分けられるのは形 1 つと色の差 1 つ」・兄弟は土台をそろえて比喩を変える）と、アイコンデザイナーと Windows の UX の専門家のペルソナに聞いた。3 つとも「格子は画面の絵で、記号を取り出す兄弟から浮き、Excel と並ぶと埋もれる」、専門家は 2 人とも 1 位に「カンマ」
  3. 2 巡目：カンマ・締め線・検印・行×列の 6 案（`round2.png`・`round2-small.png`）。値の箱 2 つの間のカンマは顔に見えたので、箱 1 つ＋大きなカンマにした。施主が「3」（生成りの地）を選んだ（`final-lineup.png`・`final-small.png`）
- 前の仮のアイコン（`#E95420` の大きな角丸に白い格子）は兄弟から浮いていた

## 強制

- **active**:
  - 版の写し方（`tools/msix.sh version`・`make msix-check` が `0.1.0-preview.1 → 0.1.1.0` など 5 例と、写さずに止める 2 例を確かめる）
  - 宣言の形（`makeappx pack` が AppxManifest のスキーマで検査する。埋めていない `@…@` が残れば `tools/msix.sh` が止まる）
  - 包みの中で起動できる（2026-11-09：開発者モードで包みの中身を登録し、`Invoke-CommandInDesktopPackage` で `mixed.csv` を渡して起動・窓を `PrintWindow` で撮った・`docs/store/README.md`）
- **planned**:
  - WACK の合格（施主が管理者の PowerShell で `tools/wack.ps1` を走らせ、結果を gate-proofs に記録してから active）
  - タグから CI で MSIX を作る（Identity が決まってから）
- **不能**: ストアの審査の合否・年齢区分（IARC の質問票）・掲載の可否

## 結果

得られるもの:

- 署名を買わずに配れる（ストアが署名する）。SmartScreen の警告が出ない。自動の更新とアンインストールがストアに付く
- CSV と TSV を「プログラムから開く」で開ける

失うもの・残る穴（**正直に**）:

- 包みは WSL の相互運用と施主の機械の Windows SDK に頼る（CI はまだ）
- ストアのスクリーンショットは窓が 1024×640 で、ストアの推す 1366×768 より小さい（`docs/store/README.md` に撮り方を書いた）
- ストアの「標準のアプリのライセンス条項」と独自のライセンス（ADR 0009）の関係は、施主が Partner Center で独自の条項を出すかを決める
- Linux の利用者には関係しない

## 施主に決めてもらうこと（提案）

| # | 問い | 推し | ほかの案 |
| --- | --- | --- | --- |
| 1 | Partner Center のアカウント | 個人（無料） | 会社（有料・D-U-N-S が要る） |
| 2 | ストアの名前 | 「NeNe Ledger」を予約 | 日本語の名前を添える |
| 3 | 値段 | 無料（ADR 0009：販売は禁止） | — |
| 4 | 出す版 | 0.1.0 を待たず、プレビュー版を「早期アクセス」の説明つきで | 0.1.0 まで待つ |
| 5 | ライセンスの条項 | 独自の LICENSE（ADR 0009）を Partner Center の「追加のライセンス条項」に出す | ストアの標準の条項だけ |
| 6 | 市場 | すべての市場（説明は日本語と英語） | 日本だけ |
| 7 | 区分 | 仕事効率化（Productivity） | 開発者ツール |
| 8 | アイコン | ✅ 2026-11-09 に兄弟と同じ作法で作り直した（決定 7） | — |

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| ストアに exe / MSI のインストーラーの URL を出す（MSIX でない提出） | インストーラーはこちらで署名が要る（ADR 0032 の決定 5 で買わないとした）。更新もストアが面倒を見ない |
| コード署名の証明書を買って zip のまま配る | 年に費用がかかる。施主「windowsStore で出せば署名いらない」（2026-11-06） |
| 版の 4 桁目に先行版の番号を置く（0.1.0.1） | ストアは 4 桁目を 0 に限る |
| Linux の `makemsix`（msix-packaging）で作る | 組み立てて入れる手間に対し、施主の機械に Windows SDK がある。CI で作るときに改めて比べる |
