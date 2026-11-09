#!/usr/bin/env bash
# MSIX の 4 桁の版の写し方を確かめる（ADR 0035 の決定 3・GitHub Issue #342）。写せない版は写さずに止まること。
set -euo pipefail
cd "$(dirname "$0")/.."
fail=0
expect() {
  local got
  got="$(bash tools/msix.sh version "$1" 2>/dev/null || echo REFUSED)"
  if [ "${got}" != "$2" ]; then
    echo "msix-check: $1 → ${got}（期待 $2）" >&2
    fail=1
  fi
}
expect 0.1.0-preview.1 0.1.1.0
expect 0.1.0-preview.2 0.1.2.0
expect 0.1.0 0.1.99.0
expect 0.1.1 0.1.199.0
expect 1.2.3-preview.98 1.2.398.0
expect 0.1.0-beta.1 REFUSED
expect 0.1.0-preview.99 REFUSED
# 今の Cargo.toml の版が写せること。
current="$(cargo pkgid -p nene-ledger | sed 's/.*@//')"
bash tools/msix.sh version "${current}" > /dev/null || { echo "msix-check: 今の版 ${current} を写せない" >&2; fail=1; }
[ "${fail}" = 0 ] && echo "msix-check: 版の写し方 7 例と今の版 ${current} → $(bash tools/msix.sh version "${current}")"
exit "${fail}"
