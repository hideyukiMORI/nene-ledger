//! 保存先の、書く前に分かっている様子（ADR 0025 の決定 2・ADR 0028 の決定 5）。

use crate::file_stamp::FileStamp;

/// 保存先の、書く前に分かっている様子。store はこれと今の様子が違えば書かない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveTarget {
    /// 既にあるファイルを置き換える。印は開く前（名前を付けて保存では、上書きを確かめたとき）に取ったもの。
    Existing(FileStamp),
    /// まだ無いファイルを作る（名前を付けて保存）。書く前と名前を付ける直前に、無いことを確かめる。
    Absent,
}
