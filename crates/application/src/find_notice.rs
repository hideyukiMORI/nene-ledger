//! 検索の知らせ（ADR 0030 の決定 1）。

/// 検索の知らせ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindNotice {
    /// 表の終わり（前へ探すときは頭）まで行って、反対の端から続けて見つけた。
    Wrapped,
    /// 見つからなかった（表を一周した）。
    NotFound,
    /// 探すのをやめた（Esc）。
    Stopped,
    /// 原本を読めなかった。
    Failed,
    /// F3 で続けて探すものが無い（まだ探していない）。
    NothingToRepeat,
}
