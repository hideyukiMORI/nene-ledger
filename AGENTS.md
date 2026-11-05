# NeNe Ledger — Agent Guide

Short English entry point. The authoritative handbook is [CLAUDE.md](CLAUDE.md) (Japanese);
the normative documents live in [docs/](docs/).

## Project identity

- Product: NeNe Ledger — a CSV / TSV editor for Windows (primary) and X11 that stays fast on huge files and never changes data on its own
- Language / UI: Rust, edition 2024 / none (Win32 + Direct2D on Windows, plain X11 through `x11rb` + cairo on Linux; every widget is drawn by hand)
- Data: the source file is read-only until save. What you see is source + overlay (edits) + column transforms + view (sort / filter). Save writes a temporary file and replaces the original; untouched rows are copied byte for byte ([ADR 0005](docs/adr/0005-source-is-read-only-until-save.md))
- Speed: `crates/scan` is the only non-Windows crate allowed `unsafe` (memory mapping, SIMD); the canonical cell-splitting rules are a plain reference implementation in core, and the fast path must match it ([ADR 0006](docs/adr/0006-confined-unsafe-for-scan.md))
- Analytics (sort, statistics, pivot, lookup) are written in-house; DuckDB, qsv, xsv and Modern CSV are benchmarks, not dependencies ([ADR 0007](docs/adr/0007-own-engine-and-external-benchmarks.md))
- Repository: public on GitHub. Issues are GitHub issues, a pull request is the integration boundary, CI only runs `make ci`, and `main` is protected by a ruleset ([ADR 0021](docs/adr/0021-public-repository-on-github.md))
- Governing principle: **one meaning, one canonical implementation path, enforced by machines**

## Status

From 2026-10-22 to 2026-11-02 the project advanced through 68 local issues (no remote): the specification, the normative documents, the machinery (`[workspace.lints]`, `clippy.toml`, `deny.toml`, `xtask`, `Makefile`, hooks), `docs/FEATURES.md` and `docs/ROADMAP.md`, and most of M1 in eight crates (core / scan / application / ui-x11 / adapters-x11 / ui-win32 / adapters-windows / app). On 2026-11-04 the code and the normative documents moved to this public repository without that history (ADR 0021). `Issue #N` in the retained records (gate-proofs, benchmarks, FEATURES) refers to the pre-publication numbering, not to GitHub. The license is a custom one (ADR 0009): free use, credit kept in modified versions, free redistribution only, no sale.

## Required reading before changing production code

0. `SPECIFICATION.md` and `docs/FEATURES.md` (the 221 features, their phase and status; mark `確認済み` only with a named test or measurement)
1. `docs/ARCHITECTURE_CONSTITUTION.md`
2. `docs/PROJECT_LAYOUT.md`
3. `docs/CODING_RULES.md`
4. `docs/QUALITY_GATES.md`
5. `docs/DEVELOPMENT_WORKFLOW.md`
6. `docs/COMMIT_CONVENTIONS.md`
7. `docs/GLOSSARY.md`

Then the active issue file and the relevant accepted ADRs.

## Agent rules

- Do not invent a second implementation path because it is locally convenient.
- Do not weaken a gate to make a change pass. Fix the code instead.
- Do not mark a rule `active` in `docs/QUALITY_GATES.md` before its enforcement exists and `docs/quality/gate-proofs.md` has its row.
- Do not write to the source file outside the save-and-replace path. Never infer cell types.
- Do not read the current time, environment, files or threads outside `crates/adapters-x11`, `crates/adapters-windows`, `crates/scan` and `crates/store`. Core crates are `#![no_std]`.
- Do not put sorting, aggregation or state transitions in the UI.
- Do not write `_` in a `match` over a closed enum.
- Do not add dependencies without an ADR and a measured `deny.toml` update. There are none yet.
- Do not claim "fast" without a measurement against the same file (QLT-014), and do not claim "works on Windows" before it has run on Windows (QLT-013).
- Do not push to `main` directly; every change goes through a pull request and a squash merge.
- Read `CLAUDE.local.md` (not tracked) for machine-specific constraints when it exists. Tracked files, commits and PR bodies hold only what is needed outside this machine; never copy machine-specific details into them (GIT-005).
- Do not claim a command passed unless it was actually executed.

## Required completion report

Issue and rule IDs, files and behavior changed, verification commands and results (and where the record lives),
documentation or schema changes, `#[expect]` suppressions added (or `none`), remaining risks.

Investigation-only requests do not authorize editing or committing.
