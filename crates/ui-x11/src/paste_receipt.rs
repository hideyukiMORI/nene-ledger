//! 貼り付けの答えを受け取った結果（CLIP-01）。

/// 貼り付けの答えを受け取った結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PasteReceipt {
    /// 受け取り終えた文字列（断られた・大きすぎたときは空）。
    Done(String),
    /// 大きな中身を分けて受け取っている途中（`INCR`・ICCCM）。
    Pending,
}
