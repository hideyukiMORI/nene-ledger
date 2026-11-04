//! 選んだ範囲の並び（今の範囲と、Ctrl で前に足した範囲）。

use alloc::vec;
use alloc::vec::Vec;
use core::ops::RangeInclusive;

use crate::copy_shape::CopyShape;
use crate::row_band::RowBand;
use crate::selection::Selection;

/// 選んだ範囲の並び。今の範囲 `active`（カーソルがある）と、Ctrl で前に足した範囲 `others`。
/// 同じセルが何度選ばれていても、数えるのは 1 回だけ（施主 2026-10-27）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ranges<'ranges> {
    active: Selection,
    others: &'ranges [Selection],
}

impl<'ranges> Ranges<'ranges> {
    /// 今の範囲 `active` と前に足した範囲 `others`。
    #[must_use]
    pub fn new(active: Selection, others: &'ranges [Selection]) -> Self {
        Self { active, others }
    }

    /// 今の範囲。
    #[must_use]
    pub fn active(self) -> Selection {
        self.active
    }

    /// すべての範囲（足した順で、今の範囲が最後）。
    pub fn all(self) -> impl Iterator<Item = Selection> + 'ranges {
        self.others
            .iter()
            .copied()
            .chain(core::iter::once(self.active))
    }

    /// 範囲が 2 つ以上あるか。
    #[must_use]
    pub fn is_multiple(self) -> bool {
        !self.others.is_empty()
    }

    /// 本体の `body` 番目の行・`column` 列目のセルを、どれかの範囲が含むか。
    #[must_use]
    pub fn contains(self, body: u64, column: usize) -> bool {
        self.all().any(|range| range.contains(body, column))
    }

    /// どれかの範囲が本体の `body` 番目の行を覆うか。
    #[must_use]
    pub fn covers_row(self, body: u64) -> bool {
        self.all().any(|range| rows_of(range).contains(&body))
    }

    /// どれかの範囲が `column` 列目を覆うか。
    #[must_use]
    pub fn covers_column(self, column: usize) -> bool {
        self.all().any(|range| columns_of(range).contains(&column))
    }

    /// すべての範囲を囲む本体の行（最初と最後）。
    #[must_use]
    pub fn row_span(self) -> (u64, u64) {
        self.all().fold((u64::MAX, 0), |(top, bottom), range| {
            (
                top.min(range.top_left().body()),
                bottom.max(range.bottom_right().body()),
            )
        })
    }

    /// 行で切った帯（上から）。帯の中はどの行も同じ列で、重なりは 1 回だけ。範囲の無い行は帯にしない。
    #[must_use]
    pub fn bands(self) -> Vec<RowBand> {
        let mut edges: Vec<u64> = self
            .all()
            .flat_map(|range| {
                [
                    range.top_left().body(),
                    range.bottom_right().body().saturating_add(1),
                ]
            })
            .collect();
        edges.sort_unstable();
        edges.dedup();
        edges
            .windows(2)
            .filter_map(|pair| {
                let (&start, &end) = (pair.first()?, pair.get(1)?);
                let columns = merged(
                    self.all()
                        .filter(|range| rows_of(*range).contains(&start))
                        .map(columns_of)
                        .collect(),
                    next_column,
                );
                (!columns.is_empty()).then(|| RowBand::new(start..=end.saturating_sub(1), columns))
            })
            .collect()
    }

    /// コピーの形。範囲が 1 つならそのまま。どの範囲も行が同じなら列を左から詰め、列が同じなら行を上から詰める
    /// （Excel と同じ・施主 2026-10-27）。揃っていなければ `None`（コピーしない）。
    #[must_use]
    pub fn copy_shape(self) -> Option<CopyShape> {
        let first = self.active;
        if self.all().all(|range| rows_of(range) == rows_of(first)) {
            return Some(CopyShape::new(
                vec![rows_of(first)],
                merged(self.all().map(columns_of).collect(), next_column),
            ));
        }
        if self
            .all()
            .all(|range| columns_of(range) == columns_of(first))
        {
            return Some(CopyShape::new(
                merged(self.all().map(rows_of).collect(), next_row),
                vec![columns_of(first)],
            ));
        }
        None
    }
}

