//! 書き換えた後のセル（ADR 0024 の層 2）。

use alloc::string::String;

/// 書き換えた後のセル。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CellValue {
    /// 利用者が入れた値（引用を外した文字列。改行を含んでよい）。空の文字列は「空のセル」。
    Text(String),
    /// 「セルが無い」（KEEP-05）。
    Absent,
}
