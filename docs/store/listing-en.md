# Store listing (English) — draft (ADR 0035 / GitHub Issue #342)

> Copy into Partner Center → Store listings → English. 🔴 While it is a preview, say so at the top (ADR 0032, decision 1).

## Product name

NeNe Ledger

## Short description

A table editor that opens huge CSV / TSV files instantly and saves your edits without touching a single byte of the rows you did not change. Leading zeros, dates and long numbers stay exactly as written.

## Description

[Preview / early access] This is a preview released before the first stable version. Some features are still missing — keep a copy of important files while you try it.

NeNe Ledger is a table editor for CSV and TSV files. Spreadsheet apps silently drop leading zeros and turn dates and long numbers into something else. NeNe Ledger keeps every value as text and never changes it — on screen or on save. Rows you did not edit are written back byte for byte.

• Opens huge files instantly — the first screen of a 1 GB, 10-million-row file appears right away; counting and indexing continue in the background.
• Detects how the file is written — delimiter, encoding (UTF-8, Shift_JIS, UTF-16, EUC-JP and more), quoting and header rows — and lets you try another reading on the spot.
• Edit and save — edit cells (including Japanese input), add and remove rows and columns, undo and redo (even after saving), save as. Paste multi-cell ranges from Excel.
• Find and replace — case, full-/half-width and kana options, regular expressions. Before replacing everything it shows what will change into what, and warns when only part of a value changes. One Ctrl+Z undoes it all.
• Spot broken rows — rows whose column count differs from the header are marked, and you can jump to the next or previous one.
• Keyboard first — Ctrl+Shift+P finds every command by name. 125 color themes, light/dark and high contrast, text size.
• No network — no account, no ads, nothing about your files leaves your PC.

License: NeNe Ledger License 1.0 (free to use, credit required for modified versions, free distribution only, no sale, no warranty). The user interface is currently Japanese only.

## Keywords (up to 7)

CSV, TSV, table, editor, Shift_JIS, encoding, large files

## Category

Productivity
