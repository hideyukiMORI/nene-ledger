//! 編集の欄の中のカーソルと選択と、確定前の文字列の下線。

/// 文字の添字で表したカーソルと選択の起点。UI が字体で画素の位置に直す。
/// 確定前の文字列（IME・FR-020）の範囲には下線を引く（`underline`・始まりと、含まない終わりの文字の添字）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextCaret {
    cursor: usize,
    anchor: usize,
    underline: Option<(usize, usize)>,
}

impl TextCaret {
    /// カーソルと起点から作る。選択が無ければ同じ値。
    #[must_use]
    pub const fn new(cursor: usize, anchor: usize) -> Self {
        Self {
            cursor,
            anchor,
            underline: None,
        }
    }

    /// `start` から `end`（含まない）の字に下線を引くもの（確定前の文字列）。
    #[must_use]
    pub const fn with_underline(self, start: usize, end: usize) -> Self {
        Self {
            underline: Some((start, end)),
            ..self
        }
    }

    /// 下線を引く範囲（始まりと、含まない終わり）。
    #[must_use]
    pub const fn underline(self) -> Option<(usize, usize)> {
        self.underline
    }

    /// カーソル（文字の添字）。
    #[must_use]
    pub const fn cursor(self) -> usize {
        self.cursor
    }

    /// 選択の起点（文字の添字）。
    #[must_use]
    pub const fn anchor(self) -> usize {
        self.anchor
    }

    /// 選択の始まり（小さいほう）。
    #[must_use]
    pub const fn start(self) -> usize {
        if self.cursor < self.anchor {
            self.cursor
        } else {
            self.anchor
        }
    }

    /// 選択の終わり（大きいほう・含まない）。
    #[must_use]
    pub const fn end(self) -> usize {
        if self.cursor < self.anchor {
            self.anchor
        } else {
            self.cursor
        }
    }

    /// 選択があるか。
    #[must_use]
    pub const fn has_selection(self) -> bool {
        self.cursor != self.anchor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_and_end_are_ordered_whichever_way_the_selection_was_made() {
        let forward = TextCaret::new(5, 2);
        let backward = TextCaret::new(2, 5);
        assert_eq!((forward.start(), forward.end()), (2, 5));
        assert_eq!((backward.start(), backward.end()), (2, 5));
        assert!(forward.has_selection());
        assert!(!TextCaret::new(3, 3).has_selection());
        assert_eq!(forward.cursor(), 5);
        assert_eq!(forward.anchor(), 2);
    }

    #[test]
    fn an_underline_marks_the_composing_range() {
        let plain = TextCaret::new(3, 3);
        assert_eq!(plain.underline(), None);
        let composing = plain.with_underline(1, 3);
        assert_eq!(composing.underline(), Some((1, 3)));
        assert_ne!(composing, plain);
    }
}
