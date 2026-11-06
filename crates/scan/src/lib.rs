//! `NeNe Ledger` の原本の読み取り。塊の読み出し・まばらな索引・裏の処理。
//!
//! 🔴 切り出しの規則の正本は core の参照実装（ADR 0006）。ここの状態機械（`record_starts`）はその速い経路で、
//! レコードの始まりは全部、全体を一度に切った参照実装と同じでなければならない（QLT-008）。
//! 塊の境目では状態だけを持ち越し、記憶はファイルやレコードの大きさに比例しない（BIG-04）。
//! 原本は読むだけで、書き込みの API を使わない（ADR 0005・CNF-015）。

pub mod block_size;
mod char_phase;
pub mod decoded_text;
pub mod decoding;
pub mod index_builder;
pub mod index_job;
pub mod index_outcome;
pub mod indexed_source;
pub mod indexing;
pub mod job_control;
pub mod mark;
pub mod progress;
pub mod range_folding;
pub mod range_job;
pub mod range_outcome;
pub mod record_fold;
pub mod record_length;
pub mod record_range;
mod record_span;
mod record_starts;
pub mod record_window;
pub mod scan_failure;
mod scan_state;
pub mod source;
mod source_origin;
pub mod sparse_index;
pub mod split_fold;
pub mod split_job;
pub mod window_record;
pub mod windowing;
