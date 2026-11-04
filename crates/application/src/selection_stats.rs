//! 選んだ範囲の集計（VIEW-14・Issue #42）。

use alloc::string::String;
use core::ops::RangeInclusive;

use nene_ledger_core::decimal::Decimal;
use nene_ledger_core::number_reading::read_number;

/// 選んだ範囲の集計。件数（空でないセル）・数の個数・合計（桁あふれなら `None`）・数として読めないセルの数。
/// 空のセルとセルが無いは数えない（RS-004）。数は 10 進で正確に足す。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionStats {
    cells: u64,
    numbers: u64,
    sum: Option<Decimal>,
    not_numbers: u64,
}

impl SelectionStats {
    /// 行 `rows`（復号した値）の、列 `columns` のセルを集計する。
    #[must_use]
    pub fn of(rows: &[&[String]], columns: &RangeInclusive<usize>) -> Self {
        let mut stats = Self::empty();
        for row in rows {
            for value in columns.clone().filter_map(|column| row.get(column)) {
                stats.add(value);
            }
        }
        stats
    }

    /// まだ何も数えていない集計（裏の仕事で 1 つずつ足していく）。
    #[must_use]
    pub fn empty() -> Self {
        Self {
            cells: 0,
            numbers: 0,
            sum: Some(Decimal::ZERO),
            not_numbers: 0,
        }
    }

    /// 行 `row`（復号した値）の、列 `columns` のセルを足す。セルが無い列は数えない。
    pub fn add_row(&mut self, row: &[String], columns: &[RangeInclusive<usize>]) {
        for value in columns
            .iter()
            .cloned()
            .flatten()
            .filter_map(|column| row.get(column))
        {
            self.add(value);
        }
    }

    /// セルの値（復号した値）`value` を 1 つ足す。空のセルは数えない。
    pub fn add(&mut self, value: &str) {
        if value.is_empty() {
            return;
        }
        self.cells = self.cells.saturating_add(1);
        match read_number(value) {
            Some(number) => {
                self.numbers = self.numbers.saturating_add(1);
                self.sum = self.sum.and_then(|sum| sum.checked_add(number));
            }
            None => self.not_numbers = self.not_numbers.saturating_add(1),
        }
    }

    /// 件数（空でないセル）。
    #[must_use]
    pub fn cells(self) -> u64 {
        self.cells
    }

    /// 数として読めたセルの数。
    #[must_use]
    pub fn numbers(self) -> u64 {
        self.numbers
    }

    /// 合計。桁あふれなら `None`。
    #[must_use]
    pub fn sum(self) -> Option<Decimal> {
        self.sum
    }

    /// 平均（数の個数で割る・小数 6 桁で四捨五入して末尾の 0 を落とす）。数が無いか桁あふれなら `None`。
    #[must_use]
    pub fn average(self) -> Option<Decimal> {
        self.sum?.average(self.numbers).map(Decimal::trimmed)
    }

    /// 数として読めなかったセルの数（空でないもの）。
    #[must_use]
    pub fn not_numbers(self) -> u64 {
        self.not_numbers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    fn table(cells: &[&[&str]]) -> Vec<Vec<String>> {
        cells
            .iter()
            .map(|row| row.iter().map(|&cell| String::from(cell)).collect())
            .collect()
    }

    fn stats(cells: &[&[&str]], columns: RangeInclusive<usize>) -> SelectionStats {
        let rows = table(cells);
        let slices: Vec<&[String]> = rows.iter().map(Vec::as_slice).collect();
        SelectionStats::of(&slices, &columns)
    }

    #[test]
    fn numbers_are_summed_exactly_and_the_rest_are_counted() {
        let figures = stats(&[&["0.1", "x"], &["0.2", ""], &["1,234", "5"]], 0..=1);
        // 空でないセル 5（空の 1 つは数えない）。数は 0.1・0.2・5 の 3 つ、数でないのは x と 1,234 の 2 つ。
        assert_eq!(figures.cells(), 5);
        assert_eq!(figures.numbers(), 3);
        assert_eq!(figures.not_numbers(), 2);
        assert_eq!(figures.sum(), Some(Decimal::new(53, 1)));
        // 5.3 / 3 = 1.766667。
        assert_eq!(figures.average(), Some(Decimal::new(1_766_667, 6)));
    }

    #[test]
    fn missing_cells_and_columns_outside_the_range_are_ignored() {
        let figures = stats(&[&["1", "2", "3"], &["4"]], 1..=2);
        assert_eq!((figures.cells(), figures.numbers()), (2, 2));
        assert_eq!(figures.sum(), Some(Decimal::new(5, 0)));
        assert_eq!(figures.average(), Some(Decimal::new(25, 1)));
        let words = stats(&[&["a"], &["b"]], 0..=0);
        assert_eq!((words.numbers(), words.average()), (0, None));
    }

    #[test]
    fn an_overflowing_sum_is_reported_as_none() {
        let huge = "9".repeat(38);
        let figures = stats(&[&[huge.as_str()], &[huge.as_str()]], 0..=0);
        assert_eq!(figures.numbers(), 2);
        assert_eq!(figures.sum(), None);
        assert_eq!(figures.average(), None);
    }
}
