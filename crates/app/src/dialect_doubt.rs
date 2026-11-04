//! 方言の判別に確信が無いときの、上位 2 つの点数。

use nene_ledger_core::detection::Detection;

/// 方言の判別に確信が無いときの、1 位と 2 位の点数（千分率）。開いたときに状態の行で知らせ、候補から選ぶ入口を示す（FR-023・Issue #3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DialectDoubt {
    top: u32,
    second: u32,
}

impl DialectDoubt {
    /// 判別 `detection` に確信が無ければ、その上位 2 つの点数。確信があれば `None`。候補が足りなければ 0 点。
    pub(crate) fn of(detection: &Detection) -> Option<Self> {
        if detection.confident() {
            return None;
        }
        let score = |rank: usize| {
            detection
                .candidates()
                .get(rank)
                .map_or(0, |candidate| candidate.score())
        };
        Some(Self {
            top: score(0),
            second: score(1),
        })
    }

    /// 1 位の点数（千分率）。
    pub(crate) fn top(self) -> u32 {
        self.top
    }

    /// 2 位の点数（千分率）。
    pub(crate) fn second(self) -> u32 {
        self.second
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_clear_table_raises_no_doubt_and_a_tie_does() {
        let clear = Detection::of(b"id,name,amount\n1,a,10\n2,b,20\n3,c,30\n4,d,40\n");
        assert_eq!(DialectDoubt::of(&clear), None);
        // カンマでもセミコロンでも同じ形に切れる。1 位と 2 位が並び、確信が無い。
        let tie = Detection::of(b"a,b;c\nd,e;f\ng,h;i\nj,k;l\n");
        let doubt = DialectDoubt::of(&tie).unwrap();
        assert!(!tie.confident());
        assert_eq!(doubt.top(), tie.candidates()[0].score());
        assert_eq!(doubt.second(), tie.candidates()[1].score());
        assert!(doubt.top().saturating_sub(doubt.second()) < 100);
    }
}
