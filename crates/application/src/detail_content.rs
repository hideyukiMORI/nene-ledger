//! 詳細ペインに出す中身。

use crate::cut_record::CutRecord;

/// 詳細ペインに出す、カーソルのセルの中身。空のセルとセルが無いを区別する（RS-004・KEEP-05）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailContent<'row> {
    /// カーソルの行が見えている範囲の外（読んでいない）。
    OutOfView,
    /// カーソルの行にその列のセルが無い。
    Missing,
    /// セルの値（復号したまま。空のこともある）。
    Value(&'row str),
    /// 上限で切り詰めた行のセルの値（先頭の部分だけ・ADR 0018 の決定 2・GitHub Issue #36）。
    Cut {
        /// 読んだ部分の値。
        value: &'row str,
        /// 切り詰めた行。
        cut: CutRecord,
    },
}
