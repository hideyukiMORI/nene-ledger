# NeNe Ledger のビルドと品質ゲート。
#
# 🔴 判定はここと xtask にある。CI は `make ci` を呼ぶだけで、CI 側にだけ存在する検査を作らない（QLT-005）。
# 🔴 道具の版は rust-toolchain.toml が決める。ここにも CI にも版を書かない（QLT-011）。
# 🔴 全件（make check）は既定ではない。差分に必要な検証を個別の target で選び、
#    全件は限定した検証で影響を覆えない理由を REASON= に添えたときだけ回す（QLT-001 / ADR 0003）。
CARGO ?= cargo

.PHONY: all dist msix msix-check test-windows notices notices-check check ci fmt fmt-check lint lint-windows build-windows test conformance validate-git coverage deny doc-check build prove screenshot bench-pollock fetch-corpus bench-corpus doctor clean

all: doctor

## check — 全件。REASON が空なら実行前に拒む。
check:
	@if [ -z "$(REASON)" ]; then \
	  echo "QLT-001: 全件検証には理由が要る。例: make check REASON='共通の lint 集合を変えたので全クレートに影響する'"; \
	  exit 2; \
	fi
	@echo "全件検証: $(REASON)"
	$(MAKE) fmt-check lint test conformance deny doc-check notices-check msix-check build

## ci — CI の必須 check。Git 規約・PR の検証記録・絶対パス・差分の空白だけを見る（QLT-005 / QLT-012・ADR 0021）。
## 手元でも同じ 1 行で走る（origin/main が要る）。
ci: validate-git
	git diff --check origin/main...HEAD

## fmt — rustfmt に設定ファイルを置かない（QLT-004）。
fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all --check

## lint — 規則との対応は docs/CODING_RULES.md。
## 🔴 --all-targets を外さないこと。外すとテストコードが検査対象から落ちる。
## 🔴 --locked は Cargo.lock が更新される状態でのゲート通過を拒む（QLT-011）。
lint:
	$(CARGO) clippy --workspace --all-targets --locked -- -D warnings

## lint-windows — Windows 版のクレートを Windows のターゲットで検査する（ADR 0002）。
## 🔴 Linux の make lint では Windows のコードは cfg で外れて見えない。Windows のコードはここでだけ検査される。
## X11 のクレート（画面とアダプタ）は Windows では組み立てない（cairo と X が無い）。合成ルートは Windows のクレートで組む（ADR 0002）。
WINDOWS_TARGET := x86_64-pc-windows-gnu
WINDOWS_EXCLUDE := --exclude nene-ledger-ui-x11 --exclude nene-ledger-adapters-x11
lint-windows:
	$(CARGO) clippy --workspace $(WINDOWS_EXCLUDE) --all-targets --locked --target $(WINDOWS_TARGET) -- -D warnings

## test-windows — Windows のターゲットの単体テストを、WSL の相互運用で Windows で直に走らせる（WSL の上でだけ・GitHub Issue #211）。
## 単体テストを Windows の OS で走らせるだけで、窓・IME・倍率の実機の確かめ（QLT-013）の代わりではない。
test-windows:
	bash tools/windows-tests.sh

## build-windows — Windows の exe を作る（mingw-w64 のリンカ）。施主の実機で確かめる exe と、配る exe（ADR 0032）。target/x86_64-pc-windows-gnu/release/nene-ledger.exe
build-windows:
	$(CARGO) build --release --locked -p nene-ledger --target $(WINDOWS_TARGET)

test:
	$(CARGO) test --workspace --locked

## conformance — このリポジトリ固有の規約検査（CNF-0xx / xtask）。
conformance:
	$(CARGO) run --quiet -p xtask --locked -- conformance

## dist — 配る zip（Windows の exe・LICENSE・著作権表示・説明書・見本）を dist/ に作る（ADR 0032）。release の workflow も同じ入口。
dist:
	bash tools/dist.sh