/// 範囲の本体の行。
fn rows_of(range: Selection) -> RangeInclusive<u64> {
    range.top_left().body()..=range.bottom_right().body()
}

/// 範囲の列。
fn columns_of(range: Selection) -> RangeInclusive<usize> {
    range.top_left().column()..=range.bottom_right().column()
}

/// 範囲の並びを、小さい順に並べて重なりと隣り合いをまとめる（`next` は次の値）。
fn merged<T: Ord + Copy>(
    mut spans: Vec<RangeInclusive<T>>,
    next: fn(T) -> T,
) -> Vec<RangeInclusive<T>> {
    spans.sort_unstable_by_key(|span| *span.start());
    let mut out: Vec<RangeInclusive<T>> = Vec::new();
    for span in spans {
        match out.last_mut() {
            Some(last) if *span.start() <= next(*last.end()) => {
                if span.end() > last.end() {
                    *last = *last.start()..=*span.end();
                }
            }
            Some(_) | None => out.push(span),
        }
    }
    out
}

/// 次の行（あふれない）。
fn next_row(row: u64) -> u64 {
    row.saturating_add(1)
}

/// 次の列（あふれない）。
fn next_column(column: usize) -> usize {
    column.saturating_add(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell_position::CellPosition;
    use crate::selection_mode::SelectionMode;

    fn range(top: u64, left: usize, bottom: u64, right: usize) -> Selection {
        Selection::at(CellPosition::new(top, left))
            .moved(CellPosition::new(bottom, right), SelectionMode::Extend)
    }

    #[test]
    fn overlapping_ranges_become_bands_counted_once() {
        // 0〜3 行 × 0〜1 列 と 2〜5 行 × 1〜2 列。
        let others = [range(0, 0, 3, 1)];
        let ranges = Ranges::new(range(2, 1, 5, 2), &others);
        assert!(ranges.is_multiple() && ranges.contains(4, 2) && !ranges.contains(4, 0));
        assert_eq!(
            ranges.bands(),
            vec![
                RowBand::new(0..=1, vec![RangeInclusive::new(0, 1)]),
                RowBand::new(2..=3, vec![RangeInclusive::new(0, 2)]),
                RowBand::new(4..=5, vec![RangeInclusive::new(1, 2)]),
            ]
        );
        assert_eq!(ranges.row_span(), (0, 5));
        assert!(ranges.covers_row(5) && !ranges.covers_row(6) && ranges.covers_column(2));
    }

    #[test]
    fn gaps_between_ranges_are_not_bands() {
        let others = [range(0, 0, 0, 0)];
        let ranges = Ranges::new(range(10, 0, 10, 0), &others);
        assert_eq!(
            ranges.bands(),
            vec![
                RowBand::new(0..=0, vec![RangeInclusive::new(0, 0)]),
                RowBand::new(10..=10, vec![RangeInclusive::new(0, 0)])
            ]
        );
    }

    #[test]
    fn copying_needs_the_same_rows_or_the_same_columns() {
        let single = Ranges::new(range(1, 1, 2, 2), &[]);
        assert_eq!(
            single.copy_shape(),
            Some(CopyShape::new(
                vec![RangeInclusive::new(1, 2)],
                vec![RangeInclusive::new(1, 2)]
            ))
        );
        // 同じ行なら列を左から詰める（重なりは 1 回）。
        let beside = [range(1, 4, 2, 5)];
        assert_eq!(
            Ranges::new(range(1, 0, 2, 1), &beside).copy_shape(),
            Some(CopyShape::new(
                vec![RangeInclusive::new(1, 2)],
                vec![0..=1, 4..=5]
            ))
        );
        // 同じ列なら行を上から詰める。
        let below = [range(8, 0, 9, 1)];
        let shape = Ranges::new(range(1, 0, 2, 1), &below).copy_shape().unwrap();
        assert_eq!(shape.rows(), &[1..=2, 8..=9]);
        assert_eq!((shape.row_count(), shape.column_count()), (4, 2));
        assert!(shape.has_row(8) && !shape.has_row(5));
        // 揃っていなければコピーしない。
        let apart = [range(5, 3, 6, 3)];
        assert_eq!(Ranges::new(range(1, 0, 2, 1), &apart).copy_shape(), None);
    }
}
