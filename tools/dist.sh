#!/usr/bin/env bash
# 配る zip を作る（ADR 0032・GitHub Issue #183）。手元（施主が実機で試す候補）と release の workflow の両方がこれを呼ぶ。
# 中身: nene-ledger.exe・LICENSE・THIRD-PARTY-NOTICES.txt・README.txt・samples/。出力は dist/ に zip と sha256。
set -euo pipefail
cd "$(dirname "$0")/.."
CARGO="${CARGO:-cargo}"

version="$("$CARGO" pkgid -p nene-ledger | sed 's/.*@//')"
name="nene-ledger-${version}-windows-x86_64"

# 依存を変えたのに著作権表示の一覧を作り直していなければ、ここで止まる。
make --no-print-directory notices-check
make --no-print-directory build-windows

stage="dist/${name}"
rm -rf "${stage}" "dist/${name}.zip" "dist/${name}.zip.sha256"
mkdir -p "${stage}/samples"
cp target/x86_64-pc-windows-gnu/release/nene-ledger.exe "${stage}/"
cp LICENSE THIRD-PARTY-NOTICES.txt "${stage}/"
cp docs/release/README.txt "${stage}/README.txt"
for sample in screenshot.csv shift-jis-pipe.csv utf16le-tab.txt keep-values.csv; do
  cp "crates/app/fixtures/${sample}" "${stage}/samples/"
done
# 改行を Windows に合わせる（メモ帳でも読めるように）。原本の見本は変えない。
for text in LICENSE THIRD-PARTY-NOTICES.txt README.txt; do
  sed -i 's/\r\?$/\r/' "${stage}/${text}"
done
(cd dist && zip -q -r -X "${name}.zip" "${name}")
(cd dist && sha256sum "${name}.zip" > "${name}.zip.sha256")
echo "dist/${name}.zip"
cat "dist/${name}.zip.sha256"
