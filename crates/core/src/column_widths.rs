//! 列の幅（半角の字数）。

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::display_width::display_columns;

/// 列の幅の下限（半角の字数）。
pub const MIN_COLUMN_COLUMNS: u32 = 3;
/// 列の幅の上限（半角の字数）。長い値は切り取って見せる（全体は詳細ペイン・VIEW-07）。
pub const MAX_COLUMN_COLUMNS: u32 = 40;
/// 利用者がドラッグで変える列の幅の上限（半角の字数・Issue #64）。下限は `MIN_COLUMN_COLUMNS`。
pub const MAX_RESIZED_COLUMNS: u32 = 200;

/// 列ごとの幅（半角の字数）。開いたときの見出しと先頭の行から決めて、スクロールでは変えない（VIEW-02）。
/// 後から列の多い行が来たら、その列だけを足す。窓の行が持たない列（見えている列の範囲の外・ADR 0033 の決定 1）は、
/// 仮の幅で足して「まだ決めていない」と覚え、初めて窓に入ったときに決める（その後は変えない・決定 3）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ColumnWidths {
    columns: Vec<u32>,
    /// 列ごとに、まだ幅を決めていないか（`columns` と同じ長さ）。
    unsettled: Vec<bool>,
}

impl ColumnWidths {
    /// `rows` の中身に合わせ、各列を `MIN_COLUMN_COLUMNS`〜`MAX_COLUMN_COLUMNS` 字に収める。
    #[must_use]
    pub fn fit(rows: &[&[String]]) -> Self {
        let mut widths = Self {
            columns: Vec::new(),
            unsettled: Vec::new(),
        };
        widths.widen(rows);
        widths
    }

    /// `rows` に、今より多い列があれば、その列の幅だけを `rows` から決めて足す。今ある列の幅は変えない。
    pub fn widen(&mut self, rows: &[&[String]]) {
        self.widen_with(
            rows,
            |row| row.len(),
            |row, column| row.get(column).map(String::as_str),
        );
    }

