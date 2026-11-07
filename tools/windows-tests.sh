#!/usr/bin/env bash
# Windows のターゲットの単体テストを、WSL の相互運用で Windows で直に走らせる（GitHub Issue #211）。
#
# `make test` は Linux のターゲットだけを走らせ、`make lint-windows` は Windows のターゲットを組み立てるだけ。ここでは
# テストの exe を `x86_64-pc-windows-gnu` で作り、Windows の一時フォルダに写して走らせる。
#
# テストは組み立てたときの Linux の道筋（`CARGO_MANIFEST_DIR`・`CARGO_TARGET_TMPDIR`）で見本のファイルを開く。Windows は
# `/home/...` のような道筋を「今のドライブの根」からの道筋として読むので、一時フォルダの中に同じ道筋で見本の写しを置き、
# 空いたドライブの文字を `subst` でそこに当て、そのドライブから走らせる。終わったら `subst` を外す。
#
# 🔴 これは単体テストを Windows の OS の上で走らせるだけ。窓・IME・画面の倍率・Excel とのクリップボードの確かめ
# （施主の実機・QLT-013）の代わりにはならない。
set -euo pipefail
cd "$(dirname "$0")/.."
CARGO="${CARGO:-cargo}"

if ! command -v cmd.exe >/dev/null || ! command -v wslpath >/dev/null; then
  echo "test-windows: WSL の相互運用（cmd.exe・wslpath）が無い。WSL の上でだけ走らせられる" >&2
  exit 2
fi

# cmd.exe は WSL の中のフォルダから呼ぶと UNC の警告を出し、失敗の終了コードを返すことがあるので、出力だけを見る。
win() { { cmd.exe /c "$1" 2>/dev/null || true; } | iconv -f CP932 -t UTF-8 -c | tr -d '\r'; }

repo="$(pwd)"
temp_win="$(win 'echo %TEMP%' | tail -1)"
work="$(wslpath "${temp_win}")/nene-ledger-tests"
root="${work}/root"
bin="${work}/bin"
rm -rf "${work}"
# 🔴 ターゲットを指して組み立てたテストの `CARGO_TARGET_TMPDIR` は `target/<ターゲット>/tmp`（`target/tmp` ではない）。
# 無いと、そこへ書くテストが「道筋が無い」で落ちる（ほかのテストが先に作れば通るので、走らせる順で結果が変わった）。
mkdir -p "${bin}" "${root}${repo}/target/x86_64-pc-windows-gnu/tmp"

# テストの exe を作る（X11 のクレートは除く）。
mapfile -t exes < <(
  "$CARGO" test --no-run --locked --workspace \
    --exclude nene-ledger-ui-x11 --exclude nene-ledger-adapters-x11 \
    --target x86_64-pc-windows-gnu --message-format=json 2>/dev/null |
    python3 -I -c '
import json, sys
for line in sys.stdin:
    try:
        message = json.loads(line)
    except ValueError:
        continue
    if message.get("reason") == "compiler-artifact" and message.get("executable") \
            and message.get("profile", {}).get("test"):
        print(message["executable"])
'
)
if [ "${#exes[@]}" -eq 0 ]; then
  echo "test-windows: テストの exe を作れなかった（cargo test --no-run --target x86_64-pc-windows-gnu を見る）" >&2
  exit 1
fi

# 見本のファイルを、組み立てたときと同じ道筋で写す。
while IFS= read -r fixtures; do
  mkdir -p "${root}${repo}/${fixtures}"
  cp -r "${fixtures}/." "${root}${repo}/${fixtures}/"
done < <(find crates -maxdepth 3 -type d -name fixtures)

# 空いているドライブの文字を選んで、一時フォルダの根に当てる。
drive=""
for letter in N O P Q R S T U V W X Y; do
  if [ "$(win "if exist ${letter}:\\ (echo used) else (echo free)" | tail -1)" = "free" ]; then
    drive="${letter}"
    break
  fi
done
if [ -z "${drive}" ]; then
  echo "test-windows: 空いているドライブの文字が無い" >&2
  exit 1
fi
root_win="$(wslpath -w "${root}")"
# 🔴 相互運用は cmd.exe へ渡す引用符を崩す（`"C:\...` が `\C:\...` になる）ので、空白の無い道筋だけを受ける。
case "${root_win}" in *" "*) echo "test-windows: 一時フォルダの道筋に空白がある（${root_win}）" >&2; exit 1 ;; esac
win "subst ${drive}: ${root_win}" >/dev/null
trap 'win "subst ${drive}: /d" >/dev/null' EXIT

failed=0
for exe in "${exes[@]}"; do
  name="$(basename "${exe}")"
  # xtask はリポジトリそのもの（git・文書）を読むので、Windows では走らせない。
  case "${name}" in xtask-*) continue ;; esac
  cp "${exe}" "${bin}/${name}"
  output="$(win "${drive}: && cd \\ && $(wslpath -w "${bin}/${name}")")"
  summary="$(printf '%s\n' "${output}" | grep -E '^test result' | tail -1)"
  echo "${name%%-*}: ${summary:-（結果が無い）}"
  if ! printf '%s\n' "${summary}" | grep -q '^test result: ok'; then
    printf '%s\n' "${output}" | grep -E -A1 'panicked' | head -20
    failed=1
  fi
done
exit "${failed}"
