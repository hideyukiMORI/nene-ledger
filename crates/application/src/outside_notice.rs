//! 外からの変更と読み直しの知らせ（ADR 0039）。

use crate::outside_change::OutsideChange;

/// 外からの変更と読み直しの知らせ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutsideNotice {
    /// 原本が外から変わったので、読み直すまで表を動かさない。`edited` は書き換えが残っているか。
    Stale {
        /// 変わった様子。
        change: OutsideChange,
        /// 書き換えが残っているか。
        edited: bool,
    },
    /// 書き換えがあるので、読み直しをもう一度求められるまで待つ（捨てることになる）。
    DiscardToReload,
    /// 同じ読み方のまま読み直した。
    Reloaded,
    /// 読み直せなかった（開けない）。表は前のまま。
    ReloadFailed,
    /// 新しい表には、読み直すファイルが無い。
    NothingToReload,
}
