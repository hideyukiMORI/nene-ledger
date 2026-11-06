//! 置換の知らせ（ADR 0030 の決定 6）。

/// 置換の知らせ。数はセルの数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplaceNotice {
    /// 全部の置換で、置き換えたセルの数（`skipped` は切り詰めたので飛ばしたレコードの数）。
    Replaced {
        /// 置き換えたセルの数。
        cells: usize,
        /// 飛ばしたレコードの数。
        skipped: u64,
    },
    /// 置き換えるセルが無かった（`skipped` は切り詰めたので飛ばしたレコードの数）。
    NothingToReplace {
        /// 飛ばしたレコードの数。
        skipped: u64,
    },
    /// 100 万セルを越えるので置き換えなかった。
    TooMany,
    /// 数えている間に表が変わったので置き換えなかった。
    Changed,
    /// 数えるのをやめた（Esc）か、件数を見て置き換えなかった。
    Stopped,
    /// 原本を読めなかった。
    Failed,
}
