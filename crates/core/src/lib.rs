//! `NeNe Ledger` の中核。意味の正本を持つ。
//!
//! 🔴 `#![no_std]`（ARC-003・CNF-007）。ファイル・時刻・環境・スレッドに構文的に到達できない。
//! 切り出しの規則（区切り・引用・改行）の正本はここの参照実装であり、速い経路（scan）は
//! これと同じ結果を出すことをテストで固定する（ADR 0006）。
#![no_std]

extern crate alloc;

pub mod appearance;
pub mod binary_sniff;
pub mod boundary;
pub mod byte_range;
pub mod caret_move;
pub mod cell_change;
pub mod cell_edits;
pub mod cell_kind;
pub mod cell_value;
pub mod cell_writing;
pub mod chrome_button;
pub mod chrome_hit;
pub mod color;
pub mod column_id;
pub mod column_order;
pub mod column_widths;
pub mod contrast;
pub mod csv_writing;
pub mod decimal;
pub mod delimiter;
pub mod detecting;
pub mod detection;
pub mod dialect;
pub mod dialect_candidate;
pub mod dialect_failure;
pub mod display_scale;
pub mod display_width;
pub mod draw_command;
mod edge_band;
mod encoding_detecting;
pub mod encoding_guess;
pub mod field;
pub mod field_scan;
pub mod frame;
pub mod glyph_metrics;
pub mod header_guess;
pub mod line_ending;
pub mod line_ending_counts;
pub mod number_reading;
pub mod order_run;
pub mod palette;
pub mod palette_role;
pub mod pasted_text;
pub mod pixel_point;
pub mod pixel_rect;
pub mod pixel_size;
pub mod pointer_point;
pub mod preedit;
pub mod quote_escape;
pub mod quoting;
pub mod record;
pub mod records;
pub mod resize_edge;
pub mod row_id;
pub mod row_order;
pub mod row_piece;
pub mod row_shape;
pub mod row_write_failure;
pub mod row_writing;
pub mod save_plan;
pub mod save_step;
pub mod scanning;
pub mod scroll_axis;
pub mod table_layout;
pub mod text_caret;
pub mod text_edit;
pub mod text_encoding;
pub mod text_format;
pub mod text_role;
pub mod text_units;
pub mod theme_catalog;
pub mod theme_choice;
pub mod theme_derivation;
pub mod theme_seed;
pub mod unclosed_quote;
pub mod value;
pub mod window_chrome;
pub mod window_placement;
