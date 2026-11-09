#!/usr/bin/env bash
# Microsoft Store に出す MSIX を作る（ADR 0035・GitHub Issue #342）。WSL の相互運用で Windows SDK の makepri.exe と
# makeappx.exe を呼ぶ（Linux に同じ道具が無いため）。ストアの側で署名するので、ここでは署名しない。
# 出力: out/msix/NeNeLedger_<4 桁の版>_x64.msix
#
#   tools/msix.sh            … 包みを作る
#   tools/msix.sh version V  … Cargo.toml の版 V（SemVer）を MSIX の 4 桁の版に写して出すだけ（試験と確かめ用）
set -euo pipefail
cd "$(dirname "$0")/.."
CARGO="${CARGO:-cargo}"

# SemVer の版を MSIX の 4 桁の版に写す。ストアの決まりで 4 桁目は 0。3 桁目に「修正の版 × 100 ＋ 先行版の番号」を置き、
# 正式の版は 99 にする（0.1.0-preview.1 → 0.1.1.0・0.1.0-preview.2 → 0.1.2.0・0.1.0 → 0.1.99.0・0.1.1 → 0.1.199.0）。
# SemVer の順と同じ順になる（先行版の番号は 1〜98）。
msix_version() {
  local semver="$1" core pre major minor patch number
  core="${semver%%-*}"
  pre=""
  [ "${core}" != "${semver}" ] && pre="${semver#*-}"
  IFS=. read -r major minor patch <<< "${core}"
  for part in "${major}" "${minor}" "${patch}"; do
    [[ "${part}" =~ ^[0-9]+$ ]] || { echo "msix: 版 ${semver} を読めない" >&2; return 1; }
  done
  if [ -z "${pre}" ]; then
    number=99
  elif [[ "${pre}" =~ ^preview\.([0-9]+)$ ]] && [ "${BASH_REMATCH[1]}" -ge 1 ] && [ "${BASH_REMATCH[1]}" -le 98 ]; then
    number="${BASH_REMATCH[1]}"
  else
    echo "msix: 先行版は preview.1〜preview.98 だけ写せる（${semver}）" >&2
    return 1
  fi
  echo "${major}.${minor}.$((patch * 100 + number)).0"
}

if [ "${1:-}" = "version" ]; then
  msix_version "$2"
  exit 0
fi

if ! command -v cmd.exe >/dev/null || ! command -v wslpath >/dev/null; then
  echo "msix: WSL の相互運用（cmd.exe・wslpath）が無い。Windows SDK の makeappx.exe を呼べない" >&2
  exit 2
fi
kits="/mnt/c/Program Files (x86)/Windows Kits/10/bin"
sdk="$(ls -d "${kits}"/10.* 2>/dev/null | sort -V | tail -1)"
if [ -z "${sdk}" ] || [ ! -x "${sdk}/x64/makeappx.exe" ]; then
  echo "msix: Windows SDK（makeappx.exe）が見つからない: ${kits}" >&2
  exit 2
fi

semver="$("$CARGO" pkgid -p nene-ledger | sed 's/.*@//')"
version="$(msix_version "${semver}")"
# shellcheck source=/dev/null
. packaging/msix/identity.env

make --no-print-directory notices-check
make --no-print-directory build-windows

win() { { cmd.exe /c "$1" 2>/dev/null || true; } | iconv -f CP932 -t UTF-8 -c | tr -d '\r'; }
temp_win="$(win 'echo %TEMP%' | tail -1)"
work="$(wslpath "${temp_win}")/nene-ledger-msix"
layout="${work}/layout"
rm -rf "${work}"
mkdir -p "${layout}/Assets"
cp target/x86_64-pc-windows-gnu/release/nene-ledger.exe "${layout}/"
cp LICENSE THIRD-PARTY-NOTICES.txt "${layout}/"
cp packaging/msix/Assets/*.png "${layout}/Assets/"
sed -e "s|@VERSION@|${version}|" \
    -e "s|@IDENTITY_NAME@|${IDENTITY_NAME}|" \
    -e "s|@PUBLISHER@|${PUBLISHER}|" \
    -e "s|@PUBLISHER_DISPLAY_NAME@|${PUBLISHER_DISPLAY_NAME}|" \
    packaging/msix/AppxManifest.xml.in > "${layout}/AppxManifest.xml"
if grep -q '@[A-Z_]*@' "${layout}/AppxManifest.xml"; then
  echo "msix: 埋めていない値がある" >&2
  exit 1
fi

name="NeNeLedger_${version}_x64.msix"
win_work="$(wslpath -w "${work}")"
# 大きさ違いのロゴ（targetsize）を選べるよう、資源の索引 resources.pri を作る。
"${sdk}/x64/makepri.exe" createconfig /cf "${win_work}\\priconfig.xml" /dq ja-JP /o > /dev/null
"${sdk}/x64/makepri.exe" new /pr "${win_work}\\layout" /cf "${win_work}\\priconfig.xml" \
  /of "${win_work}\\layout\\resources.pri" /o > /dev/null
"${sdk}/x64/makeappx.exe" pack /d "${win_work}\\layout" /p "${win_work}\\${name}" /o > /dev/null

mkdir -p out/msix
cp "${work}/${name}" "out/msix/${name}"
sha256sum "out/msix/${name}"
echo "out/msix/${name}（${semver} → ${version}・Identity: ${IDENTITY_NAME}）"
