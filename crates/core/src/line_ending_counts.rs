//! 改行の内訳。

use crate::line_ending::LineEnding;

/// レコードの改行の内訳。混在を見つけて示す（OPEN-08）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineEndingCounts {
    crlf: usize,
    lf: usize,
    cr: usize,
}

impl LineEndingCounts {
    /// 改行の列から数える（入力の終わりは数えない）。
    pub fn tally<I: Iterator<Item = LineEnding>>(endings: I) -> Self {
        endings.fold(
            Self {
                crlf: 0,
                lf: 0,
                cr: 0,
            },
            |mut counts, ending| {
                match ending {
                    LineEnding::CrLf => counts.crlf = counts.crlf.saturating_add(1),
                    LineEnding::Lf => counts.lf = counts.lf.saturating_add(1),
                    LineEnding::Cr => counts.cr = counts.cr.saturating_add(1),
                    LineEnding::EndOfInput => {}
                }
                counts
            },
        )
    }

    /// `\r\n` の数。
    #[must_use]
    pub fn crlf(self) -> usize {
        self.crlf
    }

    /// `\n` の数。
    #[must_use]
    pub fn lf(self) -> usize {
        self.lf
    }

    /// `\r` だけの数。
    #[must_use]
    pub fn cr(self) -> usize {
        self.cr
    }

    /// 2 種類以上の改行が混ざっているか。
    #[must_use]
    pub fn mixed(self) -> bool {
        [self.crlf, self.lf, self.cr]
            .iter()
            .filter(|&&n| n > 0)
            .count()
            > 1
    }

    /// いちばん多い改行。改行が 1 つも無ければ `None`。同数なら CRLF → LF → CR の順に選ぶ。
    #[must_use]
    pub fn dominant(self) -> Option<LineEnding> {
        [
            (self.crlf, LineEnding::CrLf),
            (self.lf, LineEnding::Lf),
            (self.cr, LineEnding::Cr),
        ]
        .into_iter()
        .filter(|&(count, _)| count > 0)
        .fold(
            None,
            |best: Option<(usize, LineEnding)>, (count, ending)| match best {
                Some((top, _)) if top >= count => best,
                Some(_) | None => Some((count, ending)),
            },
        )
        .map(|(_, ending)| ending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_ignore_end_of_input_and_detect_mixtures() {
        let counts = LineEndingCounts::tally(
            [
                LineEnding::Lf,
                LineEnding::Lf,
                LineEnding::Cr,
                LineEnding::EndOfInput,
            ]
            .into_iter(),
        );
        assert_eq!((counts.crlf(), counts.lf(), counts.cr()), (0, 2, 1));
        assert!(counts.mixed());
        assert_eq!(counts.dominant(), Some(LineEnding::Lf));
    }

    #[test]
    fn no_line_endings() {
        let counts = LineEndingCounts::tally([LineEnding::EndOfInput].into_iter());
        assert!(!counts.mixed());
        assert_eq!(counts.dominant(), None);
    }

    #[test]
    fn ties_prefer_crlf_then_lf() {
        let counts = LineEndingCounts::tally([LineEnding::Cr, LineEnding::Lf].into_iter());
        assert_eq!(counts.dominant(), Some(LineEnding::Lf));
    }
}
