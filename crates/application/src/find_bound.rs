//! 裏の検索の向きと境（ADR 0030 の決定 1・4）。

/// 探す向きと、境のセル（レコードの番号・表示の列）。境のセルそのものは照らさない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindBound {
    /// 境より後ろを、前から順に探し、最初に当たった所で止まる（無ければ範囲の頭から）。
    After(Option<(u64, usize)>),
    /// 境より前を探し、最後に当たった所を残す（無ければ範囲の終わりまで）。
    Before(Option<(u64, usize)>),
}
