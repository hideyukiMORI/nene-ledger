# Microsoft Store への提出（ADR 0035・GitHub Issue #342）

Win32 の exe を MSIX に包み、Microsoft Store に出すための手順と、済んだこと・施主にしてもらうことの一覧。決めごとの理由は [ADR 0035](../adr/0035-microsoft-store-msix.md)。

## 1. 置き場

| もの | 場所 |
| --- | --- |
| MSIX の宣言の型（`@…@` を埋める） | `packaging/msix/AppxManifest.xml.in` |
| Identity（Partner Center の値） | `packaging/msix/identity.env`（🔴 今は手元で試す仮の値） |
| アイコンの元とロゴ | `packaging/msix/icon.svg`・`packaging/msix/Assets/*.png`（`tools/icons.sh` で作る・比べた画像は `icon/`） |
| 包みを作る | `make msix`（`tools/msix.sh`）→ `out/msix/NeNeLedger_<版>_x64.msix` |
| 版の写し方の確かめ | `make msix-check`（`make check` からも呼ぶ） |
| 認定の検査（WACK） | `tools/wack.ps1`（管理者の PowerShell） |
| 掲載の文 | [listing-ja.md](listing-ja.md)・[listing-en.md](listing-en.md) |
| 個人情報の方針 | [privacy-policy.md](privacy-policy.md) |
| スクリーンショット（1366×768・Windows で撮った） | `screenshots/01.png`〜`04.png` |

## 2. 済んだこと（2026-11-09・実行した結果）

- `make msix` で `NeNeLedger_0.1.2.0_x64.msix`（`0.1.0-preview.1` → `0.1.1.0`）ができた。`makeappx pack` が宣言をスキーマで検査して通った。中身は exe・LICENSE・THIRD-PARTY-NOTICES.txt・ロゴ 10 枚・`resources.pri`
- 施主の機械（開発者モードが有効）で包みの中身を `Add-AppxPackage -Register` で登録し、`Invoke-CommandInDesktopPackage` で `mixed.csv` を渡して包みの中で起動した。窓が正しく描かれ（暗い配色・`PrintWindow` で撮った）、6 秒以上生きていた。確かめた後に `Remove-AppxPackage` で外した
- スクリーンショット 4 枚を、施主の機械の Windows で 1366×768 に広げた窓から撮った（設定は一時のフォルダに替えて、施主の設定を変えずに）。積んだ PR #341 の時点の exe（検索と置換の見直しが入った版）
- 通信しないことは CNF-016（外へ送る API を書けない）が守っている（個人情報の方針の裏づけ）
- exe の依存は Windows の DLL だけ（`objdump -p`）。インストール先（読み取り専用）には書かない（設定は `%APPDATA%`・保存はファイルの隣の一時ファイルから置き換え）

## 3. 施主にしてもらうこと（順に）

1. **Partner Center に開発者として登録する**（個人なら無料）：<https://partner.microsoft.com/dashboard/registration>
2. **アプリの名前を予約する**：「アプリとゲーム → 新しい製品 → MSIX または PWA アプリ」で `NeNe Ledger`
3. **Identity の 3 つの値を `packaging/msix/identity.env` に写す**：製品の「製品 ID」の `Package/Identity/Name`・`Package/Identity/Publisher`・`Package/Properties/PublisherDisplayName`。写したら `make msix` で作り直す（リナに頼めば写して PR にする）
4. **WACK を走らせる**：管理者の PowerShell で
   ```powershell
   cd <リポジトリのフォルダ>   # WSL のリポジトリなら \\wsl.localhost\<distro>\…
   powershell -ExecutionPolicy Bypass -File tools\wack.ps1 -Package out\msix\NeNeLedger_0.1.2.0_x64.msix
   ```
   10〜20 分。結果（総合と落ちた検査）をリナに渡せば gate-proofs に記録する
5. **提出の項目を埋める**（ADR 0035 の「施主に決めてもらうこと」の推しのとおりなら）：
   - 値段：無料・市場：すべて
   - 区分：仕事効率化（Productivity）
   - 年齢区分：IARC の質問票（暴力・課金・利用者どうしのやりとり 無し・ネットワーク 使わない）
   - 個人情報の方針の URL：[privacy-policy.md](privacy-policy.md) を GitHub で開いた URL（`https://github.com/hideyukiMORI/nene-ledger/blob/main/docs/store/privacy-policy.md`）
   - 追加のライセンス条項：`LICENSE`（ADR 0009）の本文
   - 掲載の文：[listing-ja.md](listing-ja.md)（日本語）・[listing-en.md](listing-en.md)（英語）
   - スクリーンショット：`screenshots/01.png`〜`04.png`
   - 包み：`out/msix/NeNeLedger_<版>_x64.msix`（Identity を替えて作り直したもの）
6. 提出して審査を待つ（数日）

## 4. main に入ったもの（2026-11-09）

- 検索・置換・パレット・表の画面の見直し（#346〜#372）・この包み（#373）・引数なしの起動と Ctrl+O（#374）・窓のアイコン（#375）・F8 で壊れた行へ（#377）。main で `make msix` を作り直した
- 次のプレビュー版 `0.1.0-preview.2` の準備（版と `docs/release/0.1.0-preview.2.md`）。MSIX の版は `0.1.2.0`
- スクリーンショットは #374・#375・#377 の前の版で撮った（画面の見た目は同じ）

## 5. 残っていること（正直に）

- **WACK はまだ走らせていない**（`appcert.exe` は昇格が要り、WSL から起動できない。UAC の確認を施主の画面に勝手に出さなかった）
- Identity は仮の値。ストアに出す包みは、施主が名前を予約してから作り直す
- アイコンは 2026-11-09 に NeNe の兄弟と同じ作法で作り直した（ADR 0035 の決定 7）。替えるなら `icon.svg` を差し替えて `tools/icons.sh`。ストアのスクリーンショット（`screenshots/`）は前のアイコンの時に撮ったもの
- タグから CI で MSIX を作る手順はまだ（ADR 0035 の planned）
- ストアの「仕事効率化」の区分・年齢区分・審査の可否は、出してみるまで分からない