## msix — Microsoft Store に出す MSIX を out/msix/ に作る（ADR 0035）。WSL の相互運用で Windows SDK の makeappx.exe を呼ぶ。
## msix-check — Cargo.toml の SemVer を MSIX の 4 桁の版に写す規則を確かめる（Windows が無くても走る）。
msix:
	bash tools/msix.sh

msix-check:
	@bash tools/msix-check.sh

## notices — 配る exe に入る外部クレートの著作権表示の一覧（THIRD-PARTY-NOTICES.txt）を Cargo.lock から作り直す（ADR 0032）。
## notices-check — 一覧が Cargo.lock の依存と食い違えば落とす。依存を変えたら notices で作り直す。
notices:
	$(CARGO) run --quiet -p xtask --locked -- notices

notices-check:
	$(CARGO) run --quiet -p xtask --locked -- notices-check

## validate-git — GIT-001〜005。手元は origin/main..HEAD と管理しているファイル、CI は PR のメタデータも見る（ADR 0021）。
validate-git:
	$(CARGO) run --quiet -p xtask --locked -- validate-git

## coverage — 中核（core / application）のカバレッジ下限（QLT-009）。
## 🔴 分岐（--branch）は stable の rustc では測れない（-Z coverage-options=branch が要る。nene-ruler 2026-10-15 実測）。
##    下限は region に置く（nene-ruler ADR 0010 の継承）。下限を割ると理由を出さずに終了 1 で落ちる。
## 導入: cargo install cargo-llvm-cov --locked
## 中核のクレートが増えたら -p を足す（application は Issue #19 で足した）。
## 🔴 測る前に必ず clean する。残った作業領域（target/llvm-cov-target）が集計に混ざり、
##    判定がそれまでに何を測ったかに依る（Issue #16: 別の版で scan を測った後に 88.46% で誤って落ちた）。
COVERAGE_PACKAGES := -p nene-ledger-core -p nene-ledger-application
coverage:
	@command -v cargo-llvm-cov >/dev/null || { echo "coverage: cargo-llvm-cov が無い。cargo install cargo-llvm-cov --locked"; exit 2; }
	$(CARGO) llvm-cov clean --workspace
	$(CARGO) llvm-cov $(COVERAGE_PACKAGES) --locked --summary-only --fail-under-regions 90

## deny — 依存の許可制（ARC-013）。方針の正本は deny.toml。
## 🔴 --deny license-not-encountered を外さないこと。
## 導入: cargo install cargo-deny --locked
deny:
	@command -v cargo-deny >/dev/null || { echo "deny: cargo-deny が無い。cargo install cargo-deny --locked"; exit 2; }
	$(CARGO) deny --locked check --deny license-not-encountered

## doc-check — rustdoc の警告（壊れた intra-doc link 等）を失敗にする。
doc-check:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --workspace --no-deps --locked

build:
	$(CARGO) build --workspace --release --locked

## prove — ゲートが意図した違反で発火することを実測する（QLT-007）。結果は docs/quality/gate-proofs.md へ転記する。
prove:
	$(CARGO) run --quiet -p xtask --locked -- prove

