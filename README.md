# NeNe Ledger

A CSV / TSV editor for Windows and X11. Rust, no UI toolkit. Fast on huge files, and it never changes your data on its own.

> **Status (2026-11-07):** **[0.1.0-preview.1](https://github.com/hideyukiMORI/nene-ledger/releases/tag/v0.1.0-preview.1)** is out — a preview for Windows, to be tried before the first release (0.1.0). Viewing (M1) and most of editing and saving (M2) work; sorting, filtering, transforms and validation (M3–M4) are not started. Feature status, honestly kept, is in [docs/FEATURES.md](docs/FEATURES.md); the rules that actually fire are `active` in [docs/QUALITY_GATES.md](docs/QUALITY_GATES.md).

## Download (Windows)

1. Download `nene-ledger-0.1.0-preview.1-windows-x86_64.zip` from the [release page](https://github.com/hideyukiMORI/nene-ledger/releases/tag/v0.1.0-preview.1) and unzip it anywhere (no installer).
2. Drop a CSV file onto `nene-ledger.exe`, or start the exe and choose a file. Samples are in `samples/`.
3. The exe is not code-signed. If Windows SmartScreen stops it, click **More info** → **Run anyway**.
4. `Ctrl+Shift+P` (or `F1`) opens the command palette, with the key list and **About** (version and license).

The zip is built by GitHub Actions from the tagged source (`make dist`, ADR 0032) and includes `LICENSE`, `THIRD-PARTY-NOTICES.txt` and a short guide in Japanese and English.

## What works in 0.1.0-preview.1

- **Huge files, at once.** A 1 GB / 12-million-row CSV shows its first screen immediately; the row index is built in the background. Delimiter, encoding (UTF-8, Shift_JIS, UTF-16 and others) and line endings are detected and can be changed from the palette.
- **Your data stays as it is.** `00123` stays `00123`; dates, long numbers and exponents are never reinterpreted. Rows you did not touch are written back byte for byte.
- **Edit and save.** Edit cells (type, F2, double-click, IME), insert and remove rows and columns, undo and redo (even after saving), save in the background with progress and `Esc` to cancel, *Save As*, and a new blank table (`Ctrl+N`).
- **Paste and cut.** Paste a multi-cell range from Excel (rows and columns are added past the edge; one `Ctrl+Z` undoes it). Cut (`Ctrl+X`) empties the source when you paste.
- **Find and replace.** `Ctrl+F` finds in the shown values (full-/half-width and case folded by default; 1 GB to the last row in about 1.6 s, in parallel). `Ctrl+H` replaces one by one, or all at once after showing the count, as one undo step. `Alt+R` switches to regular expressions; `$1` keeps the original characters.
- **Look.** Frozen columns, column reordering, a detail pane, sum and count of the selection, 125 colour themes, zoom.

Not yet: sorting, filtering, column statistics, transforms, validation, isolating broken rows, writing in another dialect, an encoding check before saving, UI languages other than Japanese, and highlighting the matches while finding ([#189](https://github.com/hideyukiMORI/nene-ledger/issues/189)).

Bugs and ideas: [GitHub Issues](https://github.com/hideyukiMORI/nene-ledger/issues).

## Where it is going

- Open a multi-gigabyte CSV and see the first screen at once; the row index is built in the background with visible progress.
- `00123` stays `00123`, `1-2` stays `1-2`. Cells are text; nothing is inferred. Rows you did not touch are written back byte for byte.
- Saving writes a temporary file and replaces the original, so a crash never leaves a half-written file.
- Sorting, filtering, statistics, pivots, lookups and column transforms run in the background and can be cancelled; the window never freezes.
- UTF-8, UTF-16 and Shift_JIS; commas, tabs, semicolons and pipes; Japanese input through the IME.
- Every widget is drawn by the app itself: Win32 + Direct2D on Windows, `x11rb` + cairo on Linux. No Electron, no GTK, no Qt, no JVM.

The requirements are in [SPECIFICATION.md](SPECIFICATION.md) (Japanese).

## Building from source

The toolchain is pinned by `rust-toolchain.toml` (it also adds the Windows target). On Linux:

```console
$ make build            # the X11 build (needs libcairo2 and fonts-noto-cjk to display Japanese)
$ make build-windows    # the Windows exe with mingw-w64 (x86_64-w64-mingw32-gcc)
$ make dist             # the zip that is published (exe, LICENSE, notices, guide, samples)
```

`make doctor` lists the tools that are missing. The checks a change must pass are described in [docs/DEVELOPMENT_WORKFLOW.md](docs/DEVELOPMENT_WORKFLOW.md).

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
