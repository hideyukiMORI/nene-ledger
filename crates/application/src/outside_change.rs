//! 開いている間に、原本が外から変わった様子（FILE-01・ADR 0039）。

/// 開いている間に、原本が外から変わった様子。窓が前面に来たときに、開く前に取った印と比べて分かる。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutsideChange {
    /// 大きさか更新日時（Unix では dev と inode も）が、開いたときと違う。
    Changed,
    /// 印を読めない（消された・動かされた・置き場が切れた）。
    Missing,
}
