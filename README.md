# NeNe Ledger

A CSV / TSV editor for Windows and X11. Rust, no UI toolkit. Fast on huge files, and it never changes your data on its own.

> **Status (2026-11-04):** M1 (open, look, select, copy, sum) is mostly implemented in eight crates and runs on X11 and on Windows; editing and saving (M2) are not started. This public repository starts from the current code: the 68 local issues that led here are not part of its history (ADR 0021). The rules that actually fire are `active` in `docs/QUALITY_GATES.md`.

## What it will be

- Open a multi-gigabyte CSV and see the first screen at once; the row index is built in the background with visible progress.
- `00123` stays `00123`, `1-2` stays `1-2`. Cells are text; nothing is inferred. Rows you did not touch are written back byte for byte.
- Saving writes a temporary file and replaces the original, so a crash never leaves a half-written file.
- Sorting, filtering, statistics, pivots, lookups and column transforms run in the background and can be cancelled; the window never freezes.
- UTF-8, UTF-16 and Shift_JIS; commas, tabs, semicolons and pipes; Japanese input through the IME.
- Every widget is drawn by the app itself: Win32 + Direct2D on Windows, `x11rb` + cairo on Linux. No Electron, no GTK, no Qt, no JVM.

The requirements are in [SPECIFICATION.md](SPECIFICATION.md) (Japanese).

## How it is built

This repository follows the AYANE strict policy: **one meaning, one canonical implementation path, enforced by machines.**
Its documents are derived from NeNeDocket.

- `docs/ARCHITECTURE_CONSTITUTION.md` — the rules (ARC-NNN)
- `docs/CODING_RULES.md` — the approved subset of Rust (RS-NNN)
- `docs/QUALITY_GATES.md` — what is enforced right now (active / planned / impossible), the only source of truth
- `docs/adr/` — decisions
- GitHub Issues are the units of work and a pull request is the integration boundary (ADR 0021)

## License

NeNe Ledger License 1.0 (`LicenseRef-NeNe-Ledger-1.0`): free to use for any purpose including commercial use; modified versions must keep the credit; redistribution only free of charge; no sale. See [LICENSE](LICENSE). This is not an OSI open-source license.
