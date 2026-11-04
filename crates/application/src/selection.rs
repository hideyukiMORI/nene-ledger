//! 選択。

use crate::cell_position::CellPosition;
use crate::selection_mode::SelectionMode;

/// 選択。起点とカーソルの 2 つのセルを角とする矩形（NAV-02）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Selection {
    anchor: CellPosition,
    cursor: CellPosition,
}

impl Selection {
    /// `at` の 1 つのセルだけの選択。
    #[must_use]
    pub fn at(at: CellPosition) -> Self {
        Self {
            anchor: at,
            cursor: at,
        }
    }

    /// 起点（広げても動かない側）。
    #[must_use]
    pub fn anchor(self) -> CellPosition {
        self.anchor
    }

    /// カーソル。
    #[must_use]
    pub fn cursor(self) -> CellPosition {
        self.cursor
    }

    /// カーソルを `to` へ動かした選択。畳むなら `to` だけ、広げるなら起点から `to` まで。
    #[must_use]
    pub fn moved(self, to: CellPosition, mode: SelectionMode) -> Self {
        match mode {
            // 範囲を足すときも、今の範囲は 1 つのセルから始まる（前の範囲は画面の状態が持つ）。
            SelectionMode::Collapse | SelectionMode::Add => Self::at(to),
            SelectionMode::Extend => Self {
                anchor: self.anchor,
                cursor: to,
            },
        }
    }

    /// 矩形の左上の角。
    #[must_use]
    pub fn top_left(self) -> CellPosition {
        CellPosition::new(
            self.anchor.body().min(self.cursor.body()),
            self.anchor.column().min(self.cursor.column()),
        )
    }

    /// 矩形の右下の角。
    #[must_use]
    pub fn bottom_right(self) -> CellPosition {
        CellPosition::new(
            self.anchor.body().max(self.cursor.body()),
            self.anchor.column().max(self.cursor.column()),
        )
    }

    /// 本体の `body` 番目の行・`column` 列目のセルが矩形に入るか。
    #[must_use]
    pub fn contains(self, body: u64, column: usize) -> bool {
        let (top, bottom) = ordered(self.anchor.body(), self.cursor.body());
        let (left, right) = ordered(self.anchor.column(), self.cursor.column());
        (top..=bottom).contains(&body) && (left..=right).contains(&column)
    }
}

/// 小さいほうと大きいほう。
fn ordered<T: Ord + Copy>(first: T, second: T) -> (T, T) {
    (first.min(second), first.max(second))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapsing_selects_only_the_cursor() {
        let selection = Selection::at(CellPosition::new(3, 2));
        assert!(selection.contains(3, 2));
        assert!(!selection.contains(3, 3));
        let moved = selection.moved(CellPosition::new(5, 0), SelectionMode::Collapse);
        assert_eq!(moved.cursor(), CellPosition::new(5, 0));
        assert!(!moved.contains(3, 2));
    }

    #[test]
    fn extending_makes_a_rectangle_in_any_direction() {
        let selection = Selection::at(CellPosition::new(3, 2))
            .moved(CellPosition::new(1, 4), SelectionMode::Extend);
        assert_eq!(selection.cursor(), CellPosition::new(1, 4));
        assert_eq!(selection.top_left(), CellPosition::new(1, 2));
        assert_eq!(selection.bottom_right(), CellPosition::new(3, 4));
        for (body, column) in [(1, 2), (3, 4), (2, 3), (1, 4), (3, 2)] {
            assert!(selection.contains(body, column), "{body} {column}");
        }
        for (body, column) in [(0, 2), (4, 2), (2, 1), (2, 5)] {
            assert!(!selection.contains(body, column), "{body} {column}");
        }
    }
}
