//! 方言の判別に確信が無いときの、1 位と競う相手。

use nene_ledger_core::detection::Detection;
use nene_ledger_core::dialect::Dialect;
use nene_ledger_core::dialect_candidate::DialectCandidate;

/// 方言の判別に確信が無いときの、1 位と競う相手の読み方。開いたときに状態の行で、何と何で迷ったかを知らせ、
/// 候補から選ぶ入口を示す（FR-023・Issue #3・GitHub Issue #381）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DialectDoubt {
    top: DialectCandidate,
    rival: Option<DialectCandidate>,
}

impl DialectDoubt {
    /// 判別 `detection` に確信が無ければ、1 位と、競う候補（`Detection::rival`・引用の逃がし方だけが違う候補は除く）の
    /// 読み方。確信があれば `None`。1 位が 0 点（どの区切りでも 1 列にしか切れない・空のファイル）なら、迷う読み方が
    /// 無いので知らせない（GitHub Issue #24）。
    pub(crate) fn of(detection: &Detection) -> Option<Self> {
        let top = detection.candidates().first().copied()?;
        if detection.confident() || top.score() == 0 {
            return None;
        }
        Some(Self {
            top,
            rival: detection.rival(),
        })
    }

    /// 開いた読み方（1 位）。
    pub(crate) fn top(self) -> Dialect {
        self.top.dialect()
    }

    /// 1 位と競った読み方。
    pub(crate) fn rival(self) -> Option<Dialect> {
        self.rival.map(DialectCandidate::dialect)
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
        assert_eq!(doubt.top(), tie.candidates()[0].dialect());
        assert_eq!(doubt.rival(), Some(tie.candidates()[1].dialect()));
        assert_ne!(
            doubt.top().delimiter(),
            tie.candidates()[1].dialect().delimiter()
        );
    }

    /// 空のファイルと 1 列のファイルは、迷う読み方が無いので知らせない（GitHub Issue #24）。
    #[test]
    fn nothing_to_doubt_in_an_empty_or_single_column_file() {
        assert_eq!(DialectDoubt::of(&Detection::of(b"")), None);
        assert_eq!(
            DialectDoubt::of(&Detection::of(b"alpha\nbeta\ngamma\n")),
            None
        );
    }
}
