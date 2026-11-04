//! 窓の縁の帯（見えない縁の当たり判定に使う）。

/// 長さ `length` の区間の、始まりか終わりから `reach` 以内の帯。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Band {
    /// 始まりの側（左か上）。
    Start,
    /// 終わりの側（右か下）。
    End,
}

/// `position` が始まりか終わりの帯に入れば、その帯。窓が狭くて両方に入るときは始まり。
pub(crate) fn band(position: u32, length: u32, reach: u32) -> Option<Band> {
    if position < reach {
        Some(Band::Start)
    } else if position.saturating_add(reach) >= length {
        Some(Band::End)
    } else {
        None
    }
}
