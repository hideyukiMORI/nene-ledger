//! 裏の検索の向きと境（ADR 0030 の決定 1・4）。

/// 探す向きと、境のセル（レコードの番号・表示の列）。境のセルそのものは照らさない。
/// 🔴 前へ探す（Shift+F3）向きは、画面につなぐ GitHub Issue #152 で足す。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FindBound {
    /// 境より後ろを、前から順に探す（無ければ範囲の頭から）。
    After(Option<(u64, usize)>),
}