    /// 行 `rows` の列の数を `len`、セルを `cell`（行・列 → 値）で引く（セルを `String` の並びで持たない行からも決められる・
    /// GitHub Issue #22）。今より多い列の幅だけを、その列のいちばん広い値から決めて足す。行は全部の列を持つとみなす。
    pub fn widen_with<R, L, C>(&mut self, rows: &[R], len: L, cell: C)
    where
        L: Fn(&R) -> usize,
        C: for<'row> Fn(&'row R, usize) -> Option<&'row str>,
    {
        let all = 0..usize::MAX;
        self.widen_held(
            rows,
            (len, cell),
            (core::slice::from_ref(&all), |_| MIN_COLUMN_COLUMNS),
        );
    }

    /// [`Self::widen_with`] の正本。行 `rows` が持つのは列の範囲 `held`（左から・重ならない）だけ（ADR 0033 の決定 1）。
    /// 今より多い列は、`held` に入るならその列のいちばん広い値から決め、入らないなら仮の幅 `provisional`（列 → 字数・
    /// 見出しの名前の幅など）で足して「まだ決めていない」と覚える。まだ決めていない列が `held` に入れば、そこで決める。
    /// 決めた列の幅は変えない（VIEW-02）。
    pub fn widen_held<R, L, C, P>(
        &mut self,
        rows: &[R],
        (len, cell): (L, C),
        (held, provisional): (&[core::ops::Range<usize>], P),
    ) where
        L: Fn(&R) -> usize,
        C: for<'row> Fn(&'row R, usize) -> Option<&'row str>,
        P: Fn(usize) -> u32,
    {
        let widest = |column: usize| {
            rows.iter()
                .filter_map(|row| cell(row, column))
                .map(display_columns)
                .max()
                .unwrap_or(0)
                .clamp(MIN_COLUMN_COLUMNS, MAX_COLUMN_COLUMNS)
        };
        let count = rows.iter().map(&len).max().unwrap_or(0);
        for column in self.columns.len()..count {
            self.columns
                .push(provisional(column).clamp(MIN_COLUMN_COLUMNS, MAX_COLUMN_COLUMNS));
            self.unsettled.push(true);
        }
        let settling = held.iter().flat_map(|range| {
            let end = range.end.min(self.columns.len());
            range.start.min(end)..end
        });
        let columns: Vec<usize> = settling
            .filter(|&column| self.is_unsettled(column))
            .collect();
        for column in columns {
            if let (Some(unsettled), Some(width)) =
                (self.unsettled.get_mut(column), self.columns.get_mut(column))
            {
                *width = widest(column);
                *unsettled = false;
            }
        }
    }

    /// まだ幅を決めていない列か（窓に入っていない列・ADR 0033 の決定 3）。無い列は `false`。
    #[must_use]
    pub fn is_unsettled(&self, column: usize) -> bool {
        self.unsettled.get(column).copied().unwrap_or(false)
    }

    /// `at` 番目の前に、幅 `widths` の列を差し込む（列を足した・消した列を戻した・ADR 0027）。`at` が列の数より
    /// 後ろなら末尾に足す。
    pub fn insert(&mut self, at: usize, widths: &[u32]) {
        let at = at.min(self.columns.len());
        let tail = self.columns.split_off(at);
        self.columns.extend_from_slice(widths);
        self.columns.extend(tail);
        let unsettled_tail = self.unsettled.split_off(at.min(self.unsettled.len()));
        self.unsettled
            .extend(core::iter::repeat_n(false, widths.len()));
        self.unsettled.extend(unsettled_tail);
    }

    /// `at` 番目から `count` 列の幅を外して返す（列を消した）。
    pub fn remove(&mut self, at: usize, count: usize) -> Vec<u32> {
        let start = at.min(self.columns.len());
        let end = at.saturating_add(count).min(self.columns.len());
        self.unsettled
            .drain(start.min(self.unsettled.len())..end.min(self.unsettled.len()));
        self.columns.drain(start..end).collect()
    }

    /// 列ごとの字数。
    #[must_use]
    pub fn columns(&self) -> &[u32] {
        &self.columns
    }

    /// 利用者が変えた幅 `custom`（元の列 → 半角の字数）を重ねた幅。字数は `MIN_COLUMN_COLUMNS`〜`MAX_RESIZED_COLUMNS` に
    /// 収め、無い列は無視する（Issue #64）。
    #[must_use]
    pub fn resized(&self, custom: &BTreeMap<usize, u32>) -> Self {
        let mut columns = self.columns.clone();
        for (&column, &chars) in custom {
            if let Some(width) = columns.get_mut(column) {
                *width = chars.clamp(MIN_COLUMN_COLUMNS, MAX_RESIZED_COLUMNS);
            }
        }
        Self {
            columns,
            unsettled: self.unsettled.clone(),
        }
    }

    /// 表示の順 `order`（表示の位置 → 元の列）に並べ替えた幅。`order` に無い列は元の位置のまま。
    #[must_use]
    pub fn reordered(&self, order: &[usize]) -> Self {
        let sources: Vec<usize> = (0..self.columns.len())
            .map(|display| order.get(display).copied().unwrap_or(display))
            .collect();
        Self {
            columns: sources
                .iter()
                .filter_map(|&source| self.columns.get(source).copied())
                .collect(),
            unsettled: sources
                .iter()
                .filter_map(|&source| self.unsettled.get(source).copied())
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn rows(cells: &[&[&str]]) -> Vec<Vec<String>> {
        cells
            .iter()
            .map(|row| row.iter().map(|&cell| String::from(cell)).collect())
            .collect()
    }

    fn slices(rows: &[Vec<String>]) -> Vec<&[String]> {
        rows.iter().map(Vec::as_slice).collect()
    }

    fn len(row: &&[String]) -> usize {
        row.len()
    }

    fn cell<'row>(row: &'row &[String], column: usize) -> Option<&'row str> {
        row.get(column).map(String::as_str)
    }

    /// 窓の行が持たない列は仮の幅で足し、初めて持ったときにその列の値から決め、その後は変えない（ADR 0033 の決定 3）。
    #[test]
    fn columns_outside_the_held_spans_are_settled_when_first_held() {
        let table = rows(&[&["a", "東京都千代田区", "bb", "x"]]);
        let slices = slices(&table);
        let mut widths = ColumnWidths::fit(&[]);
        widths.widen_held(
            &slices,
            (len, cell),
            (&[0..1, 0..0], |column| u32::try_from(column).unwrap() + 5),
        );
        assert_eq!(widths.columns(), [3, 6, 7, 8]);
        assert!(!widths.is_unsettled(0) && widths.is_unsettled(1) && widths.is_unsettled(3));
        widths.widen_held(&slices, (len, cell), (&[1..3, 0..0], |_| 30));
        assert_eq!(widths.columns(), [3, 14, 3, 8]);
        assert!(!widths.is_unsettled(2) && widths.is_unsettled(3));
        // 決めた列は、別の値の行が来ても変えない。
        let wider = rows(&[&["a-very-long-value", "", "", ""]]);
        widths.widen_held(&self::slices(&wider), (len, cell), (&[0..4, 0..0], |_| 30));
        assert_eq!(widths.columns(), [3, 14, 3, 3]);
        assert!(!widths.is_unsettled(3));
        // 列を差し込んでも消しても、印は列といっしょに動く。
        widths.widen_held(&slices, (len, cell), (&[], |_| 9));
        widths.insert(0, &[8]);
        assert!(!widths.is_unsettled(0) && !widths.is_unsettled(4));
        assert_eq!(widths.remove(0, 1), [8]);
    }

    #[test]
    fn widths_are_inserted_and_removed_with_their_columns() {
        let table = rows(&[&["id", "東京都千代田区", "x"]]);
        let mut widths = ColumnWidths::fit(&slices(&table));
        widths.insert(1, &[MIN_COLUMN_COLUMNS]);
        assert_eq!(widths.columns(), &[3, 3, 14, 3]);
        assert_eq!(widths.remove(2, 5), vec![14, 3]);
        widths.insert(9, &[7]);
        assert_eq!(widths.columns(), &[3, 3, 7]);
    }

    #[test]
    fn columns_fit_their_content_within_bounds() {
        let long = "x".repeat(100);
        let table = rows(&[&["id", "東京都千代田区", long.as_str()], &["12345", "a"]]);
        let widths = ColumnWidths::fit(&slices(&table));
        assert_eq!(widths.columns(), &[5, 14, 40]);
        assert!(ColumnWidths::fit(&[]).columns().is_empty());
    }

    #[test]
    fn widening_adds_only_new_columns() {
        let first = rows(&[&["a", "b"]]);
        let mut widths = ColumnWidths::fit(&slices(&first));
        assert_eq!(widths.columns(), &[3, 3]);
        let later = rows(&[&["a very long value", "another long one", "c", "dddd"]]);
        widths.widen(&slices(&later));
        // 今ある 2 列は変えず、増えた 2 列だけを足す。
        assert_eq!(widths.columns(), &[3, 3, 3, 4]);
        widths.widen(&slices(&rows(&[&["x"]])));
        assert_eq!(widths.columns(), vec![3, 3, 3, 4].as_slice());
    }

    #[test]
    fn widths_follow_the_display_order() {
        let table = rows(&[&["a", "bbbbb", "cccccccc"]]);
        let widths = ColumnWidths::fit(&slices(&table));
        assert_eq!(widths.reordered(&[2, 0, 1]).columns(), &[8, 3, 5]);
        // 並びが短ければ、残りは元の位置のまま。
        assert_eq!(widths.reordered(&[1, 0]).columns(), &[5, 3, 8]);
    }

    #[test]
    fn resized_widths_override_clamp_and_ignore_missing_columns() {
        let cells = rows(&[&["a", "bbbbbbbb", "c"]]);
        let widths = ColumnWidths::fit(&slices(&cells));
        assert_eq!(widths.columns(), &[3, 8, 3]);
        let custom = BTreeMap::from([(0, 12), (1, 1), (2, 500), (9, 20)]);
        assert_eq!(widths.resized(&custom).columns(), &[12, 3, 200]);
        assert_eq!(widths.resized(&BTreeMap::new()), widths);
    }
}
