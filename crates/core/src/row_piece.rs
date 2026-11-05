//! 書き換えた行の 1 つの部分（ADR 0024 の決定 4）。

use alloc::string::String;

use crate::byte_range::ByteRange;

/// 書き換えた行の 1 つの部分。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowPiece {
    /// レコードのバイト列の中の区間を、そのまま写す（触っていないセル・区切り・終わり方）。
    Source(ByteRange),
    /// 書き換えたセル（`cell_writing::written_cell` で書いた文字列。符号化の前）。
    Text(String),
}
