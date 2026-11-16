//! 原本が外から変わったかと、書き換えを捨てて読み直す問いの状態（ADR 0039）。

use crate::outside_change::OutsideChange;

/// 原本が外から変わったかと、書き換えを捨てて読み直す問いの状態。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutsideWatch {
    change: Option<OutsideChange>,
    armed: bool,
}

impl OutsideWatch {
    /// 開いたときのまま。
    #[must_use]
    pub const fn unchanged() -> Self {
        Self {
            change: None,
            armed: false,
        }
    }

    /// 外から変わっていれば、その様子。
    #[must_use]
    pub const fn change(&self) -> Option<OutsideChange> {
        self.change
    }

    /// 比べた結果 `change` を覚える。様子が変われば、読み直しの問いはやめる。
    pub fn set(&mut self, change: Option<OutsideChange>) {
        if self.change != change {
            self.armed = false;
        }
        self.change = change;
    }

    /// 読み直しのほかの操作が来たので、読み直しの問いをやめる。
    pub const fn disarm(&mut self) {
        self.armed = false;
    }

    /// 読み直しを求められた。読み直してよければ `true`。書き換えがあれば（`edited`）一度尋ねて `false` を返し、続けて
    /// もう一度求められたら `true`。
    pub const fn reload_requested(&mut self, edited: bool) -> bool {
        let allowed = !edited || self.armed;
        self.armed = !allowed;
        allowed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_edited_table_reloads_only_on_the_second_request() {
        let mut watch = OutsideWatch::unchanged();
        assert!(watch.reload_requested(false));
        assert!(!watch.reload_requested(true));
        assert!(watch.reload_requested(true));
        assert!(!watch.reload_requested(true));
        watch.disarm();
        assert!(!watch.reload_requested(true));
        // 様子が変われば尋ね直す。同じ様子なら問いは続く。
        watch.set(Some(OutsideChange::Changed));
        assert_eq!(watch.change(), Some(OutsideChange::Changed));
        assert!(!watch.reload_requested(true));
        watch.set(Some(OutsideChange::Changed));
        assert!(watch.reload_requested(true));
    }
}
