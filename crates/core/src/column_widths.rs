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
/// 後から列の多い行が来たら、その列だけを足す。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ColumnWidths {
    columns: Vec<u32>,
}

impl ColumnWidths {
    /// `rows` の中身に合わせ、各列を `MIN_COLUMN_COLUMNS`〜`MAX_COLUMN_COLUMNS` 字に収める。
    #[must_use]
    pub fn fit(rows: &[&[String]]) -> Self {
        let mut widths = Self {
            columns: Vec::new(),
        };
        widths.widen(rows);
        widths
    }

    /// `rows` に、今より多い列があれば、その列の幅だけを `rows` から決めて足す。今ある列の幅は変えない。
    pub fn widen(&mut self, rows: &[&[String]]) {
        let count = rows.iter().map(|row| row.len()).max().unwrap_or(0);
        for column in self.columns.len()..count {
            let widest = rows
                .iter()
                .filter_map(|row| row.get(column))
                .map(|cell| display_columns(cell))
                .max()
                .unwrap_or(0);
            self.columns
                .push(widest.clamp(MIN_COLUMN_COLUMNS, MAX_COLUMN_COLUMNS));
        }
    }

    /// `at` 番目の前に、幅 `widths` の列を差し込む（列を足した・消した列を戻した・ADR 0027）。`at` が列の数より
    /// 後ろなら末尾に足す。
    pub fn insert(&mut self, at: usize, widths: &[u32]) {
        let at = at.min(self.columns.len());
        let tail = self.columns.split_off(at);
        self.columns.extend_from_slice(widths);
        self.columns.extend(tail);
    }

    /// `at` 番目から `count` 列の幅を外して返す（列を消した）。
    pub fn remove(&mut self, at: usize, count: usize) -> Vec<u32> {
        let start = at.min(self.columns.len());
        let end = at.saturating_add(count).min(self.columns.len());
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
        Self { columns }
    }

    /// 表示の順 `order`（表示の位置 → 元の列）に並べ替えた幅。`order` に無い列は元の位置のまま。
    #[must_use]
    pub fn reordered(&self, order: &[usize]) -> Self {
        let columns = (0..self.columns.len())
            .filter_map(|display| {
                let source = order.get(display).copied().unwrap_or(display);
                self.columns.get(source).copied()
            })
            .collect();
        Self { columns }
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
