NeNe Ledger（プレビュー版）
==========================

大きな CSV / TSV を、開いた瞬間に表として見せ、触っていない所は 1 バイトも変えずに
直して保存する、Windows の表のエディターです。これはプレビュー版です。
できること・まだのことは、GitHub のリリースの説明を見てください。

  https://github.com/hideyukiMORI/nene-ledger/releases

■ 使い方
  1. この zip を好きなフォルダに展開します（インストールは要りません）。
  2. CSV ファイルを nene-ledger.exe の上にドラッグして落とすか、
     nene-ledger.exe を開いてファイルを選びます。samples フォルダに見本があります。
  3. Ctrl+Shift+P（または F1）で操作の一覧が開きます。
     「キーの一覧を見る…」でキーの割り当てを、「このアプリについて…」で版とライセンスを見られます。

■ 最初に開くとき
  この exe には電子署名がありません。Windows が「Windows によって PC が保護されました」と
  止めたら、「詳細情報」を押してから「実行」を押してください。

■ 主なキー
  Ctrl+F 検索 / Ctrl+H 置換 / F3 次を探す / Ctrl+Z 元に戻す / Ctrl+Y やり直す /
  Ctrl+S 保存 / Ctrl+Shift+S 名前を付けて保存 / Ctrl+N 新しい表 /
  Ctrl+C コピー / Ctrl+X 切り取り / Ctrl+V 貼り付け / F2 またはダブルクリックでセルを直す

■ ライセンス
  NeNe Ledger License 1.0（LICENSE を見てください）。使うのは自由・改変はクレジット必須・
  配布は無償に限る・販売は禁止・無保証です。
  使っている外部の部品の著作権表示は THIRD-PARTY-NOTICES.txt にあります。

■ 不具合の報告
  https://github.com/hideyukiMORI/nene-ledger/issues

--------------------------------------------------------------------------------

NeNe Ledger (preview)

A table editor for Windows that shows large CSV / TSV files instantly and saves edits
without changing a single byte of the rows you did not touch. This is a preview release;
see the release notes on GitHub for what works and what does not yet.

- Unzip anywhere (no installer). Drop a CSV file onto nene-ledger.exe, or open the exe
  and choose a file. Sample files are in the "samples" folder.
- Ctrl+Shift+P (or F1) opens the command palette, including the key list and "About".
- The exe is not code-signed. If Windows SmartScreen stops it, click "More info" and
  then "Run anyway".
- License: NeNe Ledger License 1.0 (see LICENSE): free to use, credit required for
  modified versions, redistribution only free of charge, no sale, no warranty.
  Third-party notices are in THIRD-PARTY-NOTICES.txt.
- Issues: https://github.com/hideyukiMORI/nene-ledger/issues
