#!/usr/bin/env bash
# packaging/msix/icon.svg（アイコンの唯一の元）から、MSIX のロゴの PNG と窓のアイコンの BGRA を作り直す。
# 要るもの：rsvg-convert（librsvg）・ImageMagick の convert。ADR 0035 の決定 7・GitHub Issue #375・#450。
set -euo pipefail
cd "$(dirname "$0")/.."
svg=packaging/msix/icon.svg
assets=packaging/msix/Assets
png() { rsvg-convert -w "$1" -h "$1" "$svg" -o "$2"; }
png 44 "$assets/Square44x44Logo.png"
png 71 "$assets/Square71x71Logo.png"
png 150 "$assets/Square150x150Logo.png"
png 310 "$assets/Square310x310Logo.png"
png 50 "$assets/StoreLogo.png"
for side in 16 24 32 48; do
  png "$side" "$assets/Square44x44Logo.targetsize-${side}_altform-unplated.png"
done
# 横長のタイル：生成りの地の真ん中に 150 の正方形のアイコン。
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
png 150 "$tmp/wide.png"
convert -size 310x150 xc:'#F5F2EB' "$tmp/wide.png" -gravity center -composite -alpha off "$assets/Wide310x150Logo.png"
# 窓のアイコン：上の行から・まっすぐのアルファの BGRA（crates/ui-win32/src/window_icon.rs）。
for side in 16 20 24 32 40 48 64; do
  png "$side" "$tmp/icon-$side.png"
  convert "$tmp/icon-$side.png" -depth 8 "BGRA:crates/ui-win32/assets/icon-$side.bgra"
done