## screenshot — Xvfb の上で窓を出し PNG を out/ に撮る（QLT-013）。
## 単体テストではなく環境検証である。結果は目で見て docs/quality/gate-proofs.md に記録する。
## 🔴 Xvfb は Unix の接続口を作らず TCP（localhost の 6000 ＋ 番号）で待つ。/tmp/.X11-unix が読み取り専用の環境
##    （WSLg）では Unix の接続口を作れずに起動しない（Issue #20 で実測）。
SCREENSHOT_DISPLAY ?= :99
## 開く見本（BOM 付きの UTF-8・引用の中の改行とタブ・欠けたセル・長い値・先頭の 0）。
SCREENSHOT_FILE ?= crates/app/fixtures/screenshot.csv
screenshot:
	@command -v Xvfb >/dev/null || { echo "screenshot: Xvfb が無い（apt-get install xvfb）"; exit 2; }
	@command -v import >/dev/null || { echo "screenshot: ImageMagick が無い（apt-get install imagemagick）"; exit 2; }
	$(CARGO) build --locked -p nene-ledger
	@mkdir -p out
	@Xvfb $(SCREENSHOT_DISPLAY) -nolisten unix -listen tcp -screen 0 1024x640x24 >/dev/null 2>&1 & echo $$! > out/xvfb.pid; sleep 1; \
	  DISPLAY=localhost$(SCREENSHOT_DISPLAY) ./target/debug/nene-ledger "$(SCREENSHOT_FILE)" & echo $$! > out/app.pid; sleep 2; \
	  DISPLAY=localhost$(SCREENSHOT_DISPLAY) import -window root out/xvfb.png; \
	  kill $$(cat out/app.pid) $$(cat out/xvfb.pid) 2>/dev/null; rm -f out/*.pid
	@echo "out/xvfb.png を Read で見る"

## bench-pollock — Pollock の全ファイルで正しく読めた率を測る（QLT-014・BIG-11・Issue #24）。
## 出力と時間を Pollock の置き場（results/nene/polluted_files）に書く。採点は Pollock の evaluate.py。
## 手順と記録は docs/quality/benchmarks.md。
POLLOCK ?=
bench-pollock:
	@if [ -z "$(POLLOCK)" ]; then echo "bench-pollock: POLLOCK=<Pollock の置き場> を指定する"; exit 2; fi
	$(CARGO) build --release --locked -p nene-ledger
	$(CARGO) run --quiet --release -p xtask --locked -- bench-pollock "$(POLLOCK)"

## fetch-corpus / bench-corpus — 実データの突き合わせ（QLT-014・BIG-11・ADR 0023・GitHub Issue #26）。
## 出どころと正解の表は docs/quality/corpus/sources.tsv。ファイルはリポジトリの外（CORPUS=<置き場>）に取得する。
## 参照の読み方は製品と独立した Python の csv（標準ライブラリだけ）。🔴 CI では走らせない（取得がネットワークに依る）。
## 判別を変える PR では bench-corpus を走らせ、変わったファイルを本文に書く。表の判定の列を書き直すなら
## python3 -I tools/corpus/compare.py "$(CORPUS)" --binary target/release/nene-ledger --update <版>。
CORPUS ?=
fetch-corpus:
	@if [ -z "$(CORPUS)" ]; then echo "fetch-corpus: CORPUS=<置き場> を指定する"; exit 2; fi
	python3 -I tools/corpus/fetch.py "$(CORPUS)"

bench-corpus:
	@if [ -z "$(CORPUS)" ]; then echo "bench-corpus: CORPUS=<置き場> を指定する"; exit 2; fi
	$(CARGO) build --release --locked -p nene-ledger
	python3 -I tools/corpus/compare.py "$(CORPUS)" --binary target/release/nene-ledger

## doctor — 開発環境（クラウドのコンテナ・WSLg）に道具が揃っているかを見る。検査ではない。
doctor:
	@rustc --version; cargo --version; cargo clippy --version; cargo fmt --version
	@command -v cargo-deny >/dev/null && cargo deny --version || echo "cargo-deny: 無い（cargo install cargo-deny --locked）"
	@command -v cargo-llvm-cov >/dev/null && cargo llvm-cov --version || echo "cargo-llvm-cov: 無い（cargo install cargo-llvm-cov --locked）"
	@pkg-config --modversion cairo >/dev/null 2>&1 && echo "cairo $$(pkg-config --modversion cairo)" || echo "cairo: 無い（apt-get install libcairo2-dev）"
	@command -v Xvfb >/dev/null && echo "Xvfb: ある" || echo "Xvfb: 無い（apt-get install xvfb）"
	@command -v import >/dev/null && echo "ImageMagick: ある" || echo "ImageMagick: 無い（apt-get install imagemagick）"
	@command -v xdotool >/dev/null && echo "xdotool: ある" || echo "xdotool: 無い（apt-get install xdotool）"
	@command -v x86_64-w64-mingw32-gcc >/dev/null && echo "mingw-w64: ある" || echo "mingw-w64: 無い（apt-get install mingw-w64。Windows の exe を組むのに要る・ADR 0002）"

clean:
	$(CARGO) clean
