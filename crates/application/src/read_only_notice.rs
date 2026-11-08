//! 読み取り専用の知らせ（EDIT-13・GitHub Issue #238）。

/// 読み取り専用の知らせ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadOnlyNotice {
    /// 読み取り専用にした。
    On,
    /// 読み取り専用を外した。
    Off,
    /// 読み取り専用なので、表を変える操作をしなかった。
    Refused,
}
