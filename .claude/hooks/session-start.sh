#!/bin/bash
# Claude Code on the web のセッション開始時に、ゲートが要る道具を揃える。
# 🔴 検査そのものは Makefile と xtask にある。ここは道具の導入だけ（QLT-005）。
# 版は rust-toolchain.toml が決める。ここには書かない（QLT-011）。
set -euo pipefail

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

cd "$CLAUDE_PROJECT_DIR"

# rust-toolchain.toml の版を rustup が入れる（既に在れば何もしない）。
rustup show active-toolchain >/dev/null 2>&1 || rustup toolchain install --profile minimal
cargo --version

# make deny / make coverage が使う。既に在れば飛ばす（コンテナの状態は cache される）。
command -v cargo-deny >/dev/null 2>&1 || cargo install cargo-deny --locked
command -v cargo-llvm-cov >/dev/null 2>&1 || cargo install cargo-llvm-cov --locked

# 表示の確認（Xvfb の上で窓を撮る）と Windows の exe（mingw-w64・make build-windows）に要る。apt が使えない環境では黙って飛ばす。
if command -v apt-get >/dev/null 2>&1 && [ "$(id -u)" = "0" ]; then
  missing=""
  for package in libcairo2-dev pkg-config xvfb imagemagick x11-apps x11-utils xdotool xcompmgr mingw-w64; do
    dpkg -s "$package" >/dev/null 2>&1 || missing="$missing $package"
  done
  if [ -n "$missing" ]; then
    apt-get update -qq >/dev/null 2>&1 || true
    # shellcheck disable=SC2086
    apt-get install -y -qq --no-install-recommends $missing >/dev/null 2>&1 || echo "apt-get could not install:$missing"
  fi
fi

git config --local core.hooksPath .githooks
make doctor
