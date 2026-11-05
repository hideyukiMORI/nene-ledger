//! 保存の 1 歩（ADR 0025 の決定 1）。

use alloc::string::String;

/// 保存の 1 歩。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveStep {
    /// 原本の `offset` から `length` バイトを、そのまま写す。
    Copy {
        /// 原本の中の始まり。
        offset: u64,
        /// バイト数（0 にならない）。
        length: u64,
    },
    /// 文字列を、原本の文字コードで符号化して書く（空にならない）。
    Text(String),
}
